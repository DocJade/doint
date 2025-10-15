// See your doint balance

use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::Connection;
use poise::serenity_prelude::Member;

use crate::prelude::{helper::get_nick::get_display_name, *};

/// See your doint balance.
#[poise::command(slash_command, guild_only, aliases("bal"), check = guards::in_doints_category, check = guards::in_commands)]
pub async fn balance(ctx: Context<'_>) -> Result<(), BotError> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Get the user, if they dont exist, return false.
    let Some(user) = Users::get_doint_user(ctx.author().id, &mut conn)? else {
        // Couldn't find em.
        // TODO: When commands fail, tell the user the reason instead of just silence.
        return Ok(());
    };

    let preference = if let Some(member) = &ctx.author().member {
        if let Some(user) = &member.user {
            DointFormatterPreference::from(user)
        } else {
            crate::knob::formatting::FORMATTER_PREFERENCE
        }
    } else {
        crate::knob::formatting::FORMATTER_PREFERENCE
    };

    // Format the doint number
    let doint_string = DointFormatter::display_doint_string(&user.bal, &preference);

    // Now print out their balance.
    let response: String = format!("You currently have {doint_string}.");

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}

/// Get another user's doint balance, for a fee.
#[poise::command(slash_command, guild_only, aliases("sn"), check = guards::in_doints_category)]
pub async fn snoop(
    ctx: Context<'_>,
    #[description = "Who do you want to snoop on?"] victim: Member,
) -> Result<(), BotError> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    let Some(executor) = Users::get_doint_user(ctx.author().id, &mut conn)? else {
        // Couldn't find em.
        ctx.reply("You don't exist!").await?;
        return Ok(());
    };

    let cost: BigDecimal = BigDecimal::from_i32(50).expect("Should always exist");

    // Make sure user has enough
    if executor.bal < cost {
        ctx.say("You don't have enough Doints for this").await?;
        return Ok(());
    }

    conn.transaction(|conn| {
        let transfer = DointTransfer::new(
            DointTransferParty::DointUser(executor.id),
            DointTransferParty::Bank,
            cost.clone(),
            false,
            DointTransferReason::BalSnoop,
        );

        if let Err(e) = transfer {
            return Err(BotError::from(e));
        }

        Ok(BankInterface::bank_transfer(conn, transfer.unwrap()))
    })??;

    // Get the user, if they dont exist, return false.
    let Some(victim) = Users::get_doint_user(victim.user.id, &mut conn)? else {
        // Couldn't find em.
        ctx.reply("User doesn't exist, no refunds!").await?;
        return Ok(());
    };

    let preference = if let Some(member) = &ctx.author().member {
        if let Some(user) = &member.user {
            DointFormatterPreference::from(user)
        } else {
            crate::knob::formatting::FORMATTER_PREFERENCE
        }
    } else {
        crate::knob::formatting::FORMATTER_PREFERENCE
    };

    // Format the doint number
    let doint_string = DointFormatter::display_doint_string(&victim.bal, &preference);

    // Now print out their balance.
    let response: String = format!(
        "{} currently has {doint_string}.\n\n-# Paid a fee of {}.",
        get_display_name(ctx, victim.id).await?,
        DointFormatter::display_doint_string(&cost, &preference)
    );

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}
