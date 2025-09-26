// See how much moneys the bank has

use bigdecimal::BigDecimal;
use diesel::{Connection, MysqlConnection, RunQueryDsl};

use crate::database::tables::bank::BankInfo;
use crate::bank::bank_struct::BankInterface;

use crate::schema::bank::dsl::bank;

impl BankInterface {
    /// See how much money is in the bank.
    /// 
    /// Returns error if we cant get the bal
    pub(crate) fn get_bank_balance(conn: &mut MysqlConnection) -> Result<BigDecimal, diesel::result::Error> {
        go_get_bank_balance(conn)
    }
}

fn go_get_bank_balance(conn: &mut MysqlConnection) -> Result<BigDecimal, diesel::result::Error> {
    conn.transaction(|conn|{
        let the_bank: BankInfo = bank.first(conn)?;
        Ok(the_bank.doints_on_hand)
    })
}