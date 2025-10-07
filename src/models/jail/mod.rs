pub mod data;
// Naughty Dointers

pub(crate) mod arrest;
pub(crate) mod reasons;

use thiserror::Error;

use crate::database::tables::jail::JailedUser;

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
