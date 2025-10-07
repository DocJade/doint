use chrono::Local;
use diesel::{
    BelongingToDsl, Connection, MysqlConnection, OptionalExtension, RunQueryDsl, SelectableHelper,
    query_dsl::methods::SelectDsl,
};
use log::{info, warn};

use crate::{
    database::tables::{jail::JailedUser, users::DointUser},
    models::{JailInterface, jail::JailError},
};

impl JailInterface {
    pub(crate) fn is_jailed(
        user: &DointUser,
        conn: &mut MysqlConnection,
    ) -> Result<Option<JailedUser>, JailError> {
        Ok(JailedUser::belonging_to(&user)
            .select(JailedUser::as_select())
            .first(conn)
            .optional()?)
    }

    pub(crate) fn free_user(user: &DointUser, conn: &mut MysqlConnection) -> Result<(), JailError> {
        impl_free_user(user, conn)
    }
}

fn impl_free_user(user: &DointUser, conn: &mut MysqlConnection) -> Result<(), JailError> {
    // Make sure they're in jail
    let Some(jailed_user) = JailInterface::is_jailed(user, conn)? else {
        return Err(JailError::UserNotInJail);
    };

    let now = Local::now().naive_utc().and_utc().timestamp();
    if jailed_user.until.and_utc().timestamp() >= now {
        // They still have time to serve
        return Err(JailError::StillServingSentence);
    }

    conn.transaction(|conn| {
        let result = diesel::delete(&jailed_user).execute(conn)?;
        // We expect to remove one entry, if not, we rollback to prevent damage.
        if result != 1 {
            warn!(
                "Tried to remove {result} rows from the jail table when we expected to remove 1!"
            );
            return Err(diesel::result::Error::RollbackTransaction);
        }
        Ok(())
    })?;

    info!("User `{}` was freed from jail", user.id);
    Ok(())
}

impl DointUser {
    #[inline]
    pub fn in_jail(&self, conn: &mut MysqlConnection) -> Result<Option<JailedUser>, JailError> {
        JailInterface::is_jailed(self, conn)
    }
    #[inline]
    pub fn free_from_jail(&self, conn: &mut MysqlConnection) -> Result<(), JailError> {
        JailInterface::free_user(self, conn)
    }
}
