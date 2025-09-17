// Pay another user some of your doints.

use diesel::associations::HasTable;
use diesel::{Connection, QueryDsl, RunQueryDsl, SaveChangesDsl};
use log::{debug, warn};
use poise::serenity_prelude::Member;

use crate::bank::bank_struct::BankInterface;
use crate::database::queries::get_user::get_doint_user;
use crate::database::tables::users::DointUser;
use crate::discord::checks::consented::{ctx_member_enrolled_in_doints, member_enrolled_in_doints};
use crate::discord::helper::get_nick::get_display_name;
use crate::formatting::format_struct::FormattingHelper;
use crate::types::serenity_types::{Context, Error};
use crate::schema::users::dsl::users;

/// Pay another player
#[poise::command(slash_command, guild_only, check="ctx_member_enrolled_in_doints")]
pub(crate) async fn pay(
    ctx: Context<'_>,
    #[description = "Who you are paying."]
    recipient: Member,
    #[description = "The amount of doints to pay them. 100 is 1.00"]
    payment: i32,
) -> Result<(), Error> {
    debug!("User [{}] is attempting to pay user [{}] {} doints.", ctx.author().id.get(), recipient.user.id.get(), payment);

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
    if !member_enrolled_in_doints(recipient.clone(), ctx).await? {
        // Recipient is not enrolled.
        debug!("Person user was trying to pay was not a dointer. Not allowed. Skipping.");
        let _ = ctx.say("You cant pay them, they aren't a dointer.").await?;
        return Ok(())
    }

    // No negative payments.
    if payment < 0 {
        debug!("User tried to pay a negative amount. Not allowed. Skipping.");
        let _ = ctx.say("Ha ha, very funny. You cant use this to steal money.").await?;
        // TODO: Put user in jail for attempted robbery.
        return Ok(())
    }

    // Gotta pay at least one doint.
    if payment == 0 {
        debug!("User tried to pay 0 doints. Not allowed. Skipping.");
        let _ = ctx.say("You cant pay somebody nothing.").await?;
        return Ok(())
    }

    // Calculate the transfer fee
    let transfer_fee: i32 = BankInterface::calculate_fees(&mut conn, payment.try_into().unwrap())?.try_into().expect("Transfer fees should be less than 2 billion doints");

    let mut payer_cannot_afford: bool = false;

    // Start the transfer so we can roll everything back
    conn.transaction(|conn| {
        // Check if the sender can afford the transaction including the fees
        let mut sender = get_doint_user(ctx.author().id, conn)?.expect("They must be there to call the command");
        if sender.bal < payment + transfer_fee {
            // Cannot afford it.
            payer_cannot_afford = true;
            // Cancel
            return Ok(());
        }

        // They can afford it.

        // Find the recipient
        let mut im_getting_paid: DointUser = if let Some(found) = users::table().find(recipient.user.id.get()).load::<DointUser>(conn)?.first() {
            *found
        } else {
            // This should be impossible. We didn't find them.
            warn!("Tried to find the recipient of payment, they have the dointer role, but werent found in the DB! Canceling!");
            return Err(diesel::result::Error::RollbackTransaction);
        };

        // Send 'em the money
        im_getting_paid.bal += payment;

        // Take the money from the sender
        sender.bal -= payment;

        // Now put the fees in the bank lil bro
        BankInterface::pay_bank(conn, sender, transfer_fee)?;

        // Finalize
        im_getting_paid.save_changes::<DointUser>(conn)?;
        sender.save_changes::<DointUser>(conn)?;

        // Transaction done!
        Ok(())
    })?;

    // Format the amount sent
    let amount_string = FormattingHelper::display_doint(payment);
    
    // Format the transfer fee
    let fee_string = FormattingHelper::display_doint(transfer_fee);


    // Could user afford it?
    if payer_cannot_afford {
        // Broke ass
        let broke_response: String = format!("You cannot afford that.\nYou may need to factor in the transaction fee of {fee_string}.");
        let _ = ctx.say(broke_response).await?;
        return Ok(());
    }

    // Transfer actually happened.
    debug!("User was paid.");

    // Inform user.
    // Get the name of the recipient, or if that fails, just say `them`
    let recipient_name: String = match get_display_name(ctx, recipient.user.id.get()).await {
        Ok(ok) => ok,
        Err(_) => "them".to_string(),
    };
    
    // put that all together
    let response: String = format!("You've paid {recipient_name} {amount_string}.\nYou paid a transfer fee of {fee_string}.");

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}