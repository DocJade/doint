// Universal basic income!

// Collect taxes from doint-holders.

use crate::models::BankInterface;
use crate::database::tables::bank::BankInfo;
use crate::schema::bank::dsl::bank;
use crate::schema::users::dsl::users;
use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use log::{debug, info, warn};

use crate::database::tables::users::DointUser;

impl BankInterface {
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
    pub(crate) fn disperse_ubi(conn: &mut MysqlConnection) -> Result<Option<BigDecimal>, Error> {
        go_disperse_ubi(conn)
    }
}

fn go_disperse_ubi(conn: &mut MysqlConnection) -> Result<Option<BigDecimal>, Error> {
    info!("Distributing universal basic income...");
    // Do this all in one go.
    // All of this rolls back if UBI could not be dispersed.
    conn.transaction::<Option<BigDecimal>, diesel::result::Error, _>(|conn| {
        // Load in the current state of the bank
        let the_bank: BankInfo = bank.first(conn)?;

        // Calculate the current ubi rate.
        // This is a multiplier, NOT a percentage.
        let ubi_rate: BigDecimal = BigDecimal::from_f64(f64::from(the_bank.ubi_rate) / 1000.0)
            .expect("idk this should be fine");

        // Skip UBI if disabled.
        if ubi_rate < BigDecimal::from_f64(0.001).expect("Fine.") {
            // No taxes!
            info!("UBI disabled! Skipping!");
            return Ok(Some(BigDecimal::zero()));
        }

        // Now calculate the resulting pool of money to give out.
        let amount_to_disperse: BigDecimal = &the_bank.doints_on_hand * &ubi_rate;

        // Count how many doint-holders there are
        let mut people_to_pay: Vec<DointUser> = users.load::<DointUser>(conn)?;

        // If there is nobody to pay, we're done.
        if people_to_pay.is_empty() {
            debug!("Nobody to pay UBI to.");
            return Ok(Some(BigDecimal::zero()));
        }

        // Now figure out how much to pay to each user.
        let number_of_users =
            BigDecimal::from_usize(people_to_pay.len()).expect("This should be fine! :)");
        // Divide by how many people we need to pay
        #[allow(clippy::cast_precision_loss)]
        // If we have more than 2^52 users we have bigger issues
        let unrounded_amount_per_person: BigDecimal = &amount_to_disperse / &number_of_users;

        // Now round that downwards to the nearest dent
        let mut amount_per_person = unrounded_amount_per_person.round(2);

        // We want to pay out a minimum of at least 1 doint though,
        amount_per_person = amount_per_person.max(BigDecimal::one());

        // Make sure the bank can afford that still
        #[allow(clippy::cast_possible_truncation)] // We're already fucked if we have more than i32::MAX users.
        #[allow(clippy::cast_possible_wrap)]
        let total_bank_removal = &amount_per_person * number_of_users;
        if total_bank_removal > the_bank.doints_on_hand {
            // Bank cant afford it
            debug!("Bank cant afford UBI. Skipping.");
            return Ok(None);
        }

        // Juuust in case, make sure its positive
        if total_bank_removal <= BigDecimal::zero() {
            // what
            warn!("Tried to give out negative money? {total_bank_removal}. Skipping.");
        }

        // Bank can afford it, start paying.

        // Now loop over every user, givin em money from the bank
        for user in &mut people_to_pay {
            // Give em that money
            user.bal += &amount_per_person;
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
        if update_bank.doints_on_hand < BigDecimal::zero() {
            // how
            warn!("Paid too much UBI and bank went negative! Canceling!");
            // Needs to be an error to roll back changes.
            return Err(Error::RollbackTransaction);
        }

        // Done!
        // Must be positive at this point.
        Ok(Some(amount_per_person))
    })
}
