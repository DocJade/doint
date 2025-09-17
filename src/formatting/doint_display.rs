// How to display doint numbers

use crate::{formatting::format_struct::FormattingHelper, knob::formatting::DOINT_SYMBOL};

impl FormattingHelper {
    /// Format doints for display based on user preferences
    pub(crate) fn display_doint(doints: i32) -> String {
        // TODO: Allow users to set formatting preferences.

        // Right now we display doints with 2 decimal places.
        // Thus we must have at least 3 digits. So left pad with zeros.
        let mut padded_raw: String = format!("{doints:03}");
        
        // Now insert the period
        padded_raw.insert(padded_raw.len() - 2, '.');

        // Now finally, add the doint symbol.
        padded_raw.insert(0, DOINT_SYMBOL);

        // done.
        padded_raw
    }
}