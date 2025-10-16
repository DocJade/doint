use diesel::{r2d2, result::Error as DieselError};
use log::{error, warn};
use poise::{
    FrameworkContext, FrameworkError,
    serenity_prelude::{
        self, Error as SerenityError, FullEvent, Permissions, client::Context as SerenityContext,
    },
    structs::Context,
};
use std::{
    fmt,
    process::{self},
    time::Duration,
};
use thiserror::Error;

use crate::prelude::*;

/// Defines how serious a bot error is.
#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Info,
    Critical,
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Info => "info",
            Self::Critical => "CRITICAL",
            Self::Fatal => "FATAL",
        };
        f.write_str(label)
    }
}

/// All possible bot error variants.
#[derive(Error, Debug)]
pub enum BotError {
    #[error("[{severity}] R2D2 error: {source}")]
    R2D2 {
        severity: ErrorSeverity,
        #[source]
        source: r2d2::Error,
    },

    #[error("[{severity}] Diesel error: {source}")]
    Diesel {
        severity: ErrorSeverity,
        #[source]
        source: DieselError,
    },

    #[error("[{severity}] Diesel pool error: {source}")]
    DieselPool {
        severity: ErrorSeverity,
        #[source]
        source: r2d2::PoolError,
    },

    #[error("[{severity}] Serenity error: {source}")]
    Serenity {
        severity: ErrorSeverity,
        #[source]
        source: SerenityError,
    },

    #[error("[{severity}] Guard error: {source}")]
    Guard {
        severity: ErrorSeverity,
        #[source]
        source: GuardError,
    },

    #[error("[CRITICAL] Bot is outside of server")]
    OutsideServer,

    #[error(
        "[info]
    Failed to cast a float as a BigDecimal"
    )]
    BigDecimalCast,

    #[error("[{severity}] Doint transfer construction failed: {source}")]
    DointTransferConstruction {
        severity: ErrorSeverity,
        #[source]
        source: DointTransferConstructionError,
    },

    #[error("[{severity}] Doint transfer failed: {source}")]
    DointTransfer {
        severity: ErrorSeverity,
        #[source]
        source: DointTransferError,
    },

    #[error("[{severity}] Jail error: {source}")]
    Jail {
        severity: ErrorSeverity,
        #[source]
        source: JailError,
    },
}

impl BotError {
    /// # Errors
    /// Will return an `Err` if the `BotError` cannot be constructed with a severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Result<Self, String> {
        match &mut self {
            Self::R2D2 { severity: s, .. }
            | Self::Diesel { severity: s, .. }
            | Self::DieselPool { severity: s, .. }
            | Self::Serenity { severity: s, .. }
            | Self::DointTransferConstruction { severity: s, .. }
            | Self::DointTransfer { severity: s, .. }
            | Self::Jail { severity: s, .. }
            | Self::Guard { severity: s, .. } => *s = severity,
            _ => return Err(format!("Cannot construct {self:?} with a severity")),
        }
        Ok(self)
    }
}

impl BotError {
    pub fn get_severity(&self) -> Option<ErrorSeverity> {
        match self {
            Self::R2D2 { severity, .. }
            | Self::Diesel { severity, .. }
            | Self::DieselPool { severity, .. }
            | Self::Serenity { severity, .. }
            | Self::DointTransferConstruction { severity, .. }
            | Self::DointTransfer { severity, .. }
            | Self::Jail { severity, .. }
            | Self::Guard { severity, .. } => Some(*severity),
            _ => None,
        }
    }
}

impl From<r2d2::Error> for BotError {
    fn from(err: r2d2::Error) -> Self {
        Self::R2D2 {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<DieselError> for BotError {
    fn from(err: DieselError) -> Self {
        Self::Diesel {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<r2d2::PoolError> for BotError {
    fn from(err: r2d2::PoolError) -> Self {
        Self::DieselPool {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<SerenityError> for BotError {
    fn from(err: SerenityError) -> Self {
        Self::Serenity {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<GuardError> for BotError {
    fn from(err: GuardError) -> Self {
        Self::Guard {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<DointTransferConstructionError> for BotError {
    fn from(err: DointTransferConstructionError) -> Self {
        Self::DointTransferConstruction {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<DointTransferError> for BotError {
    fn from(err: DointTransferError) -> Self {
        Self::DointTransfer {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl From<JailError> for BotError {
    fn from(err: JailError) -> Self {
        Self::Jail {
            severity: ErrorSeverity::Info,
            source: err,
        }
    }
}

impl BotError {
    #[must_use] pub fn r2d2(err: r2d2::Error, severity: ErrorSeverity) -> Self {
        Self::R2D2 {
            severity,
            source: err,
        }
    }
    #[must_use] pub fn diesel(err: DieselError, severity: ErrorSeverity) -> Self {
        Self::Diesel {
            severity,
            source: err,
        }
    }
    pub fn serenity(err: SerenityError, severity: ErrorSeverity) -> Self {
        Self::Serenity {
            severity,
            source: err,
        }
    }
    #[must_use] pub fn guard(err: GuardError, severity: ErrorSeverity) -> Self {
        Self::Guard {
            severity,
            source: err,
        }
    }
    #[must_use] pub fn doint_transfer_construction(
        err: DointTransferConstructionError,
        severity: ErrorSeverity,
    ) -> Self {
        BotError::DointTransferConstruction {
            severity,
            source: err,
        }
    }
    #[must_use] pub fn doint_transfer(err: DointTransferError, severity: ErrorSeverity) -> Self {
        BotError::DointTransfer {
            severity,
            source: err,
        }
    }
    #[must_use] pub fn jail(err: JailError, severity: ErrorSeverity) -> Self {
        BotError::Jail {
            severity,
            source: err,
        }
    }
}

/// Handles errors that occur during bot runtime.
pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn handle_poise(error: FrameworkError<'_, Data, BotError>) {
        match error {
            FrameworkError::Setup { error, .. } => {
                Self::handle_setup_error(&error);
            }

            FrameworkError::EventHandler {
                error,
                ctx,
                event,
                framework,
                ..
            } => {
                Self::handle_event_handler_error(&error, ctx, event, framework).await;
            }

            FrameworkError::Command { error, ctx, .. } => {
                Self::handle_command_error(&error, ctx).await;
            }

            FrameworkError::CommandPanic { payload, ctx, .. } => {
                Self::handle_command_panic_error(payload, ctx).await;
            }

            FrameworkError::ArgumentParse { ctx, .. } => {
                Self::handle_argument_parse_error(ctx).await;
            }

            FrameworkError::CooldownHit {
                remaining_cooldown,
                ctx,
                ..
            } => {
                Self::handle_cooldown_hit_error(remaining_cooldown, ctx).await;
            }

            FrameworkError::MissingBotPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                Self::handle_missing_bot_permissions_error(missing_permissions, ctx).await;
            }

            FrameworkError::MissingUserPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                Self::handle_missing_user_permissions_error(missing_permissions, ctx).await;
            }

            FrameworkError::NotAnOwner { ctx, .. } => {
                Self::handle_not_an_owner_error(ctx).await;
            }

            FrameworkError::GuildOnly { ctx, .. } => {
                Self::handle_guild_only_error(ctx).await;
            }

            FrameworkError::DmOnly { ctx, .. } => {
                Self::handle_dm_only_error(ctx).await;
            }

            FrameworkError::NsfwOnly { ctx, .. } => {
                Self::handle_nsfw_only_error(ctx).await;
            }

            FrameworkError::UnknownInteraction {
                ctx,
                framework,
                interaction,
                ..
            } => {
                Self::handle_unknown_interaction_error(ctx, framework, interaction);
            }

            _ => warn!("Unhandled Poise error variant."),
        }
    }

    fn handle_setup_error(error: &BotError) {
        error!("Bot startup failed! {error}");
        if matches!(
            error,
            BotError::Diesel {
                severity: ErrorSeverity::Fatal,
                ..
            }
        ) {
            error!("Fatal database error during startup! Shutting down.");
            process::exit(1);
        } else {
            process::exit(1);
        }
    }

    async fn handle_event_handler_error(
        error: &BotError,
        _ctx: &SerenityContext,
        _event: &FullEvent,
        _framework: FrameworkContext<'_, Data, BotError>,
    ) {
        error!("Event handler error: {error}");
        if let Some(ErrorSeverity::Fatal) = error.get_severity() {
            error!("Fatal event error! Exiting.");
            process::exit(1);
        }
    }

    async fn handle_command_error(error: &BotError, ctx: Context<'_, Data, BotError>) {
        warn!("Command failed: {error}");
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx
            .say("Something went wrong executing your command. Please try again later.")
            .await;
    }

    async fn handle_command_panic_error(payload: Option<String>, ctx: Context<'_, Data, BotError>) {
        error!("Command panicked! Payload: {payload:?}");
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx
            .say("Unexpected internal error while running your command. (panic)")
            .await;
    }

    async fn handle_argument_parse_error(ctx: Context<'_, Data, BotError>) {
        warn!("Argument parse error in command.");
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx
            .say("Could not parse your command arguments. Please check your input.")
            .await;
    }

    async fn handle_cooldown_hit_error(remaining: Duration, ctx: Context<'_, Data, BotError>) {
        let seconds = remaining.as_secs();
        let end_unix = ctx.created_at().timestamp() + seconds as i64;
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx
            .say(format!("You're on cooldown! Try again <t:{end_unix}:R>."))
            .await;
    }

    async fn handle_missing_bot_permissions_error(
        missing: Permissions,
        ctx: Context<'_, Data, BotError>,
    ) {
        error!("Bot missing permissions: {missing:?}");
        let _ = ctx
            .say("The bot doesn't have the required permissions for that command.")
            .await;
    }

    async fn handle_missing_user_permissions_error(
        missing: Option<Permissions>,
        ctx: Context<'_, Data, BotError>,
    ) {
        warn!("User missing permissions: {missing:?}");
        let _ = ctx
            .say("You don't have permission to use that command.")
            .await;
    }

    async fn handle_not_an_owner_error(ctx: Context<'_, Data, BotError>) {
        warn!("Non-owner attempted to use an owner-only command.");
        let _ = ctx.say("Only the bot owner can run this command.").await;
    }

    async fn handle_guild_only_error(ctx: Context<'_, Data, BotError>) {
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx
            .say("This command can only be used in a server, not in DMs.")
            .await;
    }

    async fn handle_dm_only_error(ctx: Context<'_, Data, BotError>) {
        let _ = ctx.defer_ephemeral().await;
        let _ = ctx.say("This command can only be used in DMs.").await;
    }

    async fn handle_nsfw_only_error(ctx: Context<'_, Data, BotError>) {
        error!("Attempt to use an NSFW command in a non-NSFW channel.");
        let _ = ctx
            .say("This command is NSFW-only and cannot be used here.")
            .await;
    }

    fn handle_unknown_interaction_error(
        _ctx: &SerenityContext,
        _framework: FrameworkContext<'_, Data, BotError>,
        interaction: &serenity_prelude::CommandInteraction,
    ) {
        warn!("Unknown interaction received: {interaction:#?}");
    }
}
