// errors related to the jail system.

use thiserror::Error;

use crate::database::tables::jail::JailedUser;

/// Jail-related errors.
#[derive(Error, Debug)]
pub(crate) enum JailError {
    #[error("The user is already in jail")]
    AlreadyInJail(JailedUser),

    #[error("User isn't in jail")]
    UserNotInJail,

    #[error("The user has more time to their sentence. Can't free them yet.")]
    StillServingSentence,

    #[error("Other diesel related errors.")]
    DieselError(#[from] diesel::result::Error),
}
