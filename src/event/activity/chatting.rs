// People get rewarded for talking in doccord.

use crate::prelude::*;
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use poise::serenity_prelude::Message;

use crate::{event::activity::activity_reward_struct::ActivityRewardHelper, models::BankInterface};

impl ActivityRewardHelper {
    /// Reward a user for sending messages.
    ///
    /// Rewards are scaled based on message complexity / entropy.
    pub fn reward_talking(msg: &Message, data: &PoiseContextData) {
        // Using entropy for scoring messages is nice, since it boils down a lot of complex ideas
        // (Rewarding message length, ignoring repeated characters, etc).

        // This gives us a value between 0 and infinity. Example values:

        // "I prefer carbon monoxide in my drinks, it has a more pure taste than co2"
        // 4.04963
        // "because they stole it from tv youtube"
        // 3.7435389
        // "Hi"
        // 1
        // "siughoshuerlhjsorihujsdoirthjoruiothj09r8"
        // 3.5836725

        // We don't wanna give out _too_ big of rewards for talking, so we keep these numbers small.
        // We will div all these values by 100, which from testing should give between 0 and 5 dents.

        let entropy = shannon_entropy(&msg.content) / 100.0;

        // If we cant cast this for some reason, just cancel the entire operation
        let Some(mut transfer_amount) = BigDecimal::from_f32(entropy) else {
            return;
        };

        // Round that to be proper.
        transfer_amount = transfer_amount.round(2);

        // We can skip everything if this would have been a 0 doint transfer.
        if transfer_amount == BigDecimal::zero() {
            return;
        }

        // Get the database pool
        let pool = data.db_pool.clone();

        // Get a connection. If this doesnt work, we'll just bail
        let mut conn = match pool.get() {
            Ok(ok) => ok,
            Err(_) => return,
        };

        // Create a transfer, this will automatically fail if the bank cannot pay this out, or if the user
        // cannot accept it for some reason.

        // If this fails for any reason, just don't pay the user, since this isnt critical.

        let Ok(transfer) = DointTransfer::new(
            DointTransferParty::Bank,
            DointTransferParty::DointUser(msg.author.id.get()),
            transfer_amount,
            false, // No need.
            DointTransferReason::ActivityReward,
        ) else {
            return;
        };

        // Give the user the bonus doints!

        // We don't care if this fails, or even if it works.

        let _ = BankInterface::bank_transfer(&mut conn, transfer);
    }
}

// I dont wanna import an entire crate just for a single entropy calculation, so here it is.
// https://docs.rs/entropy/latest/src/entropy/lib.rs.html#14-33

/// Get the entropy of some data.
pub fn shannon_entropy<T: AsRef<[u8]>>(data: T) -> f32 {
    let bytes = data.as_ref();
    let mut entropy = 0.0;
    let mut counts = [0usize; 256];

    for &b in bytes {
        counts[b as usize] += 1;
    }

    for &count in &counts {
        if count == 0 {
            continue;
        }

        let p: f32 = (count as f32) / (bytes.len() as f32);
        entropy -= p * p.log(2.0);
    }

    entropy
}
