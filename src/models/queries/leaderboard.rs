// leaderboard CRUD functions

use crate::prelude::*;
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

impl Leaderboard {
    /// Get the users with the highest doint balances.
    ///
    /// If the number of users is less than `limit`, all users will be returned.
    pub fn get_top_doint_balances(
        limit: i64,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<DointUser>, diesel::result::Error> {
        conn.transaction(|conn| {
            users_table
                .order_by(users_bal_table.desc())
                .limit(limit)
                .load::<DointUser>(conn)
        })
    }

    /// Get the users with the lowest doint balances.
    ///
    /// If the number of users is less than `limit`, all users will be returned.
    pub fn get_bottom_doint_balances(
        limit: i64,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<DointUser>, diesel::result::Error> {
        conn.transaction(|conn| {
            users_table
                .order_by(users_bal_table.asc())
                .limit(limit)
                .load::<DointUser>(conn)
        })
    }
}
