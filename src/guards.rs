#[allow(dead_code)]

use paste::paste;
use poise::CreateReply;

use crate::discord::checks::consented::member_enrolled_in_doints;
use crate::types::serenity_types::{CommandCheckFailureReason, Context, Error};

use crate::knob::channels::{self, DOINTS_CATEGORY_ID};

/// Check if the command was executed in the doints category
pub async fn in_doints_category(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(category) = ctx
        .http()
        .get_channel(ctx.channel_id())
        .await
        .or(Err(Error::CommandCheckFailed(
            CommandCheckFailureReason::InvalidChannel,
        )))?
        .category()
        && category.id != DOINTS_CATEGORY_ID
    {
        return Ok(false);
    }

    Ok(true)
}

#[macro_export]
macro_rules! create_channel_guard {
    ($fn_name:ident,$channel_id:expr) => {
        paste! {
            pub async fn $fn_name(ctx: crate::types::serenity_types::Context<'_>) -> Result<bool, crate::types::serenity_types::Error> {
                if ctx.channel_id() == $channel_id {
                    Ok(true)
                } else {
                    ctx.send(CreateReply::default().content(format!("This command can only be used in the <#{}> channel.", $channel_id)).ephemeral(true)).await?;

                    Ok(false)
                }
            }

            pub async fn [<not_ $fn_name>](ctx: crate::types::serenity_types::Context<'_>) -> Result<bool, crate::types::serenity_types::Error> {
                if ctx.channel_id() == $channel_id {
                    Ok(true)
                } else {
                    ctx.send(CreateReply::default().content(format!("This command cannot be used in the <#{}> channel.", $channel_id)).ephemeral(true)).await?;

                    Ok(false)
                }
            }
        }
    };
}

create_channel_guard!(in_casino, channels::DOINTS_CASINO_CHANNEL_ID);
create_channel_guard!(in_discussion, channels::DOINTS_DISCUSSION_CHANNEL_ID);
create_channel_guard!(in_commands, channels::DOINTS_COMMANDS_CHANNEL_ID);
create_channel_guard!(in_dev, channels::DOINTS_DEV_CHANNEL_ID);

/// Check if the caller has the dointer role.
pub async fn ctx_member_enrolled_in_doints(ctx: Context<'_>) -> Result<bool, Error> {
    let member = if let Some(member) = ctx.author_member().await {
        member
    } else {
        // Couldnt find user.
        // If we cant load them, chances are we arent in doccord.
        return Ok(false);
    };
    member_enrolled_in_doints(member.into_owned(), ctx).await
}
