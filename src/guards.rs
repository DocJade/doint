#![allow(dead_code)]
use crate::prelude::*;
use paste::paste;
use poise::CreateReply;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum GuardError {
    #[error("User is not enrolled in doints!")]
    UserNotEnrolled,
    #[error("Cannot run that command in this channel!")]
    InvalidChannel,
    #[error("That member count not be found!")]
    MemberNotFound,
    #[error("That member is in jail!")]
    UserInJail(JailedUser),
}

/// # Errors
/// Will return a `BotError::Serenity` if getting the channel fails
/// Will return a `BotError::Guard` if it is the incorrect channel
///
/// Check if a given command is ran in the doints category
pub async fn in_doints_category(ctx: PoiseContext<'_>) -> Result<bool, BotError> {
    if let Some(category) = ctx
        .http()
        .get_channel(ctx.channel_id())
        .await
        .map_err(BotError::from)?
        .category()
        && category.id != DOINTS_CATEGORY_ID
    {
        return Err(BotError::from(GuardError::InvalidChannel));
    }

    Ok(true)
}

#[macro_export]
macro_rules! create_channel_guard {
    ($fn_name:ident,$channel_id:expr) => {
        paste! {
            /// # Errors
            /// Returns `Err` if the query fails
            ///
            /// Check if a given command is ran in the given channel
            pub async fn $fn_name(ctx: $crate::types::serenity_types::PoiseContext<'_>) -> Result<bool, $crate::errors::BotError> {
                if ctx.channel_id() == $channel_id {
                    Ok(true)
                } else {
                    ctx.send(CreateReply::default().content(format!("This command can only be used in the <#{}> channel.", $channel_id)).ephemeral(true)).await?;

                    Err(BotError::from(GuardError::InvalidChannel))
                }
            }

            /// # Errors
            /// Returns `Err` if the query fails
            ///
            /// Check if a given command is not ran in the given channel
            pub async fn [<not_ $fn_name>](ctx: $crate::types::serenity_types::PoiseContext<'_>) -> Result<bool, $crate::errors::BotError> {
                if ctx.channel_id() == $channel_id {
                    Ok(true)
                } else {
                    ctx.send(CreateReply::default().content(format!("This command cannot be used in the <#{}> channel.", $channel_id)).ephemeral(true)).await?;

                    Err(BotError::from(GuardError::InvalidChannel))
                }
            }
        }
    };
}

create_channel_guard!(in_casino, DOINTS_CASINO_CHANNEL_ID);
create_channel_guard!(in_discussion, DOINTS_DISCUSSION_CHANNEL_ID);
create_channel_guard!(in_commands, DOINTS_COMMANDS_CHANNEL_ID);
create_channel_guard!(in_dev, DOINTS_DEV_CHANNEL_ID);

/// # Errors
/// Returns `Err` if the query fails
///
/// Check that a member is enrolled in Doints
pub async fn member_enrolled_in_doints(ctx: PoiseContext<'_>) -> Result<bool, BotError> {
    let Some(member) = ctx.author_member().await else {
        return Err(BotError::from(GuardError::MemberNotFound));
    };
    Ok(Roles::member_enrolled_in_doints(&member))
}
