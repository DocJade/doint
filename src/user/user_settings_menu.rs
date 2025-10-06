// The menu that a user, well, uses to configure settings.

use std::ops::Range;

use poise::CreateReply;

use crate::{notification::notification_settings::NotificationLocation, types::serenity_types::{Context, Error}, user::user_settings::DointUserSettings};


// How the settings window should look:
// Ephemeral, only the user changing settings can see it, this prevents others from clicking the buttons.
// The buttons should time out after 30 seconds, but all changes should be instantly submitted to the database.


// Just use a string select, a lot of cooler options were explored but tree based proc-macro generation of
// menus from structs is just, WAY out of scope.
// https://discord.com/developers/docs/components/reference#string-select


// string select from the top level. If its a struct, we open another string select with that struct's fields.
// If it's an enum, string select for the different config options.
// Then an enum has a string select done, that change is immediately updated in the DB.

// Think of folders as Structs, and settings as Enums.




/// Change notification-related settings.
#[poise::command(slash_command, guild_only)]
pub(crate) async fn settings(
    ctx: Context<'_>,
) -> Result<(), Error> {

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;



    // We need to generate buttons

    todo!()
}


// Traits to facilitate settings menus.

/// A container of sub-folders or settings.
/// 
/// A SettingsMenuFolder cannot contain both settings AND folders.
pub(super) trait SettingsMenuFolder {
    /// Get the name of the folder.
    fn name(&self) -> String;

    /// Get the configurable settings in this folder.
    /// 
    /// Returns none if there are not settings in this folder.
    fn settings(&self) -> Option<Vec<Box<dyn SettingsMenuSetting>>>;

    /// Get the sub-folders that this folder contains.
    /// 
    /// Returns none if there are no sub-folders.
    fn folders(&self) -> Option<Vec<Box<dyn SettingsMenuFolder>>>;
}

/// An type that can be placed in a settings menu, and can be configured.
/// 
/// Regardless of underlying type, all options are manipulated as strings.
pub(super) trait SettingsMenuSetting {
    /// Get the current value of this setting
    fn current_value(&self) -> SettingValue;

    /// Get the description of an option (ie "false" -> "disables XYZ").
    fn get_description(&self, value: SettingValue) -> String;

    /// Get all of the possible options for this setting
    fn get_possible_values(&self) -> Vec<SettingValue>;

    /// Change this setting.
    fn set(&mut self, new_value: SettingValue) -> Result<(), ()>;
}

/// The possible values for a setting.
pub(super) enum SettingValue {
    /// Either true or false.
    Boolean(bool),

    /// Signed integer, and its allowed range of values.
    Number(isize, Range<isize>),

    /// Freeform string input. Max length included.
    String(String, usize),

    /// Enum variant.
    /// 
    /// The variants of the enum are tracked via strings.
    Enum(&'static str)
}



//
// Rendering
//

// Render the folder view.
// We dont have a "go up a directory" function, since that would require tracking parents, which is annoying.
impl SettingsMenuFolder {
    /// Render out the folder, regardless if it has settings or more folders contained within.
    fn render(&self) -> CreateReply {
        todo!("Render out the string selection list for the sub-folders or settings in the folder")
    }

    /// Open a sub-folder.
    fn open_subfolder(&self, subfolder_identifier: String) -> SettingsMenuFolder {
        todo!("Return the sub-folder that matches.")
    }

    /// Open a setting
    fn open_setting(&self, setting_identifier: String) -> SettingsMenuSetting {
        todo!("Return the setting")
    }
}

// render the setting view
impl SettingsMenuSetting {
    /// Render out the setting. Regardless of setting type.
    fn render(&self) -> CreateReply {
        // This includes it's name, description, and has a section to change the setting accordingly.
        todo!("Render dat.")
    }

    /// Change the setting
    fn set(&mut self, new_value: SettingValue) -> CreateReply {
        // This should update the user settings type, then re-draw the menu with the setting changed.
        todo!("Render dat.")
    }
}