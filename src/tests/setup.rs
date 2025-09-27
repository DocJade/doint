// Setups for tests, and helpers for things the tests need.

use std::env;

use diesel::{
    Connection, MysqlConnection,
    r2d2::{self, ConnectionManager},
};
use dotenvy::dotenv;

/// You need to call TEST transactions on this or you will be altering the actual DB!
pub(super) fn get_test_db()
-> diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::MysqlConnection>> {
    // Get env vars for the DB url
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Open the database
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool!");

    // Now we obviously dont want the DB to be live, so we immediately go into a test db state.
    db_pool
        .get()
        .expect("Opening the DB failed. Test failure non-test related.")
}

/// Makes a test user with specified parameters, returns their id.
pub(super) fn make_test_user() -> u64 {
    todo!()
}
