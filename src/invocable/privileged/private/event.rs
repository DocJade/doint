// Force-run events.

use poise::CreateReply;

use crate::event::event_struct::EventCaller;
use crate::formatting::format_struct::FormattingHelper;
use crate::types::serenity_types::{Context, Data, Error};

/// Force disperse UBI immediately.
/// 
/// This may be automatically overridden later by other tax calculations.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR" // Only admins can run/see this command.
    )
]
pub(crate) async fn admin_force_disperse_ubi(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    let dispersed = EventCaller::ubi_time(&mut conn);
    let response_text: String = match dispersed {
        Ok(ok) => match ok {
            Some(given) => {
                #[allow(clippy::cast_possible_wrap)] // Nuh uh.
                let formatted = FormattingHelper::display_doint(given as i32);
                format!("Dispersed {formatted} to each player.")
            },
            None => {
                "Bank could not afford UBI.".to_string()
            },
        },
        Err(err) => {
            format!("UBI failed: {err:#?}")
        },
    };

    // Assemble a response
    let response = CreateReply::default().ephemeral(true).content(response_text);

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}