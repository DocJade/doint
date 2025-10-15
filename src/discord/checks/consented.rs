// Is the user enrolled in doints?

use poise::serenity_prelude::{Member, RoleId};

use crate::prelude::*;

// Inner function that checks a member, not context
pub fn member_enrolled_in_doints(member: Member, ctx: Context<'_>) -> Result<bool, Error> {
    let Some(roles) = member.roles(ctx) else {
        // Cant get roles, user has none or something failed.
        return Ok(false);
    };

    // Do they have the dointer role?
    let has = roles.iter().any(|role| role.id == DOINTS_ENABLED_ROLE_ID);
    Ok(has)
}

#[must_use]
pub fn role_ids_contains_dointer_role(roles: &Vec<RoleId>) -> bool {
    roles.iter().any(|role| role.get() == DOINTS_ENABLED_ROLE_ID)
}