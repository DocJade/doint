use diesel::r2d2::{self, ConnectionManager};
use diesel::MysqlConnection;
use log::info;
// Starting the bot
use poise::serenity_prelude as serenity;

use crate::discord::handlers::{error::handle_error, event::handle_discord_event};
use crate::invokable::standard::information::public::leaderboard::leaderboard;
use crate::types::serenity_types::{Context, Data, DbPool, Error};

/// Create the client which will be used to start the bot.
/// 
/// Requires a discord token.
/// 
/// # Panics
///
/// Will panic if we cant clean up old commands
pub async fn create_client(discord_token: String, database_url: String) -> serenity::Client {
    let wip_client = poise::Framework::builder()
    .options(
        poise::FrameworkOptions {
            commands: vec![
                leaderboard()
            ],
            // Handle errors when they occur.
            on_error: |error| {
                todo!("error handler, weirdly wants to be on the heap. {error:#?}");
                // handle_error(Box::new(&error));
            },
            // Handle discord events
            event_handler: |ctx, event, framework, data| {
                Box::pin(handle_discord_event(ctx, event, framework, data))
            },
            // automatically deduce server owner
            initialize_owners: true,
            ..Default::default()
        }
    ).setup(move |ctx, ready, framework| {
        Box::pin(async move {
            // set up slash commands
            println!("Logged in as {}", ready.user.name);
            // delete the old commands
            let old_slash_commands = match ctx.http.get_global_commands().await {
                Ok(commands) => commands,
                Err(err) => panic!("Unable to get old commands! : {err:#?}"),
            };
            if !old_slash_commands.is_empty() {
                // Remove old commands
                info!("Cleaning up old slash commands...");
                for command in old_slash_commands {
                    info!("Removing : {} ... ", command.name);
                    match ctx.http.delete_global_command(command.id).await {
                        Ok(()) => info!("Done."),
                        Err(err) => {
                            panic!("Could not delete the following command! : {err:#?}")
                        }
                    };
                }
                info!("Done cleaning up old slash commands!");
            }
            info!("Registering new slash commands...");
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            info!("Done.");

            // Now that the slash commands are set up, we will get ahold of the database.
            // Make a connection manager, and a pool.
            let manager = ConnectionManager::<MysqlConnection>::new(database_url);
            let db_pool: DbPool = r2d2::Pool::builder().build(manager).expect("Failed to create DB pool!");


            // Set up shared data.
            Ok(Data {
                db_pool,
            })
        })
    })
    .build();

    // Set gateway intents, ie get permission to do stuff.
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client: Result<serenity::Client, serenity::Error> = serenity::ClientBuilder::new(discord_token, intents)
        .framework(wip_client)
        .await;

    client.unwrap()
}