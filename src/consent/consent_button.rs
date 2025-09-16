// Clicking the consent button adds you to the database.

use crate::{database::queries::get_user::get_user, knob::terms_and_conditions::TERMS_AND_CONDITIONS_TEXT, schema::users::id, types::serenity_types::{Context, Data, Error}};
use log::{error, info, warn};
use poise::{serenity_prelude as serenity, CreateReply};
use diesel::{Connection, MysqlConnection};
use diesel::prelude::*;
use crate::schema::users::dsl::{users, bal};

use crate::{database::tables::users::DointUser};

/// Consent to the doint system.
#[poise::command(slash_command, guild_only)]
pub(crate) async fn opt_in(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // User wants to opt into the database. Check if they're already here.
    let users_id: u64 = ctx.author().id.into();

    // get db connection
    let pool = ctx.data().db_pool.clone();
    let mut conn = pool.get()?;

    if get_user(users_id, &mut conn)?.is_some() {
        // User is already in DB.
        // Tell user they are an idiot.
        // ephemeral so only they see it.
        let _ = ctx.send(CreateReply::default().ephemeral(true).content("You've already opted in.")).await?;
        return Ok(());
    }

    // This is a new user. Add them to the database.
    info!("Adding new user to database...");
    // This is a one time operation, so we do it here.

    // Assemble them.
    let new_user: DointUser = DointUser {
        id: users_id,
        bal: 0, // Broke ass lmao
    };

    // Add them.
    conn.transaction(|conn| {
        diesel::insert_into(users).values(new_user).execute(conn)
    })?;
    
    // User added!

    // Now that they have been added to the database, inform the user of their rights.
    // ephemeral so only they see it.
    let terms = CreateReply::default().ephemeral(true).content(TERMS_AND_CONDITIONS_TEXT);

    // We'll try replying 3 times before bailing out
    for _ in 0..3 {
        if ctx.send(terms.clone()).await.is_ok() {
            // Done!
            return Ok(())
        }
    }

    warn!("Failed to reply to user after adding them to the DB! Rolling back...");

    // Unable to inform user the standard way...
    // Roll back the database add.
    let removal = conn.transaction(|conn| {
        diesel::delete(users).filter(id.eq(users_id)).execute(conn)
    });

    match removal {
        Ok(ok) => {
            if ok == 1 {
                // Rollback worked.
                info!("Rolled back.");
            } else {
                error!("Failed to remove un-consenting user!");
                // TODO: This should message mods.
            }
        },
        Err(err) => {
            // Removing the user failed completely.
            // This is different from removing 0 people.
            error!("Attempt to remove un-consenting user failed!");
            error!("{err:#?}");
            // TODO: This should message mods.
            return Err(Box::new(err))
        },
    }
    
    Ok(())
}