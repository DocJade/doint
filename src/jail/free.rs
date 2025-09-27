// Take the user out of jail

use chrono::Local;
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use log::{info, warn};

use crate::{database::tables::users::DointUser, jail::error::JailError};

impl DointUser {
    /// Free user from jail, if they are in jail and their sentence is up.
    ///
    /// Returns appropriate errors if user is not eligible for removal from jail.
    pub(crate) fn free_user_from_jail(self, conn: &mut MysqlConnection) -> Result<(), JailError> {
        go_free_user_from_jail(self, conn)
    }
}

// actual implementation
fn go_free_user_from_jail(user: DointUser, conn: &mut MysqlConnection) -> Result<(), JailError> {
    // Make sure they're still in jail
    let Some(jailed_user) = user.is_jailed(conn)? else {
        // User isn't in jail, cant free them.
        return Err(JailError::UserNotInJail);
    };

    // Are they done?
    let now = Local::now().naive_utc().and_utc().timestamp();
    if jailed_user.until.and_utc().timestamp() >= now {
        // They still have time to serve
        return Err(JailError::StillServingSentence);
    }

    // Free them!
    conn.transaction(|conn| {
        let result = diesel::delete(&jailed_user).execute(conn)?;
        // We expect to remove one entry, if not, we rollback to prevent damage.
        if result != 1 {
            // what
            warn!(
                "Tried to remove {result} rows from the jail table when we expected to remove 1!"
            );
            return Err(diesel::result::Error::RollbackTransaction);
        }
        Ok(())
    })?;

    // All done!
    info!("User [{}] was freed from jail.", user.id);
    Ok(())
}
