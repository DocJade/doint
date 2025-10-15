// Pay another user some of your doints.

use bigdecimal::{BigDecimal, FromPrimitive};
use log::{debug, warn};
use poise::serenity_prelude::Member;

use crate::prelude::{helper::get_nick::get_display_name, *};

/// Pay another player
#[poise::command(slash_command, guild_only, check = guards::in_doints_category)]
pub async fn pay(
    ctx: Context<'_>,
    #[description = "Who you are paying."] recipient: Member,
    #[description = "The amount of doints to pay them."] payment: f64,
) -> Result<(), Error> {
    // Turn that float into a BigDecimal
    let Some(payment) = BigDecimal::from_f64(payment) else {
        // Failed to cast!
        return Err(Error::BigDecimalCastError);
    };

    debug!(
        "User [{}] is attempting to pay user [{}] {} doints.",
        ctx.author().id.get(),
        recipient.user.id.get(),
        payment
    );

    let preference = if let Some(member) = &ctx.author().member {
        if let Some(user) = &member.user {
            DointFormatterPreference::from(user)
        } else {
            crate::knob::formatting::FORMATTER_PREFERENCE
        }
    } else {
        crate::knob::formatting::FORMATTER_PREFERENCE
    };

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // User cannot pay self
    if ctx.author().id.get() == recipient.user.id.get() {
        // bruh
        debug!("User tried to pay self. Not allowed. Skipping.");
        let _ = ctx.say("Paying yourself? Bruh, No.").await?;
        return Ok(());
    }

    // Make sure the recipient is opted in
    if !member_enrolled_in_doints(recipient.clone(), ctx)? {
        // Recipient is not enrolled.
        debug!("Person user was trying to pay was not a dointer. Not allowed. Skipping.");
        let _ = ctx.say("You cant pay them, they aren't a dointer.").await?;
        return Ok(());
    }

    let transfer = DointTransfer::new(
        DointTransferParty::DointUser(ctx.author().id.get()),
        DointTransferParty::DointUser(recipient.user.id.get()),
        payment,
        true, // pay command is taxed.
        DointTransferReason::GenericUserPayment,
    );

    // Run the bank transfer
    let transfer_result = match transfer {
        Err(e) => Err(DointTransferError::ConstructionFailed(e)),
        Ok(transfer) => BankInterface::bank_transfer(&mut conn, transfer),
    };

    // Did that work?
    let receipt = match transfer_result {
        Ok(ok) => ok,
        Err(err) => match err {
            DointTransferError::SenderInsufficientFunds(details) => {
                // Broke ass.
                debug!("User cant afford the transfer. Cancelled.");
                let fee_money = DointFormatter::display_doint_string(
                    &details.fees_required.expect("/pay has fees"),
                    &preference,
                );
                let broke_response: String = format!(
                    "You cannot afford that.\nYou may need to factor in the transaction fee of {fee_money}."
                );
                let _ = ctx.say(broke_response).await?;
                return Ok(());
            }
            DointTransferError::RecipientFull => {
                // They have too much money
                debug!("Recipient has too much money. Cancelled.");
                let _ = ctx
                    .say("Recipient can't have any more money. They win.")
                    .await?;
                return Ok(());
            }
            DointTransferError::InvalidParty => {
                // This shouldn't happen
                debug!("One of the parties in the transaction is invalid. Cancelled.");
                let _ = ctx.say("One of the parties in the transaction was invalid.\nThat shouldn't happen, tell Doc.").await?;
                return Ok(());
            }
            DointTransferError::TransferFeesOnBank => {
                // Not doin that
                unreachable!("/pay isnt a tax collector")
            }
            DointTransferError::ZeroTransfer => {
                let _ = ctx.say("Cannot transfer zero doints.").await?;
                return Ok(());
            }

            DointTransferError::SameParty => {
                // already checked higher.
                unreachable!("Can't pay self")
            }
            DointTransferError::TransferTooBig => {
                // This shouldn't happen
                let _ = ctx
                    .say("Somehow your payment was too big.\nThat shouldn't happen, tell Doc.")
                    .await?;
                return Ok(());
            }
            DointTransferError::InvalidTransferReason => {
                // This shouldn't happen since we do user transfer types.
                unreachable!("/pay should have a valid transfer reason.");
            }
            DointTransferError::ConstructionFailed(e) => {
                let _ = ctx.say(format!("Your transfer was invalid: {e}")).await?;

                return Ok(());
            }
            DointTransferError::DieselError(error) => {
                // Well.
                warn!("Transfer was valid, but DB failed! Cancelled.");
                let _ = ctx.say("Payment failed for a DB reason. Tell Doc.").await?;
                return Err(Error::DieselError(error));
            }
        },
    };

    // Payment happened, tell user
    debug!("User was paid.");
    // TODO: ledger stuff

    // Format the amount sent
    let amount_string = DointFormatter::display_doint_string(&receipt.amount_sent, &preference);

    // Format the transfer fee
    let fee_string: String = DointFormatter::display_doint_string(
        &receipt.fees_paid.expect("/pay has fees"),
        &preference,
    );

    // Get the name of the recipient, or if that fails, just say `them`
    let recipient_name: String = match get_display_name(ctx, recipient.user.id.get()).await {
        Ok(ok) => ok,
        Err(_) => "them".to_string(),
    };

    // put that all together
    let response: String = format!(
        "You've paid {recipient_name} {amount_string}.\nYou paid a transfer fee of {fee_string}."
    );

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}
