// This gets ran a lot, so only very quick things here!

use crate::prelude::*;

use diesel::{Connection, MysqlConnection, QueryDsl, RunQueryDsl};
use log::warn;

impl EventCaller {
    /// Runs every minute.
    ///
    /// Returns true if all events worked correctly.
    pub fn minute_events(conn: &mut MysqlConnection) -> Result<bool, BotError> {
        do_minute_events(conn)
    }
}

pub fn do_minute_events(conn: &mut MysqlConnection) -> Result<bool, BotError> {
    // Do everything in a transaction.
    conn.transaction(|conn| {
        // Loop over the people in jail and free them if we can.
        for in_jail in &jail_table.load::<JailedUser>(conn)? {
            let user = users_table.find(in_jail.id).get_result::<DointUser>(conn)?;
            // try freeing them
            if let Err(bad) = user.free_from_jail(conn) {
                match bad {
                    JailError::AlreadyInJail(_) => {
                        unreachable!("We aren't putting someone in jail.")
                    }
                    JailError::UserNotInJail => {
                        // Maybe they got removed between load and check?
                        warn!("Jail claims to not have user we just loaded from jail!");
                        // Skip this mf
                    }
                    JailError::StillServingSentence => {
                        // Can't free someone whos still in jail.
                    }
                    JailError::DieselError(error) => return Err(error.into()),
                }
            } else {
                // Free!
                // Freeing notifications handled elsewhere.
            }
        }

        // All done.
        Ok(true)
    })
}
