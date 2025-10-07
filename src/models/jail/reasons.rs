use core::fmt;
use std::io::Write;

use chrono::TimeDelta;
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    mysql::{Mysql, MysqlValue},
    serialize::{Output, ToSql},
    sql_types::Text,
};
use log::warn;

#[derive(FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum JailReason {
    /// Attempted to steal money from a user (did not succeed)
    AttemptedRobbery,

    /// Unknown, probably an old reason that was deleted.
    ///
    /// If a user has this reason, they'll be freed from jail as usual when their sentence ends.
    #[deprecated = "This is only used when loading in unknown values from the DB. This should NOT be outgoing!"]
    Unknown,
}

impl JailReason {
    /// Returns a [`TimeDelta`] of how long a user should be jailed for based on the crime.
    pub(crate) fn to_time(self) -> TimeDelta {
        // Get how many seconds they should be in jail for
        let duration_seconds: i64 = match self {
            JailReason::AttemptedRobbery => 60 * 60, // 1 hour
            #[allow(deprecated)] // Need to handle the case regardless.
            JailReason::Unknown => {
                // You shouldn't be going to jail for an unknown reason.
                // 10 seconds just to get them back out of jail ASAP.
                warn!("Tried to send a user to jail for a reason of Unknown!");
                10
            }
        };

        // Turn that into a delta, and return it
        TimeDelta::seconds(duration_seconds)
    }
}

/// Who/what sent this user to jail?
#[derive(FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum JailCause {
    /// An admin sent this user to jail manually.
    Admin,

    /// The police caught them
    ThePolice,

    /// Unknown, probably old.
    #[deprecated = "This is only used when loading in unknown values from the DB. This should NOT be outgoing!"]
    Unknown,
}

impl fmt::Display for JailReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JailReason::AttemptedRobbery => write!(f, "AttemptedRobbery"),
            #[allow(deprecated)] // Need to handle the case regardless.
            JailReason::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TryFrom<&str> for JailReason {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AttemptedRobbery" => Ok(JailReason::AttemptedRobbery),
            // Anything else is no longer in our schema, hence unknown.
            #[allow(deprecated)] // Need to handle the case regardless.
            _ => Ok(JailReason::Unknown),
        }
    }
}

impl FromSql<Text, Mysql> for JailReason {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Mysql> for JailReason {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl fmt::Display for JailCause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JailCause::Admin => write!(f, "Admin"),
            JailCause::ThePolice => write!(f, "ThePolice"),
            #[allow(deprecated)] // Need to handle the case regardless.
            JailCause::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TryFrom<&str> for JailCause {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Admin" => Ok(JailCause::Admin),
            // Anything else is no longer in our schema, hence unknown.
            #[allow(deprecated)] // Need to handle the case regardless.
            _ => Ok(JailCause::Unknown),
        }
    }
}

impl FromSql<Text, Mysql> for JailCause {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Mysql> for JailCause {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}
