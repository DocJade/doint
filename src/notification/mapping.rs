// All possible notifications, and the settings that gate them.

use crate::notification::notification_settings::{NotificationLocation, UserNotificationSettings};

pub(crate) enum NotificationType {
    // user related

    /// User was robbed
    Robbery,

    /// User was released from jail
    FreedFromJail,

    // bank related

    /// UBI was dispersed
    UBIDispersal,

    /// Bank collected taxes
    BankTaxes
}

/// Find where this kind of notification goes according to a user's preferences.
pub(super) fn find_notification_destination(notification: NotificationType, prefs: UserNotificationSettings) -> NotificationLocation {
    match notification {
        NotificationType::Robbery => prefs.crime_notifications.robbed,
        NotificationType::UBIDispersal => prefs.bank_notifications.ubi_dispersed,
        NotificationType::BankTaxes => prefs.bank_notifications.taxes_collected,
        NotificationType::FreedFromJail => prefs.crime_notifications.released_from_jail,
    }
}