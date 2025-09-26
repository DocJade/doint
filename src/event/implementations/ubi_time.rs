// UBI dispersal call

use bigdecimal::BigDecimal;
use diesel::MysqlConnection;

use crate::{bank::bank_struct::BankInterface, event::event_struct::EventCaller};
use diesel::result::Error;

// Collect taxes
impl EventCaller {
    /// Collect taxes as defined in the bank.
    pub(crate) fn ubi_time(conn: &mut MysqlConnection) -> Result<Option<BigDecimal>, Error> {
        // Call it
        BankInterface::disperse_ubi(conn)
    }
}