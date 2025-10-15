#[cfg(test)]
mod formatter_tests {
    use bigdecimal::{BigDecimal, FromPrimitive};

    use crate::prelude::*;

    #[test]
    pub fn american() {
        let localized = BigDecimal::from_f64(123000.45).unwrap();

        assert_eq!(
            DointFormatter::display_doint_string(&localized, &DointFormatterPreference::American),
            "Đ123,000.45"
        );
    }

    #[test]
    pub fn european() {
        let localized = BigDecimal::from_f64(123000.45).unwrap();

        assert_eq!(
            DointFormatter::display_doint_string(&localized, &DointFormatterPreference::European),
            "Đ123.000,45"
        );
    }
}
