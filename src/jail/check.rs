// Check if a user is in jail.
// Impl on the user, easier to work with.

use diesel::{query_dsl::methods::SelectDsl, BelongingToDsl, MysqlConnection, OptionalExtension, RunQueryDsl, SelectableHelper};

use crate::{database::tables::{jail::JailedUser, users::DointUser}, jail::error::JailError};


// Impl it on DointUser for ease of use.
impl DointUser {
    /// Check if this user is in jail.
    /// 
    /// Returns a `JailedUser`.
    pub(crate) fn is_jailed(self, conn: &mut MysqlConnection) -> Result<Option<JailedUser>, JailError> {
        check_user_in_jail(self, conn)
    }
}



// actual implementation
fn check_user_in_jail(user: DointUser, conn: &mut MysqlConnection) -> Result<Option<JailedUser>, JailError> {
    Ok(JailedUser::belonging_to(&user).select(JailedUser::as_select()).first(conn).optional()?)
}