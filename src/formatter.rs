use bigdecimal::BigDecimal;
use poise::serenity_prelude::User;

pub struct DointFormatter {}

pub enum DointFormatterPreference {
    American, // 1,000,000.000
    European, // 1.000.000,000
}

impl From<&User> for DointFormatterPreference {
    fn from(user: &User) -> Self {
        if let Some(locale) = &user.locale
            && *locale == "en-US".to_owned()
        {
            DointFormatterPreference::American
        } else {
            DointFormatterPreference::European
        }
    }
}

pub type DelimiterPair = (char, char);

impl DointFormatter {
    #[must_use] pub fn display_doint_string(
        doints: &BigDecimal,
        preference: &DointFormatterPreference,
    ) -> String {
        let raw = format!("{doints:.2}");

        let (pre, decimals) = raw
            .split_once('.')
            .expect("Should have a decimal component.");

        let (pre_delimiter, post_delimiter) = DointFormatter::get_delimiter(preference);

        let mut post = String::new();

        for (index, char) in pre.chars().rev().enumerate() {
            if (index + 1) % 3 == 0 {
                post.insert_str(0, &format!("{pre_delimiter}{char}"));
            } else {
                post.insert(0, char);
            }
        }

        if post.starts_with(pre_delimiter) {
            post.remove(0);
        }

        format!(
            "{}{post}{post_delimiter}{decimals}",
            crate::knob::formatting::DOINT_SYMBOL
        )
    }

    #[inline]
    #[must_use] pub fn get_delimiter(preference: &DointFormatterPreference) -> DelimiterPair {
        match preference {
            DointFormatterPreference::American => (',', '.'),
            DointFormatterPreference::European => ('.', ','),
        }
    }
}
