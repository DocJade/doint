// Get the name of a user based on a user ID.
use crate::{
    discord::helper::get_user::get_user_from_id,
    types::serenity_types::{Context, Data, Error},
};
use poise::serenity_prelude as serenity;

/// Gets the display of a user based on a UserID.
///
/// Note that this is may be different from their nickname.
pub(crate) async fn get_display_name(ctx: Context<'_>, id: u64) -> Result<String, Error> {
    // First we need the user
    let user = get_user_from_id(ctx, id).await?;

    // Now for the name
    Ok(user.display_name().to_string())
}
