// leaderboard CRUD functions

use crate::schema::users::dsl::{bal, users};
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};

use crate::database::tables::users::DointUser;

pub(crate) enum DointBalanceSortMode {
    HighestBalance,
    LowestBalance,
}

/// Get the users with the highest (or lowest) doint balances.
/// 
/// If the number of users is less than `limit`, all users will be returned.
pub(crate) fn get_doint_balance_leaderboard(
    limit: i64,
    mode: DointBalanceSortMode,
    conn: &mut MysqlConnection,
) -> Result<Vec<DointUser>, diesel::result::Error> {
    conn.transaction(|conn| {
        users
            .order_by(match DointBalanceSortMode {
                DointBalanceSortMode::HighestBalance => bal.desc(),
                DointBalanceSortMode::LowestBalance => bal.asc(),
            })
            .limit(limit)
            .load::<DointUser>(conn)
    })
}
