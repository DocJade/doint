// Let uses set their notification preferences.

// We use `Documented` to use the doc comments as our display strings

use diesel::{expression::AsExpression, prelude::{Insertable, Queryable}, sql_types::{Json, Text}, MysqlConnection};
use documented::Documented;

use crate::{database::tables::users::DointUser, types::serenity_types::{Context, Error}};


use serde::{Deserialize, Serialize};

/// All of the settings a user is allowed to touch that are related to how they
/// are notified about events.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)] // Turn it back and forth from JSON
pub(crate) struct UserNotificationSettings {
    /// Events related to criminal stuff
    pub(crate) crime_notifications: CrimeNotifications,
    /// Events related to banking
    pub(crate) bank_notifications: BankingNotifications,
}

/// Where a user would like to get their notifications.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[derive(Documented)]
#[derive(Serialize, Deserialize)] // Turn it back and forth from JSON
pub(crate) enum NotificationLocation {
    /// Get notified with a direct message.
    DirectMessage,

    /// Get notified with a new message in the channel that the notification originated in.
    /// IE, if somebody robs you in general, a message will be sent there, pinging you about it.
    /// 
    /// If the action that created this notification doesn't have an origin (Think UBI dispersal, or taxes) the notification
    /// will fall back to the doint-notification channel.
    SameChannel,

    /// Get notified via a reply to the message that caused the notification.
    /// IE, if somebody robs you, the bot will reply to that message and ping you.
    Reply,

    /// Do not notify me.
    /// This is the default option.
    #[default]
    DoNotNotify
}

/// What general events a user wants to be notified about
#[derive(Documented, Default, Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)] // Turn it back and forth from JSON
pub(crate) struct CrimeNotifications {
    /// You've been the victim of a robbery!
    pub(crate) robbed: NotificationLocation,

    /// You've been released from jail.
    pub(crate) released_from_jail: NotificationLocation,
}

/// What banking related events a user wants to be notified about
#[derive(Documented, Default, Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)] // Turn it back and forth from JSON
pub(crate) struct BankingNotifications {
    /// Daily universal basic income has been dispersed!
    /// 
    /// This notification tells you how many doints you received.
    pub(crate) ubi_dispersed: NotificationLocation,

    /// Taxes have been collected,
    /// 
    /// This notification tells you how many doints you paid.
    pub(crate) taxes_collected: NotificationLocation
}


// Getting the settings of a user
impl DointUser {
    /// Get the notification settings of this user
    pub(crate) fn notification_settings(&self, conn: &mut MysqlConnection) -> Result<UserNotificationSettings, Error> {
        todo!()
    }
}