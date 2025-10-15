use bigdecimal::BigDecimal;

/// Convert a `tax_rate` into a percentage by dividing by 1000.
#[inline]
#[must_use]
pub fn tax_rate_to_percentage_bd(bank_tax_rate: i16) -> BigDecimal {
    let rate = BigDecimal::from(bank_tax_rate);

    rate / BigDecimal::from(1000)
}
