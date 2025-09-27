use bigdecimal::BigDecimal;
use diesel::MysqlConnection;

use crate::{bank::bank_struct::BankInterface, event::event_struct::EventCaller};
use diesel::result::Error;

// Collect taxes
impl EventCaller {
    /// Collect taxes as defined in the bank.
    pub(crate) fn tax_time(conn: &mut MysqlConnection) -> Result<BigDecimal, Error> {
        // Just call the taxes method.
        BankInterface::collect_taxes(conn)
    }
}
