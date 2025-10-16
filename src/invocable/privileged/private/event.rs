// Force-run events.

use poise::CreateReply;

use crate::prelude::*;

/// Force disperse UBI immediately.
///
/// This may be automatically overridden later by other tax calculations.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR", // Only admins can run/see this command.
    check = guards::in_doints_category,
    check = guards::in_commands
    )
]
pub async fn admin_force_disperse_ubi(ctx: PoiseContext<'_>) -> Result<(), BotError> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    let dispersed = EventCaller::ubi_time(&mut conn);
    let response_text: String = match dispersed {
        Ok(ok) => match ok {
            Some(given) => {
                #[allow(clippy::cast_possible_wrap)] // Nuh uh.
                let preference = if let Some(member) = &ctx.author().member {
                    if let Some(user) = &member.user {
                        DointFormatterPreference::from(user)
                    } else {
                        crate::knob::formatting::FORMATTER_PREFERENCE
                    }
                } else {
                    crate::knob::formatting::FORMATTER_PREFERENCE
                };

                let formatted = DointFormatter::display_doint_string(&given, &preference);
                format!("Dispersed {formatted} to each player.")
            }
            None => "Bank could not afford UBI.".to_string(),
        },
        Err(err) => {
            format!("UBI failed: {err:#?}")
        }
    };

    // Assemble a response
    let response = CreateReply::default()
        .ephemeral(true)
        .content(response_text);

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}
