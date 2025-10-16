use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use crate::prelude::*;

// User data, which is stored and accessible in all command invocations.
// This includes things like access to the database pool.
pub type DbPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug)]
pub struct PoiseContextData {
    pub db_pool: DbPool,
}

pub type PoiseContext<'a> = poise::Context<'a, PoiseContextData, BotError>;

pub type GuildMember = poise::serenity_prelude::Member;
