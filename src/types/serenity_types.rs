use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use thiserror::Error;

use crate::prelude::*;

// Error and context types

//
// The error type
// This needs to be able to hold outcomes for things like commands being denied due to
// cooldowns and such, but adding new fields to this error type is a LAST resort.
//

#[derive(Error, Debug)]
pub enum DointBotError {
    #[error("R2D2 pooling error.")]
    R2D2Error(#[from] r2d2::Error),

    #[error("Diesel pool error.")]
    DieselPoolError(#[from] diesel::r2d2::PoolError),

    #[error("Diesel database error.")]
    DieselError(#[from] diesel::result::Error),

    #[error("Discord/Serenity error.")]
    SerenityError(#[from] poise::serenity_prelude::Error),

    #[error(
        "Failed to cast a number. This should never happen, so if you see this, this IS a bug."
    )]
    #[deprecated]
    BigDecimalCastError,

    #[error("A bank transfer failed.")] // TODO: Phase this out, its varients need to be handled lower down.
    #[deprecated]
    BankTransferError(#[from] crate::models::bank::transfer::DointTransferError),

    #[error("A bank transfer failed to construct.")]
    BankTransferConstructionError(
        #[from] crate::models::bank::transfer::DointTransferConstructionError,
    ),

    #[error("Failed to jail a user.")] // TODO: Phase this out, its varients need to be handled lower down.
    #[deprecated]
    JailingError(#[from] crate::models::jail::JailError),

    #[error(
        "Some errors are highly unlikely, this should be handled elsewhere, but this is your way out."
    )]
    ThisShouldNotHappen(ThisShouldNotHappen),

    #[error("Command check failed.")]
    CommandCheckFailed(CommandCheckFailureReason),
}

// We also have a variant for situations that should not happen, but are not completely impossible.

/// These are error modes that should not happen in most cases, but we cannot prove they will never happen.
///
/// You may add varients to this enum as you'd like, but really consider, should you be handling this yourself instead
/// of making the bot's error handler deal with it?
#[derive(Debug)]
pub enum ThisShouldNotHappen {
    BotIsOutsideServer,
}

/// When a check fails and you need to pass in the `DointBotError` we need a bit more info.
#[derive(Debug)]
pub struct CommandCheckFailure {
    /// The error you got.
    pub bot_error: Box<DointBotError>,

    /// The name of the check this scoured in.
    ///
    /// This is not matched against, this is purely for printing.
    pub where_fail: String,
}

/// When command checks fail, they return a reason as to why they failed so we can inform the user.
#[derive(Debug)]
pub enum CommandCheckFailureReason {
    /// User was/is not enrolled in doints.
    UserNotEnrolled,

    /// There was another unexpected error. This is for handling things like failing to get DB connections during a check, or
    /// otherwise something that makes the check un-completable.
    CheckErroredOut(CommandCheckFailure),

    /// This command is not allowed in the channel it was called.
    InvalidChannel,

    /// We were unable to find the caller of the command's `Member`. This is distinct from a user not being enrolled, as
    /// we couldn't even check if they were enrolled, due to not finding them. Kapiche?
    MemberNotFound,

    /// Some crates (looking at you r2d2) return shitty error types (Strings) so we just have a generic for them.
    R2D2Failure(String),

    /// The user is currently in jail.
    UserInJail(JailedUser),
}

pub type Error = DointBotError; //TODO: This may need to be a box. Who knows
pub type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations.
// This includes things like access to the database pool.

pub type DbPool = diesel::r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug)]
pub struct Data {
    pub db_pool: DbPool,
}
