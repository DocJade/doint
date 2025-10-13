// UBI dispersal call

use bigdecimal::BigDecimal;
use diesel::MysqlConnection;

use crate::{event::event_struct::EventCaller, models::BankInterface};
use diesel::result::Error;

// Collect taxes
impl EventCaller {
    /// Collect taxes as defined in the bank.
    pub fn ubi_time(conn: &mut MysqlConnection) -> Result<Option<BigDecimal>, Error> {
        // Call it
        BankInterface::disperse_ubi(conn)
    }
}
