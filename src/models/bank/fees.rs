use crate::prelude::*;
use bigdecimal::BigDecimal;
use diesel::{Connection, MysqlConnection, RunQueryDsl};

impl BankInterface {
    /// Calculate the fees for a transaction.
    ///
    /// Returns a [`DieselError`][diesel::result::Error] if tax collection fails.
    pub fn calculate_fees(
        conn: &mut MysqlConnection,
        transaction_amount: &BigDecimal,
    ) -> Result<BigDecimal, diesel::result::Error> {
        go_calculate_fees(conn, transaction_amount)
    }
}

fn go_calculate_fees(
    conn: &mut MysqlConnection,
    transaction_amount: &BigDecimal,
) -> Result<BigDecimal, diesel::result::Error> {
    // Get the fee info
    let fee_info: FeeInfo = conn.transaction(|conn| fees_table.first(conn))?;

    let flat_fee: BigDecimal = fee_info.flat_fee;

    // Add the percentage fee.
    // Rounds down.
    let percent_fee = conversions::tax_rate_to_percentage_bd(fee_info.percentage_fee);

    let total_fee = (percent_fee * transaction_amount) + flat_fee;

    Ok(total_fee)
}
