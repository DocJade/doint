use crate::{models::queries, prelude::*};

impl queries::Member {
    /// # Errors
    /// Returns `Err` when the bot is outside the server, or if the user doesn't exist.
    ///
    /// Retrives a [`GuildMember`] from the id
    pub async fn from_id(
        ctx: PoiseContext<'_>,
        user_id: u64,
    ) -> Result<Option<GuildMember>, BotError> {
        // Get the guild from the cache, or regularly if we can't
        let guild = if let Some(guild) = ctx.cache().guild(DOCCORD_SERVER_ID) {
            guild.clone()
        } else if let Some(ok) = ctx.guild() {
            ok.clone()
        } else {
            // This should not happen since we check the guild at runtime, still handle it just in case
            return Err(BotError::OutsideServer)?;
        };

        match guild.member(ctx, user_id).await {
            Ok(member) => Ok(Some(member.into_owned())),
            Err(e) => Err(BotError::from(e)),
        }
    }
}

impl queries::Member {
    /// # Errors
    /// Returns `Err` if the query fails
    ///
    /// Get the display name of a [`User`]
    pub async fn get_display_name(ctx: PoiseContext<'_>, id: u64) -> Result<String, BotError> {
        let user = Users::get_user_from_id(ctx, id).await?;

        Ok(user.display_name().into())
    }
}
