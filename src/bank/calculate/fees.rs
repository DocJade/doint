// Transactional fees

use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{Connection, MysqlConnection, RunQueryDsl};

use crate::{bank::bank_struct::BankInterface, database::tables::fees::FeeInfo};

use crate::schema::fees::dsl::fees;

impl BankInterface {
    /// Put in how much the user is "spending", and the returned amount will be the
    /// fees alone. Flat + percentage.
    /// 
    /// Fees will always be positive.
    /// 
    /// Returns diesel error if DB stuff dies.
    pub(crate) fn calculate_fees(conn: &mut MysqlConnection, transaction_amount: &BigDecimal) -> Result<BigDecimal, diesel::result::Error> {
        go_calculate_fees(conn, transaction_amount)
    }
}

fn go_calculate_fees(conn: &mut MysqlConnection, transaction_amount: &BigDecimal) -> Result<BigDecimal, diesel::result::Error> {

    // Get the rates
    // Start with the flat fee
    let fee_info: FeeInfo = conn.transaction(|conn| {
        fees.first(conn)
    })?;

    let mut total_fee: BigDecimal = fee_info.flat_fee;

    // Add the percentage fee.
    // Rounds down.
    let percent_fee: BigDecimal = BigDecimal::from_f64((f64::from(fee_info.percentage_fee) / 1000.0).floor().abs()).expect("hehe");

    let mut calculated_percent_fee_int: BigDecimal = transaction_amount * percent_fee;

    // Round up the flat fee to the nearest dent
    calculated_percent_fee_int = calculated_percent_fee_int.round(2);

    total_fee += calculated_percent_fee_int;

    // All done
    Ok(total_fee)
}