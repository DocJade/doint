// get a Serenity User from a ID
// Tries the cache first.

use crate::types::serenity_types::{Context, Error, Data};
use poise::serenity_prelude as serenity;

/// Get the User that this ID refers to.
/// 
/// Tries reading from cache first.
pub(crate) async fn get_user_from_id(ctx: Context<'_>, id: u64) -> Result<serenity::User, Error> {
    if let Some(cached) = ctx.cache().user(id) {
        // User was cached.
        return Ok(cached.clone())
    }

    // User was not cached.
    // We must go find them.
    Ok(ctx.http().get_user(id.into()).await?)
}