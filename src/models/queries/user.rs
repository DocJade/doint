// user CRUD functions

use crate::models::queries;
use crate::schema::users::dsl::users;
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

use crate::models::data::users::DointUser;

impl queries::Users {
    /// Returns a `DointUser` if the user with the respective `id` exists.
    pub(crate) fn get_doint_user(
        id: impl Into<u64>,
        conn: &mut MysqlConnection,
    ) -> Result<Option<DointUser>, diesel::result::Error> {
        let id: u64 = id.into();
        let maybe_user =
            conn.transaction(|conn| users.find(id).first::<DointUser>(conn).optional())?;

        Ok(maybe_user)
    }
}
