// Check if the economy is fucked

use bigdecimal::BigDecimal;
use diesel::dsl::sum;
use diesel::{Connection, MysqlConnection};

use crate::prelude::*;
use diesel::prelude::*;
use diesel::result::Error;
use log::{debug, warn};

#[derive(Debug)]
pub enum InflationLeak {
    /// Too many doints are in circulation!
    TooMany,
    /// Not enough doints! Lossy system!
    TooFew,
}

// Collect taxes
impl EventCaller {
    /// Make sure the total money in circulation is the same as the amount that's supposed to be.
    ///
    /// Returns `Some()` if there is a leak
    pub fn inflation_check(conn: &mut MysqlConnection) -> Result<Option<InflationLeak>, Error> {
        debug!("Checking for inflation/deflation.");
        // TODO: admin warnings
        match conn.transaction(|conn| {
            // get the bank
            let the_bank: BankInfo = bank_table.first(conn)?;
            let expected_amount = the_bank.total_doints;

            // Tally up all the doints
            let mut all_doints: BigDecimal = the_bank.doints_on_hand;

            // Get how much money all users have
            let user_total: Option<BigDecimal> = users_table
                .select(sum(bal_col))
                .first(conn)
                .expect("Sum should always return 1 thing");
            let user_total: BigDecimal =
                user_total.expect("This always returns a number even on 0 rows");
            all_doints += user_total;

            // Does that match?
            if expected_amount == all_doints {
                // All good!
                debug!("No inflation/deflation detected.");
                return Ok(None);
            }

            warn!("The economy is leaking!");

            // There's a leak!
            if expected_amount > all_doints {
                // Not enough
                warn!("Doints are disappearing!");
                warn!("{} are missing!", expected_amount - all_doints);
                Ok(Some(InflationLeak::TooFew))
            } else {
                // Too many!
                warn!("Doints are being created!");
                warn!("{} over expected amount!", all_doints - expected_amount);
                Ok(Some(InflationLeak::TooMany))
            }
        }) {
            Ok(ok) => Ok(ok),
            Err(err) => {
                // Check failed
                warn!("Inflation check did not run! {err:#?}");
                Err(err)
            }
        }
    }
}
