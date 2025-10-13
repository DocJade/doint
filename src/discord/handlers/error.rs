// Sometimes, errors happen

// Printing error information out to users is done on a best effort basis, if we fail to inform them, too bad.

use std::any::Any;
use std::process::exit;
use std::time::Duration;

use crate::types::serenity_types::CommandCheckFailureReason;
use crate::types::serenity_types::{Data, DointBotError, Error};
use log::error;
use log::warn;
use poise::Framework;
use poise::serenity_prelude::{FullEvent, Message, Permissions, Ready};
use poise::structs::Context;
use tokio::sync::Mutex;

/// When something goes wrong.
pub async fn handle_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup {
            error,
            framework,
            data_about_bot,
            ctx,
            ..
        } => {
            handle_setup_error(&error, framework, data_about_bot, ctx);
        }
        poise::FrameworkError::EventHandler {
            error,
            ctx,
            event,
            framework,
            ..
        } => {
            handle_event_handler_error(&error, ctx, event, framework);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            handle_command_error(&error, ctx);
        }
        poise::FrameworkError::SubcommandRequired { ctx } => {
            handle_subcommand_required_error(ctx);
        }
        poise::FrameworkError::CommandPanic { payload, ctx, .. } => {
            handle_command_panic_error(payload, ctx);
        }
        poise::FrameworkError::ArgumentParse { ctx, .. } => {
            // Running the command failed, we don't really care that deeply, just tell user that something went wrong.
            handle_argument_parse_error(ctx).await;
        }
        poise::FrameworkError::CommandStructureMismatch { .. } => {
            // Command is probably out of date. We can't really do anything due to a lack of proper context., so do nothing!
            // This results in an `Unknown interaction` but whatever. This shouldn't happen anyways, user might just need to
            // restart discord.
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            handle_cooldown_hit_error(remaining_cooldown, ctx).await;
        }
        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            handle_missing_bot_permissions_error(missing_permissions, ctx).await;
        }
        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            handle_missing_user_permissions_error(missing_permissions, ctx).await;
        }
        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            handle_not_an_owner_error(ctx).await;
        }
        poise::FrameworkError::GuildOnly { ctx, .. } => {
            handle_guild_only_error(ctx).await;
        }
        poise::FrameworkError::DmOnly { ctx, .. } => {
            handle_dm_only_error(ctx).await;
        }
        poise::FrameworkError::NsfwOnly { ctx, .. } => {
            handle_nsfw_only_error(ctx).await;
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            handle_command_check_failed_error(
                error.expect("Failed command checks always have an error."),
                ctx,
            )
            .await;
        }
        poise::FrameworkError::DynamicPrefix {
            error, ctx, msg, ..
        } => {
            handle_dynamic_prefix_error(error, ctx, msg).await;
        }
        poise::FrameworkError::UnknownCommand {
            ctx,
            msg,
            prefix,
            msg_content,
            framework,
            invocation_data,
            trigger,
            ..
        } => {
            handle_unknown_command_error(
                ctx,
                msg,
                prefix,
                msg_content,
                framework,
                invocation_data,
                trigger,
            )
            .await;
        }
        poise::FrameworkError::UnknownInteraction {
            ctx,
            framework,
            interaction,
            ..
        } => {
            handle_unknown_interaction_error(ctx, framework, interaction);
        }
    }
}

//
//
// ============
// Yes, we have a function for every single one of those error modes.
// got a better idea? Talk is cheap! Submit pull requests!
// ============
//
//

/// Handles the `Setup` variant of framework errors.
///
/// Returns nothing.
fn handle_setup_error(
    error: &DointBotError,
    _framework: &Framework<Data, DointBotError>,
    _data_about_bot: &Ready,
    _ctx: &poise::serenity_prelude::Context,
) {
    // This error was thrown during bot startup. We cannot recover from this cleanly, we'll shut down.
    error!(
        "Creating user data on bot startup failed! \n\n{error}\n\nWe cannot recover from this state! Shutting down."
    );
    exit(-1);
}

/// Handles the `EventHandler` variant of framework errors.
///
/// Returns nothing.
fn handle_event_handler_error(
    error: &DointBotError,
    _ctx: &poise::serenity_prelude::client::Context,
    _event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, DointBotError>,
) {
    // todo
    todo!("Write this error handler! {error} [event handler error]")
}

/// Handles the `Command` variant of framework errors.
///
/// Returns nothing.
fn handle_command_error(error: &DointBotError, _ctx: Context<'_, Data, DointBotError>) {
    // todo
    todo!("Write this error handler! {error} [command error]")
}

/// Handles the `SubcommandRequired` variant of framework errors.
///
/// Returns nothing.
fn handle_subcommand_required_error(_ctx: Context<'_, Data, DointBotError>) {
    // todo
    todo!("Write this error handler! [subcommand]")
}

/// Handles the `CommandPanic` variant of framework errors.
///
/// Returns nothing.
fn handle_command_panic_error(_payload: Option<String>, _ctx: Context<'_, Data, DointBotError>) {
    // todo
    todo!("Write this error handler! [panic]")
}

/// Handles the `ArgumentParse` variant of framework errors.
///
/// Returns nothing.
async fn handle_argument_parse_error(ctx: Context<'_, Data, DointBotError>) {
    // Since this was just a parse error, there isn't much we can do, since we dont plan on re-parsing their
    // command.
    warn!("Failed to parse a command!");
    // defer if possible, not a huge deal if not.
    let _ = ctx.defer_ephemeral().await;
    // Best attempt at informing user, dont really care if this message sends.
    let _ = ctx.reply("Hey uh, this is awkward, but running your command failed. Please try again, and tell Doc if this keeps happening. [Parse]").await;
}

/// Handles the `CooldownHit` variant of framework errors.
///
/// Returns nothing.
async fn handle_cooldown_hit_error(
    remaining_cooldown: Duration,
    ctx: Context<'_, Data, DointBotError>,
) {
    // User is on cooldown for this command. Let them know.
    // TODO: Use a relative unix timestamp to how user when they can do this again. (https://r.3v.fi/discord-timestamps/)
    let seconds_remaining = remaining_cooldown.as_secs();
    #[allow(clippy::cast_possible_wrap)] // users wont have to wait >100 years for a command.
    let end_unix_time = ctx.created_at().timestamp() + seconds_remaining as i64;
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx
        .say(format!(
            "You're on cooldown! You can run this command <t:{end_unix_time}:R>."
        ))
        .await;
}

/// Handles the `MissingBotPermissions` variant of framework errors.
///
/// Returns nothing.
async fn handle_missing_bot_permissions_error(
    missing_permissions: Permissions,
    ctx: Context<'_, Data, DointBotError>,
) {
    // The bot is lacking the permissions it needs to run this slash command.
    // Not aware of any use of setting required permissions on commands, so template message.
    let _ = ctx
        .say("Bot doesn't have perms for that command? This shouldn't happen, tell Doc!")
        .await;
    // TODO: Admin warning
    error!("Bot missing permissions on command invoke! {missing_permissions}");
}

/// Handles the `MissingUserPermissions` variant of framework errors.
///
/// Returns nothing.
async fn handle_missing_user_permissions_error(
    missing_permissions: Option<Permissions>,
    ctx: Context<'_, Data, DointBotError>,
) {
    // The user was required to have some permissions when running this command, but didn't.
    // Not used at the moment, so generic handling.
    error!("User tried to run a command they don't have permissions for! {missing_permissions:#?}");
    let _ = ctx
        .say("You're not allowed to do that! TELL DOC IMMEDIATELY!")
        .await;
}

/// Handles the `NotAnOwner` variant of framework errors.
///
/// Returns nothing.
async fn handle_not_an_owner_error(ctx: Context<'_, Data, DointBotError>) {
    // We shouldn't have owner only commands, but let's cover our bases.
    // TODO: admin warning
    warn!("Non-owner tried to run an owner command! Selfbot?");
    let _ = ctx.say("Bro is NOT the owner of doccord 💀").await;
}

/// Handles the `GuildOnly` variant of framework errors.
///
/// Returns nothing.
async fn handle_guild_only_error(ctx: Context<'_, Data, DointBotError>) {
    // Most commands can only be ran in doccord.
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx.say("This command can only be ran in Doccord.").await;
}

/// Handles the `DmOnly` variant of framework errors.
///
/// Returns nothing.
async fn handle_dm_only_error(ctx: Context<'_, Data, DointBotError>) {
    // As of right now, we don't have any DM only commands, but we'll handle it.
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx.say("This command can only be ran in DMs.").await;
}

/// Handles the `NsfwOnly` variant of framework errors.
///
/// Returns nothing.
async fn handle_nsfw_only_error(ctx: Context<'_, Data, DointBotError>) {
    // what
    error!("Why is there a NSFW command?");
    let _ = ctx.say("What freaky ass command are you trying to run? Why does the doint bot have freaky commands???").await;
}

/// Handles the `CommandCheckFailed` variant of framework errors.
///
/// Returns nothing.
async fn handle_command_check_failed_error(
    error: DointBotError,
    ctx: Context<'_, Data, DointBotError>,
) {
    // Pull the inner error out.
    let DointBotError::CommandCheckFailed(inner) = error else {
        unreachable!("Command check failures ALWAYS return an error of CommandCheckFailed type.")
    };

    actually_handle_command_check_failure(ctx, inner).await;
}

/// Handles the `DynamicPrefix` variant of framework errors.
///
/// Returns nothing.
async fn handle_dynamic_prefix_error(
    error: DointBotError,
    ctx: poise::PartialContext<'_, Data, DointBotError>,
    msg: &Message,
) {
    // Not aware of us ever using dynamic prefixes.
    error!("Dynamic prefix called! We don't use those! {error}");
    let _ = msg
        .reply(
            ctx.serenity_context,
            "Dynamic prefix? What??? Tell Doc IMMEDIATELY!",
        )
        .await;
}

/// Handles the `UnknownCommand` variant of framework errors.
///
/// Returns nothing.
async fn handle_unknown_command_error(
    ctx: &poise::serenity_prelude::Context,
    msg: &Message,
    prefix: &str,
    msg_content: &str,
    _framework: poise::FrameworkContext<'_, Data, DointBotError>,
    _invocation_data: &Mutex<Box<dyn Any + Send + Sync>>,
    _trigger: poise::MessageDispatchTrigger,
) {
    // User used a prefix (we dont do that) and tried to call a command that doesn't exit.
    error!("Prefix command was not known, but we dont use prefixes! {prefix} | {msg_content}");
    let _ = msg
        .reply(
            ctx,
            "??? Why are prefix commands enabled? Tell Doc IMMEDIATELY!",
        )
        .await;
}

/// Handles the `UnknownInteraction` variant of framework errors.
///
/// Returns nothing.
fn handle_unknown_interaction_error(
    _ctx: &poise::serenity_prelude::Context,
    _framework: poise::FrameworkContext<'_, Data, DointBotError>,
    interaction: &poise::serenity_prelude::CommandInteraction,
) {
    // User did an interaction on an unknown name... Not sure what that means.
    // Probably a command that used to exist, and doesn't anymore.

    // It makes sense for the bot to not respond at all in this case, and since the ctx here is wacky, we're fine with that.
    warn!("User tried to run an unknown interaction! {interaction:#?}");
}

/*
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

*/

//
//
// ============
// Handle when command checks fail
// ============
//
//

/// Inform users, or do other things when a check for a command fails.
async fn actually_handle_command_check_failure(
    ctx: Context<'_, Data, DointBotError>,
    failure: CommandCheckFailureReason,
) {
    match failure {
        CommandCheckFailureReason::UserNotEnrolled => {
            let _ = ctx.defer_ephemeral().await;
            let _ = ctx
                .say("You aren't enrolled in Doints! Run `/opt_in` to join!")
                .await;
        }
        CommandCheckFailureReason::CheckErroredOut(command_check_failure) => {
            // Checking itself failed.
            // TODO: Actually do something with this.
            let _ = ctx.defer_ephemeral().await;
            let _ = ctx.say("Hey uh, this is awkward, but running your command failed. Please try again, and tell Doc if this keeps happening. [CheckErroredOut]").await;
            error!("Command check errored out! {command_check_failure:#?}");
        }
        CommandCheckFailureReason::InvalidChannel => {
            // Our channel guard in guards sends an error message itself, so do nothing!
            // let _ = ctx.defer_ephemeral().await;
            // let _ = ctx
            //     .say("You can't run that in here! Go run this command in the appropriate channel.")
            //     .await;
        }
        CommandCheckFailureReason::MemberNotFound => {
            let _ = ctx.defer_ephemeral().await;
            let _ = ctx.say("Hey uh, this is awkward, but running your command failed. Please try again, and tell Doc if this keeps happening. [MemberNotFound]").await;
        }
        CommandCheckFailureReason::R2D2Failure(err) => {
            // TODO: Tell admins
            warn!("R2D2 FAILURE! : {err}");
            let _ = ctx.defer_ephemeral().await;
            let _ = ctx.say("Hey uh, this is awkward, but running your command failed. Please try again, and tell Doc if this keeps happening. [R2D2]").await;
        }
        CommandCheckFailureReason::UserInJail(jailed_user) => {
            let _ = ctx.defer_ephemeral().await;
            let _ = ctx
                .say(format!(
                    "You're in jail! You'l be released <t:{}:R>.",
                    jailed_user.until.and_utc().timestamp()
                ))
                .await;
        }
    }
}
