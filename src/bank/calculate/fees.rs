// Transactional fees

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
    pub(crate) fn calculate_fees(conn: &mut MysqlConnection, transaction_amount: u32) -> Result<u32, diesel::result::Error> {
        go_calculate_fees(conn, transaction_amount)
    }
}

fn go_calculate_fees(conn: &mut MysqlConnection, transaction_amount: u32) -> Result<u32, diesel::result::Error> {

    // Get the rates
    let fee_info: FeeInfo = conn.transaction(|conn| {
        fees.first(conn)
    })?;

    // Keep track of the fees
    // Start with the flat fee
    #[allow(clippy::cast_sign_loss)] // Database constrained to be positive.
    let mut total_fee: u32 = fee_info.flat_fee as u32;

    // Add the percentage fee.
    // Rounds down.
    let percent_fee = f64::from(fee_info.percentage_fee) / 1000.0;
    #[allow(clippy::cast_sign_loss)] // Will be positive
    #[allow(clippy::cast_possible_truncation)] // Floored
    let calculated_percent_fee_int: u32 = (f64::from(transaction_amount) * percent_fee).floor() as u32;

    total_fee += calculated_percent_fee_int;

    // All done
    Ok(total_fee)
}