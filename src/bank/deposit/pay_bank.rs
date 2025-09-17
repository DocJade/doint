// Pay directly into the bank

use crate::database::queries::get_user::get_doint_user;
use crate::database::tables::users::DointUser;
use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use log::warn;
use crate::bank::bank_struct::BankInterface;
use crate::database::tables::bank::BankInfo;
use crate::schema::bank::dsl::bank;

impl BankInterface {
    /// Take money from a user and put it in the bank.
    pub(crate) fn pay_bank(conn: &mut MysqlConnection, user: DointUser, amount: i32) -> Result<(), Error> {
        go_pay_bank(conn, user, amount)
    }
}

fn go_pay_bank(conn: &mut MysqlConnection, user: DointUser, amount: i32) -> Result<(), Error> {
    // Re-load in the user, just in case.
    // Everything is done in a transaction.
    conn.transaction(|conn|{
        // Load in the user
        let mut found_user = if let Some(found) = get_doint_user(user.id, conn)? {
            found
        } else {
           // User not in DB? Crazy.
           return Err(Error::NotFound) 
        };

        // Load in the bank
        let mut the_bank: BankInfo = bank.first(conn)?;

        // Make sure they have enough money to give us, and that the requested change is not negative
        if amount < 0 || amount > found_user.bal {
            // Cannot afford, or negative input.
            warn!(
                "Attempted to transfer {amount} doints from user [{}] to the bank.\
                Either this number is negative, or they could not afford it with their balance of {}",
                found_user.id, found_user.bal
            );
            return Err(Error::RollbackTransaction) 
        }

        // Take their money, and give it to ourselves.
        found_user.bal -= amount;
        the_bank.doints_on_hand += amount;

        // Save those.
        found_user.save_changes::<DointUser>(conn)?;
        the_bank.save_changes::<BankInfo>(conn)?;

        // All done!
        return Ok(());
    })
}