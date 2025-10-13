// Gets a member from a user ID.
// Assumes we are in doccord.

use crate::prelude::*;
use log::warn;
use poise::serenity_prelude::{self as serenity, UserId};

/// Get the Member that this ID refers to, if they exist.
///
/// Incoming context must come from a guild.
///
/// Tries reading from cache first.
pub async fn get_member_from_id(
    ctx: Context<'_>,
    user_id: u64,
) -> Result<Option<serenity::Member>, Error> {
    // Get the cached guild if possible.
    let guild = if let Some(guild) = ctx.cache().guild(DOCCORD_SERVER_ID) {
        guild.clone()
    } else if let Some(ok) = ctx.guild() {
        // Not cached, get it normally.
        ok.clone()
    } else {
        // This was called outside of the guild, we dont care.
        warn!("Tried to get a member from an ID while ctx was outside guild!");
        warn!("Bot should not respond in DMs or in non-doccord!");
        // Ignored.
        return Err(Error::ThisShouldNotHappen(ThisShouldNotHappen::BotIsOutsideServer))?;
    };

    // Does the guild have this member?
    // `after` here can be used to filter for a person.
    // Not sure if the `after` is exclusive, so workarounds.
    let maybe = guild
        .members(ctx, Some(2), Some(UserId::from(user_id - 1)))
        .await?;

    // We should always get a result (unless this person happened to have a REALLY high user ID)
    // doing .find on empty is fine.

    // Check if the user is in there. Either is, or isnt. Dur.
    let found = maybe.iter().find(|member| member.user.id == user_id);
    Ok(found.cloned())
}
