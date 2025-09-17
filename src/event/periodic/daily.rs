// Things that happen at midnight every day.

use diesel::{Connection, MysqlConnection};
use log::info;

use crate::{
    event::event_struct::EventCaller, types::serenity_types::{
        Context,
        Data,
        Error
    }
};

/// Actions that happen every day at midnight.
/// 
/// Returns true if all events worked correctly.
pub(crate) fn daily_events(conn: &mut MysqlConnection) -> Result<bool, Error> {
    info!("Running daily events...");
    // Do everything in a transaction.
    conn.transaction(|conn|{
        // Give out money.
        // If we're too broke, we'll try again after taxes.
        info!("Distribuiting UBI...");
        let rerun_ubi = EventCaller::ubi_time(conn)?.is_none();
        
        // Collect taxes
        info!("Collecting taxes...");
        let tax_collected = EventCaller::tax_time(conn)?;

        // Re-run UBI if needed
        if rerun_ubi {
            info!("Re-running UBI...");
            let _ = EventCaller::ubi_time(conn)?;
            // if that fails, oh well, we already tried
        }

        // All done.
        Ok(true)
    })
}