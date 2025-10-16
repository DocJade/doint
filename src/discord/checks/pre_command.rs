// These checks run _before_ every command. This can print information to users if they are ineligible to do things

use log::{debug, info};

use crate::prelude::*;

/// Runs before every command.
///
/// Returns false if the user cannot run a command.
pub async fn pre_command_call(ctx: PoiseContext<'_>) -> Result<bool, BotError> {
    // Skip everything if user is opting in.
    // TODO: Put this after channel checks
    if ctx.invoked_command_name() == "opt_in" {
        debug!("Opt-in command, skipping pre-command checks...");
        return Ok(true);
    }

    // Get the user that called the command
    let Some(member) = ctx.author_member().await else {
        // Couldnt find user.
        // If we cant load them, chances are we arent in doccord.
        // We just wont respond.
        debug!("Pre-command check, couldn't find member.");
        return Err(BotError::from(GuardError::MemberNotFound));
    };

    // If the user is not enrolled in doints, let them know.
    let is_enrolled = Roles::member_enrolled_in_doints(&member);

    // We need to also check if the user is trying to opt in, if they are, we cant cancel the command.
    if !is_enrolled {
        return Err(BotError::from(GuardError::UserNotEnrolled));
    }

    // If the user is an admin, we dont need to do any more checks.
    if let Some(perms) = member.permissions
        && perms.administrator()
    {
        // User is an admin
        info!("Skipping pre_command checks, this user is an administrator.");
        return Ok(true);
    }

    // User is enrolled, get the actual DB entry to do more checks
    let pool = ctx.data().db_pool.clone();
    let mut conn = pool.get()?;

    // Get the user
    let user: DointUser = match Users::get_doint_user(member.user.id.get(), &mut conn) {
        Ok(ok) => {
            // They should be there, otherwise we need to bail.
            let Some(user) = ok else {
                return Err(BotError::from(GuardError::UserNotEnrolled));
            };

            user
        }
        Err(err) => {
            return Err(BotError::from(err));
        }
    };

    // Check if the user is in jail
    match user.in_jail(&mut conn) {
        Ok(ok) => {
            if let Some(jailed_user) = ok {
                // Cant run commands while in jail.
                return Err(BotError::from(GuardError::UserInJail(jailed_user)));
            }
        }
        Err(err) => {
            match err {
                JailError::AlreadyInJail(_jailed_user) => {
                    unreachable!("We aren't putting the user in jail here.")
                }
                JailError::DieselError(error) => {
                    // Checking if the user was in jail failed.
                    return Err(BotError::from(error));
                }
                _ => unreachable!(),
            }
        }
    }

    // User is not in jail.

    // All checks good!
    debug!("All checks pass, user can run command.");
    Ok(true)
}
