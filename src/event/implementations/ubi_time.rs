// UBI dispersal call

use diesel::MysqlConnection;

use crate::{bank::{deposit::taxes::collect_taxes, withdrawal::ubi::disperse_ubi}, event::event_struct::EventCaller};
use diesel::result::Error;

// Collect taxes
impl EventCaller {
    /// Collect taxes as defined in the bank.
    pub(crate) fn ubi_time(conn: &mut MysqlConnection) -> Result<Option<u32>, Error> {
        // Call it
        disperse_ubi(conn)
    }
}