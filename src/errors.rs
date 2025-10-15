use diesel::{r2d2, result::Error as DieselError};
use poise::serenity_prelude::Error as SerenityError;
use std::fmt;
use thiserror::Error;

use crate::prelude::*;

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
}

impl BotError {
    pub fn r2d2(err: r2d2::Error, severity: ErrorSeverity) -> Self {
        Self::R2D2 {
            severity,
            source: err,
        }
    }

    pub fn diesel(err: DieselError, severity: ErrorSeverity) -> Self {
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

    pub fn guard(err: GuardError, severity: ErrorSeverity) -> Self {
        Self::Guard {
            severity,
            source: err,
        }
    }
}

impl BotError {
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        match &mut self {
            Self::R2D2 { severity: s, .. }
            | Self::Diesel { severity: s, .. }
            | Self::Guard { severity: s, .. }
            | Self::DieselPool { severity: s, .. }
            | Self::Serenity { severity: s, .. } => *s = severity,
        }
        self
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

pub struct ErrorHandler {}

pub type ErrorHandlerData<'a> = poise::FrameworkError<'a, Data, BotError>;

impl ErrorHandler {
    pub async fn handle(err: ErrorHandlerData<'static>) {
        match err {
            Some(poise::FrameworkError) => ErrorHandler::handle_framework_error(err).await
        }
    }
}

impl ErrorHandler {
    pub async fn handle_framework_error(err: ErrorHandlerData) {}
}
