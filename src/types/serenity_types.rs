use diesel::{r2d2::{self, ConnectionManager}, MysqlConnection};

// Error and context types
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations.
// This includes things like access to the database pool.


pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;


#[derive(Debug)]
pub struct Data {
    pub db_pool: DbPool
}