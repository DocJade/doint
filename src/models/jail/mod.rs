pub mod data;
// Naughty Dointers

pub mod arrest;
pub mod reasons;

use thiserror::Error;

use crate::prelude::*;

#[derive(Error, Debug)]
pub enum JailError {
    #[error("The user is already in jail")]
    AlreadyInJail(JailedUser),

    #[error("User isn't in jail")]
    UserNotInJail,

    #[error("The user has more time to their sentence. Can't free them yet.")]
    StillServingSentence,

    #[error("Other diesel related errors.")]
    DieselError(#[from] diesel::result::Error),
}
