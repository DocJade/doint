// steal moneys from people

use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use bigdecimal::ToPrimitive;
use diesel::Connection;
use log::{debug, warn};
use poise::serenity_prelude::Member;
use rand::rng;
use rand::seq::IndexedRandom;

use crate::bank::bank_struct::BankInterface;
use crate::bank::movement::move_doints::{DointTransfer, DointTransferParty, DointTransferReason};
use crate::database::queries::get_user::get_doint_user;
use crate::discord::checks::consented::ctx_member_enrolled_in_doints;
use crate::formatting::format_struct::FormattingHelper;
use crate::jail::arrest::JailForm;
use crate::jail::reasons::{JailCause, JailReason};
use crate::types::serenity_types::{Context, Error};


/// Rob someone. Odds of the robbery are based on wealth disparity.
#[poise::command(slash_command, guild_only)]
pub(crate) async fn rob(
    ctx: Context<'_>,
    #[description = "Who would you like to rob?"]
    who: Member,
) -> Result<(), Error> {
    debug!("User [{}] is robbing User [{}]!", ctx.author().id.get(), who.user.id.get());

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Get the user that is doing the robbery
    let Some(robber) = get_doint_user(ctx.author().id, &mut conn)? else {
        // Has role, but not in DB.
        // TODO: error for this / correction
        warn!("User not in DB!");
        let _ = ctx.say("Uhh, you're not in the doint DB properly, tell doc.").await?;
        return Ok(());
    };

    // get the user that is getting robbed.
    let Some(victim) = get_doint_user(who.user.id, &mut conn)? else {
        let _ = ctx.say("You cant rob someone who isn't a Dointer!").await?;
        return Ok(());
    };
    
    // Make sure you arent robbing yourself
    if robber.id == victim.id {
        // no
        let _ = ctx.say("You cant rob yourself!").await?;
        return Ok(());
    }

    // figure out the wealth disparity between the two users.
    // If a user is robbing someone with a lot of money, its more likely to succeed without sending them to jail.

    // put together the jail form early
    let jail_form: JailForm = JailForm {
        law_broke: JailReason::AttemptedRobbery,
        arrested_by: JailCause::ThePolice,
        jail_for: None, // Standard robbery, so none
        can_bail: false // Currently unused.
    };

    // if victim has less than half of the robbers bal, then thats fucked up, so we just jail the robber.
    // The same is also true if the victim is completely broke.
    if &robber.bal / 2 > victim.bal || victim.bal == BigDecimal::zero() {
        // TO JAIL!
        robber.jail_user(&jail_form, &mut conn)?;
        
        let _ = ctx.say("Mf robbing poor people, straight to jail.").await?;
        return Ok(())
    }
    
    // Robbing people in jail sends you to jail
    if victim.is_jailed(&mut conn)?.is_some() {
        robber.jail_user(&jail_form, &mut conn)?;
        let _ = ctx.say("You snuck into jail to rob them, thats breaking and entering! You've been sent to jail!").await?;
        return Ok(())
    }

    // the chance of success is based on how much the person you're robbing has.
    // if they have 2x the money you do, you are 50% likely to succeed, if they have 3x, 66%, etc
    // to prevent people with no money from always winning the robbery, we'll have a minimum failure odds of 10%

    // Do note the math we get here is the chance of succeeding the robbery

    // We explicitly check
    let robbery_odds: f64 = if robber.bal == BigDecimal::zero() {
        // 10% failure rate, thus 90% win rate
        0.90
    } else {
        let raw_odds = (victim.bal.to_f64().expect("Should fit") / robber.bal.to_f64().expect("Should fit")) / 10.0;
        
        // max odds of 90% win rate
        raw_odds.min(0.90)
    };

    debug!("Odds of this robbery working are {robbery_odds:.3}");

    // Now the amount you steal is also based on wealth disparity, so its percentage based.

    // You can steal up to 5% of victim's bal, times win rate. Thus harder steals pay less.
    // On top of that, its also random, for fun.

    let max_steal = victim.bal.to_f64().expect("Should fit.") * 0.05 * robbery_odds;

    // We also round down the steal amount.
    // Yes its possible to steal 0, we'll check for that.
    #[allow(clippy::cast_possible_truncation)] // Already floored.
    #[allow(clippy::cast_sign_loss)] // We floor it, this shouldn't ever be negative.
    let steal_amount: BigDecimal = BigDecimal::from_f64(rand::random_range(0.0..max_steal).floor()).expect("Should fit.");

    // If the steal amount is zero, special case.
    if steal_amount == BigDecimal::zero() {
        // lol
        debug!("Robbery canceled, would have robbed 0 doint.");
        ctx.say("You were going to rob them, but you forgot to take your ADHD meds and forgot.").await?;
        return Ok(())
    }

    // Now flip the odds.
    let robbery_worked = rand::random_bool(robbery_odds);

    if !robbery_worked {
        // Robbery failed.
        // Send them to jail.
        let failure_message = format!("{}\nYou've been sent to jail for attempted robbery!", get_robbery_flavor_text(false));
        robber.jail_user(&jail_form, &mut conn)?;
        ctx.say(failure_message).await?;
        return Ok(())
    }

    // Robbery worked!
    // Take the money!
    conn.transaction(|conn| {
        let transfer: DointTransfer = DointTransfer {
            sender: DointTransferParty::DointUser(victim.id),
            recipient: DointTransferParty::DointUser(robber.id),
            transfer_amount: steal_amount.clone(),
            apply_fees: false, // this is theft
            transfer_reason: DointTransferReason::CrimeRobbery
        };

        BankInterface::bank_transfer(conn, transfer)
    })?;
    
    // Inform user
    let victory_message = format!("{} {}!", get_robbery_flavor_text(true), FormattingHelper::display_doint(&steal_amount));
    ctx.say(victory_message).await?;

    Ok(())
}





// Dumb reasons as to why the robbery worked or failed.
fn get_robbery_flavor_text(worked: bool) -> String {
    if worked {
        (*SUCCESS_FLAVOR.choose(&mut rng()).expect("there are always messages")).to_string()
    } else {
        (*FAIL_FLAVOR.choose(&mut rng()).expect("there are always messages")).to_string()
    }
}

const SUCCESS_FLAVOR: [&str; 5] = [
    "You ran by and stole their hat worth",
    "You ran off with",
    "You tied their shoelaces together while you leafed through their wallet, taking",
    "You pointed a banana at them, and they thought it was a gun! Ran off with",
    "You dipped into their back pocket, which contained 3 jelly beans, a fish skeleton, and",
];

const FAIL_FLAVOR: [&str; 5] = [
    "You asked a cop what the best way to rob somebody was, and they didn't think that was very funny!",
    "You sneezed while reaching into their backpack, and they called the police!",
    "You reached into their back pocket, but it wasn't a back pocket... Shit!",
    "Some french lady started yelling when you walked near the target, alerting the police!",
    "You tried to mug them with a banana, but they didn't fall for it!",
];