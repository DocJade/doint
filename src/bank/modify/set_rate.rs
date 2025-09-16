// Set the tax rate of the bank.

use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use log::info;
use crate::database::tables::bank::BankInfo;
use crate::schema::bank::dsl::bank;
use crate::schema::users::bal;
use crate::schema::users::dsl::users;

/// Change the tax rate of the bank.
/// 
/// Returns a Diesel error if change fails.
/// 
/// Returns a bool on if the tax rate was set or not.
pub(crate) fn set_tax_rate(conn: &mut MysqlConnection, new_rate: u16) -> bool {
    // Bounds check that new rate
    if new_rate > 1000 {
        // Too high, cant set.
        return false;
    }
    let result = conn.transaction(|conn| {
        let mut update_bank: BankInfo = bank.first(conn)?;
        update_bank.tax_rate = new_rate as i16;
        update_bank.save_changes::<BankInfo>(conn)
    });

    if result.is_err() {
        // Didn't set.
        return false
    }

    // Must have been set.
    true
}