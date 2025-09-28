// Notify users of stuff.

use crate::{database::tables::users::DointUser, notification::{mapping::{find_notification_destination, NotificationType}, notification_settings::UserNotificationSettings, notification_struct::NotificationHelper}, types::serenity_types::Error};


// To send a notification, you need to build it.

// Notifications will be sent as embeds. The body text goes in the embed.

/// A notification to send to a user.
pub(crate) struct NotificationMessage {
    /// We need to have a reason that we match against user settings to check the method we use to send the notification.
    /// This enum does not show up in the final message, its only used to figure out where/if to send it.
    notification_type: NotificationType
}



// We dont impl sending notifications to users with the Notification helper, since it is called on a user directly.


impl DointUser {
    /// Send a notification to a user.
    /// 
    /// Automatically checks notification status on the user.
    /// 
    /// Notifications will be tried at most 5 times before giving up.
    /// If the final attempt fails, this will return an Error.
    pub(crate) fn notify(&self) -> Result<(), Error> {
        // Get the user's preferences
        self.
        // Get the user's notification preference for this action
        find_notification_destination()
    }
}


