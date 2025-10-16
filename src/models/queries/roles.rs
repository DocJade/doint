use crate::{models::queries, prelude::*};

impl queries::Roles {
    /// Check if a [`GuildMember`] has a given role
    #[inline]
    #[must_use]
    pub fn member_has_role(member: &GuildMember, role_id: u64) -> bool {
        member.roles.iter().any(|r| r.get() == role_id)
    }

    /// Check if a [`GuildMember`] is enrolled in doints
    ///
    /// This is an alias of [`Roles::member_has_role`] with `DOINTS_ENABLED_ROLE_ID`
    #[inline]
    #[must_use]
    pub fn member_enrolled_in_doints(member: &GuildMember) -> bool {
        Self::member_has_role(member, DOINTS_ENABLED_ROLE_ID)
    }
}

impl queries::Roles {
    /// # Errors
    /// Returns `Err` if any query fails
    ///
    /// Give a given `UserId` a role
    pub async fn give_role(
        ctx: PoiseContext<'_>,
        user_id: u64,
        role_id: u64,
    ) -> Result<bool, BotError> {
        let member = match queries::Member::from_id(ctx, user_id).await? {
            Some(member) => member,
            None => return Ok(false),
        };

        member.add_role(&ctx.http(), role_id).await?;

        Ok(true)
    }

    /// # Errors
    /// Returns `Err` if the transaction fails
    ///
    /// This is an alias of [`Roles::give_role`] with `DOINTS_ENABLED_ROLE_ID`
    #[inline]
    pub async fn give_doints_role(ctx: PoiseContext<'_>, user_id: u64) -> Result<bool, BotError> {
        Self::give_role(ctx, user_id, DOINTS_ENABLED_ROLE_ID).await
    }

    /// # Errors
    /// Returns `Err` if any query fails
    ///
    /// Revoke a `role_id` from a given `UserId`
    pub async fn revoke_role(
        ctx: PoiseContext<'_>,
        user_id: u64,
        role_id: u64,
    ) -> Result<bool, BotError> {
        let member = match queries::Member::from_id(ctx, user_id).await? {
            Some(member) => member,
            None => return Ok(false),
        };

        member.remove_role(&ctx.http(), role_id).await?;

        Ok(true)
    }

    /// # Errors
    /// Returns `Err` if the transaction fails
    ///
    /// This is an alias of [`Roles::revoke_role`] with `DOINTS_ENABLED_ROLE_ID`
    #[inline]
    pub async fn revoke_doints_role(ctx: PoiseContext<'_>, user_id: u64) -> Result<bool, BotError> {
        Self::revoke_role(ctx, user_id, DOINTS_ENABLED_ROLE_ID).await
    }
}
