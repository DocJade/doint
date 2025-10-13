// leaderboard CRUD functions

use crate::models::queries;
use crate::schema::users::dsl::{bal, users};
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

use crate::models::data::users::DointUser;

impl queries::Leaderboard {
    /// Get the users with the highest doint balances.
    ///
    /// If the number of users is less than `limit`, all users will be returned.
    pub(crate) fn get_top_doint_balances(
        limit: i64,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<DointUser>, diesel::result::Error> {
        conn.transaction(|conn| {
            users
                .order_by(bal.desc())
                .limit(limit)
                .load::<DointUser>(conn)
        })
    }

    /// Get the users with the lowest doint balances.
    ///
    /// If the number of users is less than `limit`, all users will be returned.
    pub(crate) fn get_bottom_doint_balances(
        limit: i64,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<DointUser>, diesel::result::Error> {
        conn.transaction(|conn| {
            users
                .order_by(bal.asc())
                .limit(limit)
                .load::<DointUser>(conn)
        })
    }
}
