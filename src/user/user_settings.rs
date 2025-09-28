// The settings a user has access to changing.

// This entire struct is serialized into and out of the database for simplicity.

// The settings command is automatically deduced from the structure of this struct, so doc commends are required on everything
// this references.

use documented::Documented;
use serde::{Deserialize, Serialize};

use crate::notification::notification_settings::UserNotificationSettings;

/// Everything that a user can configure goes inhere. Everything from notification preferences, to 
#[derive(Documented, Default)] // Access to doc comments is required, and make sure we can get a default variant.
#[derive(Serialize, Deserialize)] // Turn it back and forth from JSON
pub(crate) struct DointUserSettings {
    /// Notification settings
    notification_settings: UserNotificationSettings
}