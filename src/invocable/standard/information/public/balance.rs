// See your doint balance

use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::Connection;
use poise::serenity_prelude::Member;

use crate::models::queries::Users;
use crate::discord::helper::get_nick::get_display_name;
use crate::formatting::format_struct::FormattingHelper;
use crate::guards;
use crate::models::BankInterface;
use crate::models::bank::transfer::{DointTransfer, DointTransferParty, DointTransferReason};
use crate::types::serenity_types::{Context, Error};

/// See your doint balance.
#[poise::command(slash_command, guild_only, aliases("bal"), check = guards::in_doints_category, check = guards::in_commands)]
pub(crate) async fn balance(ctx: Context<'_>) -> Result<(), Error> {
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

    // Format the doint number
    let doint_string = FormattingHelper::display_doint(&user.bal);

    // Now print out their balance.
    let response: String = format!("You currently have {doint_string}.");

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}

/// Get another user's doing balance, for a fee.
#[poise::command(slash_command, guild_only, aliases("sn"), check = guards::in_doints_category)]
pub(crate) async fn snoop(
    ctx: Context<'_>,
    #[description = "Who do you want to snoop on?"] victim: Member,
) -> Result<(), Error> {
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
        let transfer = DointTransfer {
            sender: DointTransferParty::DointUser(executor.id),
            recipient: DointTransferParty::Bank,
            transfer_amount: cost.clone(),
            apply_fees: false,
            transfer_reason: DointTransferReason::BalSnoop,
        };

        BankInterface::bank_transfer(conn, transfer)
    })?;

    // Get the user, if they dont exist, return false.
    let Some(victim) = Users::get_doint_user(victim.user.id, &mut conn)? else {
        // Couldn't find em.
        ctx.reply("User doesn't exist, no refunds!").await?;
        return Ok(());
    };

    // Format the doint number
    let doint_string = FormattingHelper::display_doint(&victim.bal);

    // Now print out their balance.
    let response: String = format!(
        "{} currently has {doint_string}.\n\n-# Paid a fee of {}.",
        get_display_name(ctx, victim.id).await?,
        FormattingHelper::display_doint(&cost)
    );

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}
