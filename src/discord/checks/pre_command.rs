// These checks run _before_ every command. This can print information to users if they are ineligible to do things

use log::{debug, info};

use crate::{
    database::queries::get_user::get_doint_user,
    discord::checks::consented::member_enrolled_in_doints,
    types::serenity_types::{CommandCheckFailure, Context, DointBotError, Error},
};

use crate::types::serenity_types::CommandCheckFailureReason::*;

/// Runs before every command.
///
/// Returns false if the user cannot run a command.
pub(crate) async fn pre_command_call(ctx: Context<'_>) -> Result<bool, Error> {
    // The only way we can get info out of here besides a generic "check failed" boolean is to return an error.
    // To this end, `DointBotError::CommandCheckFailed(_)` exists. If a check fails, return that. NOT false.

    // Skip everything if user is opting in.
    // TODO: Put this after channel checks
    if ctx.invoked_command_name() == "opt_in" {
        debug!("Opt-in command, skipping pre-command checks...");
        return Ok(true);
    }

    // Get the user that called the command
    let member = if let Some(member) = ctx.author_member().await {
        member
    } else {
        // Couldnt find user.
        // If we cant load them, chances are we arent in doccord.
        // We just wont respond.
        debug!("Pre-command check, couldn't find member.");
        return Err(Error::CommandCheckFailed(MemberNotFound));
    };

    // If the user is not enrolled in doints, let them know.
    let is_enrolled = match member_enrolled_in_doints(member.clone().into_owned(), ctx).await {
        Ok(ok) => ok,
        Err(err) => {
            // Couldn't check if user was enrolled. Not much we can do.
            // Still want that inner error tho
            return Err(Error::CommandCheckFailed(CheckErroredOut(
                CommandCheckFailure {
                    bot_error: Box::new(err),
                    where_fail: "Member enrollment check.".to_string(),
                },
            )));
        }
    };

    // We need to also check if the user is trying to opt in, if they are, we cant cancel the command.

    if !is_enrolled {
        // User is not enrolled in doints.
        return Err(Error::CommandCheckFailed(UserNotEnrolled));
    }

    // If the user is an admin, we dont need to do any more checks.
    if let Some(perms) = member.permissions {
        if perms.administrator() {
            // User is an admin
            info!("Skipping pre_command checks, this user is an administrator.");
            return Ok(true);
        }
    }

    // User is enrolled, get the actual DB entry to do more checks
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = match pool.get() {
        Ok(ok) => ok,
        Err(err) => {
            // Failed to get DB connection, nothing we can do. Fail out.
            return Err(DointBotError::CommandCheckFailed(R2D2Failure(
                err.to_string(),
            )));
        }
    };

    // Get the user
    let user = match get_doint_user(member.user.id.get(), &mut conn) {
        Ok(ok) => {
            // They should be there, otherwise we need to bail.
            if let Some(all_good) = ok {
                all_good
            } else {
                // Well, didnt find them
                return Err(Error::CommandCheckFailed(UserNotEnrolled));
            }
        }
        Err(err) => {
            // Failed to load them in, cant go further.
            return Err(Error::CommandCheckFailed(CheckErroredOut(
                CommandCheckFailure {
                    bot_error: Box::new(err.into()),
                    where_fail: "Getting the Doint user.".to_string(),
                },
            )));
        }
    };

    // Check if the user is in jail
    match user.is_jailed(&mut conn) {
        Ok(ok) => {
            if let Some(jail) = ok {
                // Cant run commands while in jail.
                return Err(Error::CommandCheckFailed(UserInJail(jail)));
            }
        }
        Err(err) => {
            match err {
                crate::jail::error::JailError::AlreadyInJail(_jailed_user) => {
                    unreachable!("We aren't putting the user in jail here.")
                }
                crate::jail::error::JailError::UserNotInJail => {
                    unreachable!("We aren't freeing the user from jail.")
                }
                crate::jail::error::JailError::StillServingSentence => {
                    unreachable!("We aren't freeing the user from jail.")
                }
                crate::jail::error::JailError::DieselError(error) => {
                    // Checking if the user was in jail failed.
                    return Err(Error::CommandCheckFailed(CheckErroredOut(
                        CommandCheckFailure {
                            bot_error: Box::new(error.into()),
                            where_fail: "Failed to check if user was in jail.".to_string(),
                        },
                    )));
                }
            }
        }
    }

    // User is not in jail.

    // All checks good!
    debug!("All checks pass, user can run command.");
    Ok(true)
}
