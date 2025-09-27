// Get the top n doint holders

use crate::schema::users::dsl::{bal, users};
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

use crate::database::tables::users::DointUser;

/// Get the top doint holders, up to a limit.
/// Sorted de
///
/// May return less than the limit if less than that many users exist.
pub(crate) fn get_top_n(
    limit: i64,
    conn: &mut MysqlConnection,
) -> Result<Vec<DointUser>, diesel::result::Error> {
    // Transaction not really needed here but lol.
    // Users table, ordered by balance, limited.
    conn.transaction(|conn| {
        users
            .order_by(bal.desc())
            .limit(limit)
            .load::<DointUser>(conn)
    })
}
