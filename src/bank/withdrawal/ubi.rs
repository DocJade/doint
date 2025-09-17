// Universal basic income!

// Collect taxes from doint-holders.

use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use log::{debug, info, warn};
use crate::database::tables::bank::BankInfo;
use crate::schema::bank::dsl::bank;
use crate::schema::users::dsl::users;


use crate::{database::tables::users::DointUser};


/// Disperse UBI to all enrolled users.
/// 
/// The UBI rate is a percentage of all of the liquid doints currently in the bank, then that amount is split
/// between all dointers. Rounds down, with a minimum of 1 doint.
/// 
/// If the bank is too broke to afford UBI, dispersal will fail, returning None.
/// 
/// If UBI is disabled, IE the rate is set to 0, then Some(0) is returned.
/// 
/// Returns how many doints each user got.
/// 
/// Returns a diesel error if db stuff fails.
pub(crate) fn disperse_ubi(conn: &mut MysqlConnection) -> Result<Option<u32>, Error> {
    info!("Distributing universal basic income...");
    // Do this all in one go.
    // All of this rolls back if UBI could not be dispersed.
    conn.transaction::<Option<u32>, diesel::result::Error, _>(|conn| {
        // Load in the current state of the bank
        let the_bank: BankInfo = bank.first(conn)?;

        // Calculate the current ubi rate.
        // This is a multiplier, NOT a percentage.
        let ubi_rate: f64 = f64::from(the_bank.tax_rate) / 1000.0;

        // Skip UBI if disabled.
        if ubi_rate < 0.001 {
            // No taxes!
            info!("UBI disabled! Skipping!");
            return Ok(Some(0));
        }

        // Now calculate the resulting pool of money to give out.
        let amount_to_disperse = f64::from(the_bank.doints_on_hand) * ubi_rate;

        // Count how many doint-holders there are
        let mut people_to_pay: Vec<DointUser> = users.load::<DointUser>(conn)?;

        // If there is nobody to pay, we're done.
        if people_to_pay.is_empty() {
            debug!("Nobody to pay UBI to.");
            return Ok(Some(0));
        }

        // Now figure out how much to pay to each user.
        // Divide by how many people we need to pay
        #[allow(clippy::cast_precision_loss)] // If we have more than 2^52 users we have bigger issues
        let amount_per_person: f64 = amount_to_disperse / people_to_pay.len() as f64;

        // Now round that downwards to the nearest int.
        #[allow(clippy::cast_possible_truncation)] // Already truncated.
        let mut int_amount_per_person = amount_per_person.floor() as i32;

        // We want to pay out a minimum of at least 1 doint though,
        int_amount_per_person = int_amount_per_person.max(1);

        // Make sure the bank can afford that still
        #[allow(clippy::cast_possible_truncation)] // We're already fucked if we have more than i32::MAX users.
        #[allow(clippy::cast_possible_wrap)]
        let total_bank_removal = int_amount_per_person * people_to_pay.len() as i32;
        if  total_bank_removal > the_bank.doints_on_hand {
            // Bank cant afford it
            debug!("Bank cant afford UBI. Skipping.");
            return Ok(None)
        }

        // Juuust in case, make sure its positive
        if total_bank_removal <= 0 {
            // what
            warn!("Tried to give out negative money? {total_bank_removal}. Skipping.");
        }

        // Bank can afford it, start paying.
        
        
        // Now loop over every user, givin em money from the bank
        for user in &mut people_to_pay {
            // Give em that money
            user.bal += int_amount_per_person;
        }

        // Save those changes

        // Now everyone's balances has been updated.
        // Go apply all of the changes.
        for user in &people_to_pay {
            user.save_changes::<DointUser>(conn)?;
        }

        // Take that money out of the bank
        let mut update_bank: BankInfo = bank.first(conn)?;
        update_bank.doints_on_hand -= total_bank_removal;
        update_bank.save_changes::<BankInfo>(conn)?;

        // Make sure that didnt go negative.
        if update_bank.doints_on_hand < 0 {
            // how
            warn!("Paid too much UBI and bank went negative! Canceling!");
            // Needs to be an error to roll back changes.
            return Err(Error::RollbackTransaction);
        }

        // Done!
        // Must be positive at this point.
        Ok(Some(int_amount_per_person.try_into().expect("This should be >=1")))
    })
}