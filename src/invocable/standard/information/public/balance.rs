// See your doint balance

use crate::database::queries::user::get_doint_user;
use crate::formatting::format_struct::FormattingHelper;
use crate::types::serenity_types::{Context, Error};


/// See your doint balance.
#[poise::command(slash_command, guild_only, aliases("bal"))]
pub(crate) async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Get the user, if they dont exist, return false.
    let user = if let Some(user) = get_doint_user(ctx.author().id, &mut conn)? {
        user
    } else {
        // Couldn't find em.
        // TODO: When commands fail, tell the user the reason instead of just silence.
        return Ok(());
    };

    // Format the doint number
    let doint_string = FormattingHelper::display_doint(&user.bal);

    // Now print out their balance.
    let response: String = format!("You currently have {doint_string}.");

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}
