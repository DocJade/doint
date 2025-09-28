// The menu that a user, well, uses to configure settings.

use crate::types::serenity_types::{Context, Error};


// How the settings window should look:
// Ephemeral, only the user changing settings can see it, this prevents others from clicking the buttons.
// The buttons should time out after 30 seconds, but all changes should be instantly submitted to the database.
// Its an embed. Each option in the current page/tab you're in is displayed in order
// folders that you can enter are bold and prefaced with `/`
// There is a selection that you can move up and down in the far left row.
// folders are always put at the top to make them easier to get into.
// the cursor can wrap around the top and bottom.
// The currently highlighted option is also bolded.
// You can only have up to 5 buttons to interact with on a message.

// Doint settings
// /**folder name** // Display the current tab
// This is a description of this settings tab, ie what it covers.
// ------------ // hori bar
// . / folder
// . / folder
// > **/ folder**
// . setting
// . setting
// ------------ // hori bar
// [Back] [^] [\/] [Select]

// When you've selected a setting, the entire window is replaced (we dont open it like a folder) and you can pick from the enum varients via the same
// cursor-ing selection.

// Setting an option doesn't boot you out of the menu.

// Doint settings
// Tax collection // Setting that is currently being changed
// (description goes here)
// ------------ // hori bar
// . [ ] enum variant
// - Description of variant
// . [*] enum variant // this option is currently enabled
// - Description of variant
// > [ ] **enum variant** // this option is currently selected
// - Description of variant
// . [ ] enum variant
// - Description of variant
// ------------ // hori bar

// [Back] [^] [\/] [Set]


//
// The user settings command
//

/// Configure your Doint experience!
#[poise::command(slash_command, guild_only)]
pub(crate) async fn settings(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // We need to generate buttons

    todo!()
}