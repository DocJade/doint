// Checks for when a command takes both the person who is running the command, and
// another user as an operand

use poise::serenity_prelude::Member;
use thiserror::Error;

use crate::types::serenity_types::Context;

/// Enum for keeping track of reasons that the user cannot be called against.
#[derive(Error, Debug)]
pub(crate) enum IneligibleDestinationUser {
    #[error("That user is not a dointer.")]
    UserNotEnrolled,

    #[error("User is immune from this command.")]
    UserImmune(UserImmuneReason),

    #[error("Other diesel related errors.")]
    DieselError(#[from] diesel::result::Error),
}

/// Sometimes users are immune from a certain command, these are the reasons.
#[derive(Debug)]
pub(crate) enum UserImmuneReason {
    /// This user has paid for some kind of immunity to this action
    UserBoughtProtection,
}

/// The type of command being called.
#[derive(Debug)]
pub(crate) enum CommandType {
    /// Crime related, user is on the receiving end.
    NegativeCrime,

    /// User will gain doints from the interaction with no downsides
    IncomingDoints,
}

/// Requirements for the user being checked.
///
/// Check is skipped on a None.
///
/// All users must be in the doint DB. This will always be checked.
pub(crate) struct DestinationUserRequirements {}

/// Checks that a user you're trying to do an operation against can receive this command.
///
/// Prints information to the user if this user is unable to have actions called against them.
///
/// Returns an error if the user does not pass after printing information to user.
pub(crate) fn check_destination_user(ctx: Context<'_>, member: Member) {}
