// Hashtag just hourly things.

use diesel::{Connection, MysqlConnection};
use log::{info, warn};

use crate::prelude::*;

impl EventCaller {
    /// Actions that run once a day. Doesn't run at a specific time, just every 24 hours after the bot starts.
    ///
    /// Returns true if all events worked correctly.
    pub fn hourly_events(conn: &mut MysqlConnection) -> Result<bool, Error> {
        do_hourly_events(conn)
    }
}

pub fn do_hourly_events(conn: &mut MysqlConnection) -> Result<bool, Error> {
    info!("Running hourly events...");
    // Do everything in a transaction.
    conn.transaction(|conn| {
        // all checks pass?
        let mut canary = true;

        // Check for inflation/deflation
        info!("- - Inflation / deflation check");
        if let Some(kind) = EventCaller::inflation_check(conn)? {
            // Inflation detected!
            // TODO: inform admins
            warn!("INFLATION/DEFLATION DETECTED!");
            warn!("TYPE: {kind:#?}!");
            canary = false;
        } else {
            // All good
        }

        // All done.
        Ok(canary)
    })

    // Did that all work?
}
