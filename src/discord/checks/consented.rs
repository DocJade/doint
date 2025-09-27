// Is the user enrolled in doints?

use poise::serenity_prelude::Member;

use crate::{
    database::{queries::top_n::get_top_n, tables::users::DointUser},
    discord::helper::get_nick::get_display_name,
    knob::roles::DOINTS_ENABLED_ROLE_ID,
    types::serenity_types::{Context, Data, Error},
};

/// Check if the caller has the dointer role.
pub(crate) async fn ctx_member_enrolled_in_doints(ctx: Context<'_>) -> Result<bool, Error> {
    let member = if let Some(member) = ctx.author_member().await {
        member
    } else {
        // Couldnt find user.
        // If we cant load them, chances are we arent in doccord.
        return Ok(false);
    };
    member_enrolled_in_doints(member.into_owned(), ctx).await
}

// Inner function that checks a member, not context
pub(crate) async fn member_enrolled_in_doints(
    member: Member,
    ctx: Context<'_>,
) -> Result<bool, Error> {
    let roles = if let Some(roles) = member.roles(ctx) {
        roles
    } else {
        // Cant get roles, user has none or something failed.
        return Ok(false);
    };

    // Do they have the dointer role?
    let has = roles.iter().any(|role| role.id == DOINTS_ENABLED_ROLE_ID);
    Ok(has)
}
