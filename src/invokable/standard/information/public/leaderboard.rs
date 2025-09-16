// Display a doint leaderboard

use crate::{database::{queries::top_n::get_top_n, tables::users::DointUser}, discord::helper::get_nick::get_display_name, types::serenity_types::{Context, Data, Error}};

/// See the top Doint holders!
#[poise::command(slash_command, guild_only, aliases("lb"))]
pub(crate) async fn leaderboard(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a cobnnection
    let mut conn = pool.get()?;

    // Go get the top 10 users
    let users: Vec<DointUser> = get_top_n(10, &mut conn)?;

    // Now construct a nicer list with the user's names.
    let mut names_and_points: Vec<(String, i32)> = Vec::with_capacity(users.len());
    for user in users {
        let name = get_display_name(ctx, user.id).await?;
        names_and_points.push((name, user.bal));
    };

    // Now make a leaderboard message out of that.
    let mut response: String = "Leaderboard:".to_string();
    for (rank, (name, doints)) in names_and_points.iter().enumerate() {
        response.push_str(&format!("\n- {}: {name} [{doints}]", rank + 1));
    };

    // Send it.
    let _ = ctx.say(response).await?;
    Ok(())
}