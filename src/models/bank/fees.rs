use crate::models::BankInterface;
use crate::models::bank::conversions;
use crate::models::data::fees::FeeInfo;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{Connection, MysqlConnection, RunQueryDsl};

use crate::schema::fees::dsl::fees;

impl BankInterface {
    /// Calculate the fees for a transaction.
    ///
    /// Returns a [`DieselError`][diesel::result::Error] if tax collection fails.
    pub(crate) fn calculate_fees(
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
    let fee_info: FeeInfo = conn.transaction(|conn| fees.first(conn))?;

    let mut total_fee: BigDecimal = fee_info.flat_fee;

    // Add the percentage fee.
    // Rounds down.
    let percent_fee: BigDecimal = BigDecimal::from_f64(
        conversions::tax_rate_to_percentage(fee_info.percentage_fee)
            .floor()
            .abs(),
    )
    .expect("Should always be valid");

    let mut calculated_percent_fee_int: BigDecimal = transaction_amount * percent_fee;

    // Round up the flat fee to the nearest dent
    calculated_percent_fee_int = calculated_percent_fee_int.round(2);

    total_fee += calculated_percent_fee_int;

    Ok(total_fee)
}
