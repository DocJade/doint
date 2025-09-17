// Collect taxes from doint-holders.

use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use log::info;
use crate::bank::bank_struct::BankInterface;
use crate::database::tables::bank::BankInfo;
use crate::schema::bank::dsl::bank;
use crate::schema::users::bal;
use crate::schema::users::dsl::users;


use crate::{database::tables::users::DointUser};

impl BankInterface {
    /// Immediately collect taxes from all users.
    /// 
    /// Taxes are based on a percentage of all of your doints at the moment taxes are taken.
    /// 
    /// Returns how many doints were collected into the bank.
    /// 
    /// Returns a diesel error if tax collection fails.
    pub(crate) fn collect_taxes(conn: &mut MysqlConnection) -> Result<u32, Error> {
        go_collect_taxes(conn)
    }
}


fn go_collect_taxes(conn: &mut MysqlConnection) -> Result<u32, Error> {
    info!("Collecting taxes...");
    // Do this all in one go.
    // If any of this fails, the entire transaction will be rolled back, and taxes will not be collected.
    conn.transaction::<u32, diesel::result::Error, _>(|conn| {
        // Load in the current state of the bank
        let the_bank: BankInfo = bank.first(conn)?;

        // Calculate the current tax rate.
        // This is a multiplier, NOT a percentage.
        let tax_rate: f64 = f64::from(the_bank.tax_rate) / 1000.0;

        // if the tax rate is zero, we can skip all taxation.
        // Tax rate is done in tenths of a percent, so if its under that, taxes are zero.
        // Yes we coulda just compared earlier, shush.
        if tax_rate < 0.001 {
            // No taxes!
            info!("Tax rate is zero. Skipping!");
            return Ok(0)
        }


        // Now we need to tax everyone.
        // No need to tax people with no money, or with negative money.
        let mut to_update: Vec<DointUser> = users.filter(bal.gt(0)).load::<DointUser>(conn)?;

        // Now loop over every user, figuring out how much to take from each of them.
        // We also keep track of how much money we have gathered
        let mut collected_taxes: i32 = 0;
        for user in &mut to_update {
            // Figure out how much to take
            let adjustment_float = f64::from(user.bal) * tax_rate;
            // Then we round that upwards to the nearest integer
            #[allow(clippy::cast_possible_truncation)] // Already rounded up.
            let adjustment_int: i32 = adjustment_float.ceil() as i32;

            // Now we need to bound that, since if somehow the math ends up negative, we would
            // steal money from the bank, and we also cant take more money than the user has.

            // Cant take more money than they have
            let mut final_adjustment = std::cmp::min(user.bal, adjustment_int);

            // You must pay in at least 1 doint.
            // Tough luck if that's your last doint.
            final_adjustment = std::cmp::max(final_adjustment, 1);

            // Now we have our amount to adjust by, remove that from the user
            user.bal -= final_adjustment;

            // And put it back into the bank.
            // This must be a positive number now.
            collected_taxes += final_adjustment;
        }

        // Now everyone's balances has been updated.
        // Go apply all of the changes.
        for user in &to_update {
            user.save_changes::<DointUser>(conn)?;
        }

        // Now that all of the taxes have been deducted from users, put it in the bank.
        let mut update_bank: BankInfo = bank.first(conn)?;
        update_bank.doints_on_hand += collected_taxes;
        update_bank.save_changes::<BankInfo>(conn)?;

        // Taxes got!
        info!("Tax collection finished!");
        info!("Collected [{collected_taxes}] doints via taxes.");
        #[allow(clippy::cast_sign_loss)] // This is positive.
        Ok(collected_taxes as u32)
    })
}