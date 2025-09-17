use diesel::MysqlConnection;

use crate::{bank::deposit::taxes::collect_taxes, event::event_struct::EventCaller};
use diesel::result::Error;

// Collect taxes
impl EventCaller {
    /// Collect taxes as defined in the bank.
    pub(crate) fn tax_time(conn: &mut MysqlConnection) -> Result<u32, Error> {
        // Just call the taxes method.
        collect_taxes(conn)
    }
}