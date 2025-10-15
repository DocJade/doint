// Things that happen at midnight every day.

use diesel::{Connection, MysqlConnection};
use log::info;

use crate::prelude::*;

impl EventCaller {
    /// Actions that run once a day. Doesn't run at a specific time, just every 24 hours after the bot starts.
    ///
    /// Returns true if all events worked correctly.
    pub fn daily_events(conn: &mut MysqlConnection) -> Result<bool, BotError> {
        do_daily_events(conn)
    }
}

pub fn do_daily_events(conn: &mut MysqlConnection) -> Result<bool, BotError> {
    info!("Running daily events...");
    // Do everything in a transaction.
    conn.transaction(|conn| {
        // Collect taxes
        info!("Collecting taxes...");
        let tax_collected = EventCaller::tax_time(conn)?;
        info!("Collected {tax_collected} in taxes...");

        // Give out money.
        // If we're too broke, we'll try again after another round of taxes.
        info!("Distribuiting UBI...");
        let rerun_ubi = EventCaller::ubi_time(conn)?.is_none();

        // Re-run UBI if needed
        if rerun_ubi {
            info!("Re-running taxes and UBI...");
            let tax_again = EventCaller::tax_time(conn)?;
            info!("Collected an additional {tax_collected} in taxes...");
            let _ = EventCaller::ubi_time(conn)?;
            // if that fails, oh well, we already tried
        }

        // All done.
        Ok(true)
    })
}
