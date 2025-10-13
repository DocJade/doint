// How to display doint numbers

use bigdecimal::BigDecimal;

impl super::format_struct::FormattingHelper {
    /// Format doints for display based on user preferences
    #[must_use]
    pub fn display_doint(doints: &BigDecimal) -> String {
        // TODO: Allow users to set formatting preferences.

        let raw: String = format!("{doints:.2}");

        // Get everything before the decimal point so we can add commas
        let (pre_decimal, decimals) = raw
            .split_once('.')
            .expect("Should have a decimal component.");

        // Now reverse the pre to make adding the commas easier, and add commas
        let mut new_pre_decimal = String::new();

        for (index, char) in pre_decimal.chars().rev().enumerate() {
            if (index + 1) % 3 == 0 {
                // Add a comma as well
                new_pre_decimal.insert_str(0, &format!(",{char}"));
            } else {
                new_pre_decimal.insert(0, char);
            }
        }

        // Remove the comma at the end, if there is one.
        if new_pre_decimal.starts_with(',') {
            let _ = new_pre_decimal.remove(0);
        }

        // Add the doint symbol.
        new_pre_decimal.insert(0, crate::knob::formatting::DOINT_SYMBOL);

        // done.
        format!("{new_pre_decimal}.{decimals}")
    }
}
