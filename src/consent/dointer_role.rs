// After user has signed the blood oath, give them the doint role.

use log::warn;

use crate::{
    discord::helper::get_member::get_member_from_id, knob::roles::DOINTS_ENABLED_ROLE_ID,
    types::serenity_types::Context,
};

/// Gives a user the dointer role. Should only be used when users have just been added to the DB.
///
/// returns false if they didnt end up with the role.
pub async fn give_dointer_role(ctx: Context<'_>, user_id: u64) -> bool {
    // Get the member
    let member = match get_member_from_id(ctx, user_id).await {
        Ok(ok) => {
            // If they didnt exist, we cant do anything.
            if let Some(good) = ok {
                good
            } else {
                // They did not exist.
                return false;
            }
        }
        Err(_) => {
            // Member didnt exist, or something else brokie.
            return false;
        }
    };

    // Now give them the role.
    match member.add_role(ctx, DOINTS_ENABLED_ROLE_ID).await {
        Ok(()) => {
            // Worked!
            true
        }
        Err(err) => {
            // Adding it failed for some reason.
            warn!("Tried to give member the dointer role, but they refused! {err:#?}");
            false
        }
    }
}

/// Revoke the dointer role from a user.
///
/// Returns true if the user no-longer has the role.
pub async fn revoke_dointer_role(ctx: Context<'_>, user_id: u64) -> bool {
    // Get the member
    let member = match get_member_from_id(ctx, user_id).await {
        Ok(ok) => {
            // If they didnt exist, we cant do anything.
            if let Some(good) = ok {
                good
            } else {
                // They did not exist.
                return false;
            }
        }
        Err(_) => {
            // Member didnt exist, or something else brokie.
            return false;
        }
    };

    // Now give them the role.
    match member.remove_role(ctx, DOINTS_ENABLED_ROLE_ID).await {
        Ok(()) => {
            // Worked!
            true
        }
        Err(err) => {
            warn!("Failed to remove dointer role from user! {err:#?}");
            false
        }
    }
}
