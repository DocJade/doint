use crate::discord::checks::consented::member_enrolled_in_doints;
use crate::types::serenity_types::{CommandCheckFailureReason, Context, Error};

use crate::knob::channels::DOINTS_CATEGORY_ID;

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
