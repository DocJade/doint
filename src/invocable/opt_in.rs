// Clicking the consent button adds you to the database.
use crate::prelude::*;

use bigdecimal::{BigDecimal, Zero};
use diesel::Connection;
use diesel::prelude::*;
use log::{error, info, warn};
use poise::CreateReply;

/// Consent to the doint system.
#[poise::command(slash_command, guild_only)]
pub async fn opt_in(ctx: PoiseContext<'_>) -> Result<(), BotError> {
    // User wants to opt into the database. Check if they're already here.
    let users_id: u64 = ctx.author().id.into();

    // get db connection
    let pool = ctx.data().db_pool.clone();
    let mut conn = pool.get()?;

    if Users::get_doint_user(users_id, &mut conn)?.is_some() {
        // User is already in DB.
        // Tell user they are an idiot.
        // ephemeral so only they see it.
        let _ = ctx
            .send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("You've already opted in."),
            )
            .await?;
        return Ok(());
    }

    // This is a new user. Add them to the database.
    info!("Adding new user to database...");
    // This is a one time operation, so we do it here.

    // Assemble them.
    let new_user: DointUser = DointUser {
        id: users_id,
        bal: BigDecimal::zero(), // Broke ass lmao
    };

    // Add them.
    conn.transaction(|conn| {
        diesel::insert_into(users_table)
            .values(new_user)
            .execute(conn)
    })?;

    // User added!

    // Now that they have been added to the database, inform the user of their rights.
    // ephemeral so only they see it.
    let terms = CreateReply::default()
        .ephemeral(true)
        .content(TERMS_AND_CONDITIONS_TEXT);

    // We'll try replying 3 times before bailing out
    for _ in 0..3 {
        // Give them the dointer role.
        if !Roles::give_doints_role(ctx, users_id).await? {
            // Adding the role failed.
            continue;
        }
        if ctx.send(terms.clone()).await.is_err() {
            continue;
        }
        // User now has role, and saw message.
        return Ok(());
    }

    // Failed to either give the user the role, or to tell them the T&C
    warn!("New user enrollment failed! Rolling back...");

    // Unable to inform user the standard way...
    // Roll back the database add.
    let removal: Result<usize, diesel::result::Error> = conn.transaction(|conn| {
        diesel::delete(users_table)
            .filter(user_id_col.eq(users_id))
            .execute(conn)
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
        }
        Err(err) => {
            // Removing the user failed completely.
            // This is different from removing 0 people.
            error!("Attempt to remove un-consenting user failed!");
            error!("{err:#?}");
            // TODO: This should message mods.
            return Err(err)?;
        }
    }

    // Removing the role is done afterwards, since if they didnt get removed from the DB, they still need the role.
    if !Roles::revoke_doints_role(ctx, users_id).await? {
        warn!("User [{users_id}] now has the dointer role without being in the DB!");
        // TODO: This should message mods.
    }

    Ok(())
}
