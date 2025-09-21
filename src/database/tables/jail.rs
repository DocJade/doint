// laughing all the way there. haw haw haw.

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::{database::tables::users::DointUser, jail::reasons::{JailCause, JailReason}};

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Insertable)]
#[diesel(belongs_to(DointUser, foreign_key = id))]
#[diesel(table_name = crate::schema::jail)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct JailedUser {
    /// Key to the user in the `users` table.
    pub id: u64,

    /// The time they'll be released from jail. UTC
    pub until: NaiveDateTime,

    /// See the `JailReason` enum
    pub reason: JailReason,

    // See the `JailCause` enum
    pub cause: JailCause,

    // Can this person be bailed out?
    pub can_bail: bool,
}


/*
`id` BIGINT UNSIGNED NOT NULL COMMENT 'fkey to users table',
`until` TIMESTAMP NOT NULL COMMENT 'When the user should be let out of jail',
`reason` TINYTEXT NOT NULL COMMENT 'See the JailReason enum',
`cause` TINYTEXT NOT NULL COMMENT 'See the JailCause enum',
`can_bail` TINYINT NOT NULL COMMENT 'Can this person be bailed out? 0/1',
*/