use crate::models::queries;
use crate::prelude::*;

use diesel::prelude::*;
use diesel::{Connection, MysqlConnection};
use poise::serenity_prelude as serenity;

impl queries::Users {
    /// # Errors
    /// Returns `Err` if the query fails
    ///
    /// Returns a [`DointUser`] if the user with the respective `id` exists.
    pub fn get_doint_user(
        id: impl Into<u64>,
        conn: &mut MysqlConnection,
    ) -> Result<Option<DointUser>, diesel::result::Error> {
        let id: u64 = id.into();
        let maybe_user =
            conn.transaction(|conn| users_table.find(id).first::<DointUser>(conn).optional())?;

        Ok(maybe_user)
    }

    /// # Errors
    /// Returns `Err` if the query fails
    ///
    /// Retrieve a [`serenity::User`] from their id.
    pub async fn get_user_from_id(
        ctx: PoiseContext<'_>,
        id: u64,
    ) -> Result<serenity::User, BotError> {
        // If the user exists in the cache, return the cached value.
        if let Some(cached) = ctx.cache().user(id) {
            return Ok(cached.clone());
        }

        Ok(ctx.http().get_user(id.into()).await?)
    }
}
