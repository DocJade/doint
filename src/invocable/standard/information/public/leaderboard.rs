// Display a doint leaderboard

use bigdecimal::BigDecimal;

use crate::prelude::{helper::get_nick::get_display_name, *};

/// See the top Doint holders!
#[poise::command(slash_command, guild_only, aliases("lb"), check = guards::in_doints_category, check = guards::in_commands)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), BotError> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Go get the top 10 users
    let users: Vec<DointUser> = Leaderboard::get_top_doint_balances(10, &mut conn)?;

    // Now construct a nicer list with the user's names.
    let mut names_and_points: Vec<(String, BigDecimal)> = Vec::with_capacity(users.len());
    for user in users {
        let name = get_display_name(ctx, user.id).await?;
        names_and_points.push((name, user.bal));
    }

    let preference = if let Some(member) = &ctx.author().member {
        if let Some(user) = &member.user {
            DointFormatterPreference::from(user)
        } else {
            crate::knob::formatting::FORMATTER_PREFERENCE
        }
    } else {
        crate::knob::formatting::FORMATTER_PREFERENCE
    };

    // Now make a leaderboard message out of that.
    let mut response: String = "Leaderboard:".to_string();
    for (rank, (name, doints)) in names_and_points.iter().enumerate() {
        // Format the doint string
        let doint_string = DointFormatter::display_doint_string(doints, &preference);
        response.push_str(&format!("\n- {}: {name} - {doint_string}", rank + 1));
    }

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only, aliases("poor"), check = guards::in_doints_category, check = guards::in_commands)]
/// See the bottom 10 Doint holders!
pub async fn broke(ctx: Context<'_>) -> Result<(), BotError> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Go get the top 10 users
    let users: Vec<DointUser> = Leaderboard::get_bottom_doint_balances(10, &mut conn)?;

    // Now construct a nicer list with the user's names.
    let mut names_and_points: Vec<(String, BigDecimal)> = Vec::with_capacity(users.len());
    for user in users {
        let name = get_display_name(ctx, user.id).await?;
        names_and_points.push((name, user.bal));
    }

    let preference = if let Some(member) = &ctx.author().member {
        if let Some(user) = &member.user {
            DointFormatterPreference::from(user)
        } else {
            crate::knob::formatting::FORMATTER_PREFERENCE
        }
    } else {
        crate::knob::formatting::FORMATTER_PREFERENCE
    };

    // Now make a leaderboard message out of that.
    let mut response: String = "Brokies:".to_string();
    for (rank, (name, doints)) in names_and_points.iter().enumerate() {
        // Format the doint string
        let doint_string = DointFormatter::display_doint_string(doints, &preference);
        response.push_str(&format!("\n- {}: {name} - {doint_string}", rank + 1));
    }

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}
