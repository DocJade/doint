/// Convert a `tax_rate` into a percentage by dividing by 1000.
#[inline]
#[must_use] pub fn tax_rate_to_percentage(bank_tax_rate: i16) -> f64 {
    f64::from(bank_tax_rate) / 1000.0
}
