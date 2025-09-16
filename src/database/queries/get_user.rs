// Check if a user exists by an ID.

use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use crate::schema::users::dsl::{users, bal};

use crate::{database::tables::users::DointUser};

/// Gets the DointUser from a discord user ID, if they exist.
pub(crate) fn get_doint_user(id: impl Into<u64>, conn: &mut MysqlConnection) -> Result<Option<DointUser>, diesel::result::Error> {
    let id: u64 = id.into();
    // Transaction not really needed here but lol.
    // Users table, find the user.
    let maybe_user = conn.transaction(|conn|{
        users.find(id).load::<DointUser>(conn)
    })?;

    if maybe_user.is_empty() {
        return Ok(None)
    }
    Ok(Some(maybe_user[0]))
}