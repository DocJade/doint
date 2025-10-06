// This gets ran a lot, so only very quick things here!

use crate::database::tables::jail::JailedUser;
use crate::database::tables::users::DointUser;
use crate::schema::jail::dsl::jail;
use crate::schema::users::dsl::users;
use diesel::{Connection, MysqlConnection, QueryDsl, RunQueryDsl};
use log::warn;

use crate::{
    event::event_struct::EventCaller,
    types::serenity_types::Error,
};

impl EventCaller {
    /// Runs every minute.
    ///
    /// Returns true if all events worked correctly.
    pub(crate) fn minute_events(conn: &mut MysqlConnection) -> Result<bool, Error> {
        do_minute_events(conn)
    }
}

pub(crate) fn do_minute_events(conn: &mut MysqlConnection) -> Result<bool, Error> {
    // Do everything in a transaction.
    conn.transaction(|conn| {
        // Loop over the people in jail and free them if we can.
        for in_jail in &jail.load::<JailedUser>(conn)? {
            let user = users.find(in_jail.id).get_result::<DointUser>(conn)?;
            // try freeing them
            if let Err(bad) = user.free_user_from_jail(conn) {
                match bad {
                    crate::jail::error::JailError::AlreadyInJail(_) => {
                        unreachable!("We aren't putting someone in jail.")
                    }
                    crate::jail::error::JailError::UserNotInJail => {
                        // Maybe they got removed between load and check?
                        warn!("Jail claims to not have user we just loaded from jail!");
                        // Skip this mf
                    }
                    crate::jail::error::JailError::StillServingSentence => {
                        // Can't free someone whos still in jail.
                    }
                    crate::jail::error::JailError::DieselError(error) => return Err(error.into()),
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
