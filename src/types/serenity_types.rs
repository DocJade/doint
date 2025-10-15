use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use crate::prelude::*;

pub type Context<'a> = poise::Context<'a, Data, BotError>;

// User data, which is stored and accessible in all command invocations.
// This includes things like access to the database pool.
pub type DbPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug)]
pub struct Data {
    pub db_pool: DbPool,
}
