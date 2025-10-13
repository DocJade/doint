// Flip a coin, double your money!

use bigdecimal::{BigDecimal, FromPrimitive as _, Zero};
use diesel::Connection;
use log::{debug, warn};

use crate::formatting::format_struct::FormattingHelper;
use crate::guards;
use crate::models::BankInterface;
use crate::models::bank::transfer::{DointTransfer, DointTransferParty, DointTransferReason};
use crate::models::queries::Users;
use crate::types::serenity_types::{Context, Error};

// a coin
#[derive(Debug, poise::ChoiceParameter, PartialEq, Eq)]
enum Coin {
    #[name = "Heads"]
    Heads,
    #[name = "Tails"]
    Tails,
}

/// Flip a coin, pick a side. If you pick the correct side, you double your money (minus fees)
#[poise::command(slash_command, guild_only, user_cooldown = 300, check = guards::in_doints_category, check = guards::in_casino)]
pub async fn flip(
    ctx: Context<'_>,
    #[description = "Heads or tails?"] side: Coin,
    #[description = "How much are you betting? You can bet a maximum of 1,000.00"]
    #[max = 1000] // 1,000 doints
    bet: f64,
) -> Result<(), Error> {
    // Turn that float into a BigDecimal
    let Some(bet) = BigDecimal::from_f64(bet) else {
        // Failed to cast!
        return Err(Error::BigDecimalCastError);
    };

    debug!(
        "User [{}] is playing coin flip, they bet {bet} and picked {side:?}",
        ctx.author().id.get()
    );

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Get the user that is betting
    let Some(better) = Users::get_doint_user(ctx.author().id, &mut conn)? else {
        // Has role, but not in DB.
        // TODO: error for this / correction
        warn!("User not in DB!");
        let _ = ctx
            .say("Uhh, you're not in the doint DB properly, tell doc.")
            .await?;
        return Ok(());
    };

    // Make sure the user can afford the bet.
    let final_bet_amount: BigDecimal = if better.bal < bet {
        // User cant afford bet, Guess the'll bet it ALL! HAHA
        debug!("User tried to bet more than they have, defaulting to all of their money...");
        better.bal.clone()
    } else {
        bet.clone()
    };

    // Can't flip nothing.
    if final_bet_amount <= BigDecimal::zero() {
        debug!("Flip was worth 0.");
        let _ = ctx.say("Bet something, will ya?!").await?;
        return Ok(());
    }

    // Make sure bank can afford the bet
    if BankInterface::get_bank_balance(&mut conn)? < final_bet_amount {
        // bank couldn't pay this bet out
        debug!("Bank cant afford bet.");
        let _ = ctx
            .say("The bank doesn't have enough money for that bet, sorry.")
            .await?;
        return Ok(());
    }

    // The fee that would be paid if the user wins.
    let fees_to_pay = BankInterface::calculate_fees(&mut conn, &final_bet_amount)?;

    // If the fees are more than or equal to the possible winnings, the flip is pointless.
    if fees_to_pay >= final_bet_amount {
        debug!("Fees outweigh possible winnings.");
        let _ = ctx
            .say("Fees on this flip would cost more than the winnings.")
            .await?;
        return Ok(());
    }

    // Do the coin flip.
    // Heads or tails buddy?
    let flip = if rand::random_bool(0.5) {
        // 50%
        Coin::Heads
    } else {
        Coin::Tails
    };

    // Now move money around
    let receipt = conn.transaction(|conn| {
        // If the user lost, just take their money
        if flip != side {
            // Lost!
            let transfer = DointTransfer {
                sender: DointTransferParty::DointUser(ctx.author().id.get()),
                recipient: DointTransferParty::Bank,
                transfer_amount: final_bet_amount.clone(),
                apply_fees: false, // fees get applied after wins
                transfer_reason: DointTransferReason::CasinoLoss,
            };

            // Need the receipt in both cases, since we need to know fees.
            return BankInterface::bank_transfer(conn, transfer);
        }

        // User won!
        // Remember to deduce their fee.
        let take_home = &final_bet_amount - &fees_to_pay;
        let transfer = DointTransfer {
            sender: DointTransferParty::Bank,
            recipient: DointTransferParty::DointUser(ctx.author().id.get()),
            transfer_amount: take_home,
            apply_fees: false, // already added in.
            transfer_reason: DointTransferReason::CasinoWin,
        };

        BankInterface::bank_transfer(conn, transfer)
    })?;

    // Build message.
    // "[Heads/Tails]! You Won 1.23!\n\n-# Paid a fee of 0.05. (1.28 - 0.05 = 12.3)"
    // "[Heads/Tails]! You lost 1.23!\n\n-# Better luck next time!"

    let side_name = if flip == Coin::Heads {
        "Heads"
    } else {
        "Tails"
    };
    let bet_size = FormattingHelper::display_doint(&final_bet_amount);

    // rest of the owl
    let response = if flip == side {
        // win
        let paid_fees = FormattingHelper::display_doint(&fees_to_pay);
        let takeaway = FormattingHelper::display_doint(&receipt.amount_sent);
        format!(
            "{side_name}! You won {takeaway}!\n\n-# Paid a fee of {paid_fees}. ({bet_size} - {paid_fees} = {takeaway})"
        )
    } else {
        // | || || |_
        format!("{side_name}! You lost {bet_size}!\n\n-# Better luck next time!")
    };

    // If they bet more than they're worth, we truncated the value. Make sure they know that happened.
    let final_response = if final_bet_amount < bet {
        // Over-bet
        format!("You went all in!\n{response}")
    } else {
        // No adjustment was made
        response
    };

    // Send it.
    let _ = ctx.say(final_response).await?;
    Ok(())
}
