// These checks run _before_ every command. This can print information to users if they are ineligible to do things

use chrono::Local;
use log::{debug, info};

use crate::{
    database::queries::get_user::get_doint_user,
    discord::{
        checks::consented::member_enrolled_in_doints
    },
    types::serenity_types::{
        Context,
        Data,
        Error
    }
};


/// Runs before every command.
/// 
/// Returns false if the user cannot run a command.
pub(crate) async fn pre_command_call(ctx: Context<'_>) -> bool {

    // Skip everything if user is opting in.
    if ctx.invoked_command_name() == "opt_in" {
        debug!("Opt-in command, skipping pre-command checks...");
        return true
    }


    // Get the user that called the command
    let member = if let Some(member) = ctx.author_member().await {
        member
    } else {
        // Couldnt find user.
        // If we cant load them, chances are we arent in doccord.
        // We just wont respond.
        debug!("Pre-command check, couldn't find member.");
        return false
    };
    
    // If the user is not enrolled in doints, let them know.
    let is_enrolled = match member_enrolled_in_doints(member.clone().into_owned(), ctx).await {
        Ok(ok) => ok,
        Err(_) => {
            // Couldn't check if user was enrolled. Not much we can do.
            // Hence, do nothing.
            debug!("Pre-command check, failed to run member_enrolled_in_doints.");
            return false
        },
    };

    // We need to also check if the user is trying to opt in, if they are, we cant cancel the command.
    
    if !is_enrolled {
        // User is not enrolled in doints. Let them know.
        // If this fails, oh well.
        let _ = ctx.say("You have not opted-in to the doint system.\nIf you wish to use doints, please run `/opt_in`.").await;
        // Cant go any further.
        return false
    }

    // If the user is an admin, we dont need to do any more checks.
    if let Some(perms) = member.permissions {
        if perms.administrator() {
            // User is an admin
            info!("Skipping pre_command checks, this user is an administrator.");
            return true;
        }
    }


    
    // User is enrolled, get the actual DB entry to do more checks
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = if let Ok(worked) = pool.get() {
        worked
    } else {
        // Failed to get DB connection, nothing we can do. Fail out.
        return false
    };

    // Get the user
    let user = if let Ok(found) = get_doint_user(member.user.id.get(), &mut conn) {
        // They should be there, otherwise we need to bail.
        if let Some(all_good) = found {
            all_good
        } else {
            // Well, didnt find them
            return false
        }
    } else {
        // Failed to load them in, cant go further.
        return false
    };
    
    // Check if the user is in jail
    if let Ok(check) = user.is_jailed(&mut conn) {
        if let Some(jail) = check {
            // User _is_ in jail. Tell them how much time they have left.
            let time_left_total_seconds = jail.until.signed_duration_since(Local::now().naive_utc()).num_seconds();

            // If this number is negative, that means they should have already been released from jail, but haven't yet.
            if time_left_total_seconds <= 0 {
                let _ = ctx.say("You're in jail! You'll be released soon.").await;
                // Still gotta wait for them to be free though.
                return false
            }

            let seconds = time_left_total_seconds % 60;
            let minutes = time_left_total_seconds / 60 % 60;
            let hours = time_left_total_seconds / 60 / 60;
            let seconds_string = format!("{seconds:02} seconds.");
            let minutes_string = if minutes > 0 {
                format!("{minutes:02} minutes, and ")
            } else {
                String::new()
            };

            let hours_string = if hours > 0 {
                format!("{hours:02} hours, ")
            } else {
                String::new()
            };

            // Put that all together
            let _ = ctx.say(format!("You're in jail! You'll be released in {hours_string}{minutes_string}{seconds_string}")).await;

            // Cant run the command while in jail.
            return false
        }
    } else {
        // Failed to check if they're in jail.
        return false
    }

    // User is not in jail.

    // All checks good!
    debug!("All checks pass, user can run command.");
    true
    
}