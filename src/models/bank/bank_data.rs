use bigdecimal::BigDecimal;
use diesel::{Connection, MysqlConnection, RunQueryDsl, SaveChangesDsl};

use crate::prelude::*;

impl BankInterface {
    /// Returns the balance of a `BankInterface`.
    ///
    /// Returns a [`DieselError`][diesel::result::Error] if retrieving fails
    pub fn get_bank_balance(
        conn: &mut MysqlConnection,
    ) -> Result<BigDecimal, diesel::result::Error> {
        get_balance(conn)
    }
}

fn get_balance(conn: &mut MysqlConnection) -> Result<BigDecimal, diesel::result::Error> {
    conn.transaction(|conn| {
        let the_bank: BankInfo = bank_table.first(conn)?;
        Ok(the_bank.doints_on_hand)
    })
}

impl BankInterface {
    /// Change the tax rate of the bank.
    ///
    /// Returns a bool on if the tax rate was set or not.
    /// Returns a [`DieselError`][diesel::result::Error] if change fails.
    pub fn set_tax_rate(conn: &mut MysqlConnection, new_rate: u16) -> bool {
        go_set_tax_rate(conn, new_rate)
    }

    /// Change the UBI rate of the bank.
    ///
    /// Returns a bool on if the tax rate was set or not.
    /// Returns a [`DieselError`][diesel::result::Error] if change fails.
    pub fn set_ubi_rate(conn: &mut MysqlConnection, new_rate: u16) -> bool {
        go_set_ubi_rate(conn, new_rate)
    }
}

fn go_set_tax_rate(conn: &mut MysqlConnection, new_rate: u16) -> bool {
    // Can't set tax_rate over 100%.
    if new_rate > 1000 {
        return false;
    }

    let result = conn.transaction(|conn| {
        let mut update_bank: BankInfo = bank_table.first(conn)?;
        update_bank.tax_rate = new_rate as i16;
        update_bank.save_changes::<BankInfo>(conn)
    });

    result.is_ok()
}

fn go_set_ubi_rate(conn: &mut MysqlConnection, new_rate: u16) -> bool {
    // Can't set tax_rate greater than 100%.
    if new_rate > 1000 {
        return false;
    }

    let result = conn.transaction(|conn| {
        let mut update_bank: BankInfo = bank_table.first(conn)?;
        update_bank.ubi_rate = new_rate as i16;
        update_bank.save_changes::<BankInfo>(conn)
    });

    result.is_ok()
}
