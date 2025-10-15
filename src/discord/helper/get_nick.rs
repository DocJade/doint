// Get the name of a user based on a user ID.
use crate::prelude::*;

/// Gets the display of a user based on a `UserID`.
///
/// Note that this is may be different from their nickname.
pub async fn get_display_name(ctx: Context<'_>, id: u64) -> Result<String, Error> {
    // First we need the user
    let user = helper::get_user::get_user_from_id(ctx, id).await?;

    // Now for the name
    Ok(user.display_name().to_string())
}
