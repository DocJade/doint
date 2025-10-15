// user CRUD functions

use crate::models::queries;
use crate::prelude::*;

use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

impl queries::Users {
    /// # Errors
    ///
    /// Returns a `DointUser` if the user with the respective `id` exists.
    pub fn get_doint_user(
        id: impl Into<u64>,
        conn: &mut MysqlConnection,
    ) -> Result<Option<DointUser>, diesel::result::Error> {
        let id: u64 = id.into();
        let maybe_user =
            conn.transaction(|conn| users_table.find(id).first::<DointUser>(conn).optional())?;

        Ok(maybe_user)
    }
}
