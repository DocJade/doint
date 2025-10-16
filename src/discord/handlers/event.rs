use std::{sync::Once, time::Duration};

use log::{error, info, warn};

use crate::{event::activity::activity_reward_struct::ActivityRewardHelper, prelude::*};
use poise::serenity_prelude as serenity;

static INIT: Once = Once::new();

pub async fn handle_discord_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, PoiseContextData, BotError>,
    data: &PoiseContextData,
) -> Result<(), BotError> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("Ready! Logged in as {}", data_about_bot.user.name);
            info!(
                "Obtained eminent domain over {} guilds",
                data_about_bot.guilds.len()
            );

            // Leave servers other than Doccord.
            if data_about_bot.guilds.len() != 1 {
                while let Some(guild) = &data_about_bot
                    .guilds
                    .iter()
                    .find(|guild| guild.id != DOCCORD_SERVER_ID)
                {
                    ctx.http.leave_guild(guild.id).await?;
                }

                return Ok(());
            }

            // Set up things that run a single time.
            info!("Doing first time setup");

            // Only do this once.
            let mut keep_going = false;
            INIT.call_once(|| {
                // This will only run the first time we pass this code block, which will let the following
                // startup routines run. Otherwise, we will always skip it.
                keep_going = true;
            });

            if !keep_going {
                // Stop here.
                info!("Prevented setup from running again.");
                return Ok(());
            }

            info!("Running setup...");
            info!("Spinning up periodic tasks...");

            // Daily tasks
            info!("- Daily tasks...");
            let daily_db_pool = data.db_pool.clone();
            tokio::spawn(async move {
                // every day, 24 hours
                loop {
                    // We try running the daily tasks 5 times at max.
                    let mut worked = false;
                    for _ in 0..5 {
                        info!("Running daily tasks...");
                        // Get that DB connection
                        let maybe_conn = daily_db_pool.get();

                        let Ok(mut conn) = maybe_conn else {
                            warn!("Failed to get DB connection!");
                            continue;
                        };

                        info!("- - Taxes and UBI");
                        let run = EventCaller::daily_events(&mut conn);
                        worked = if let Ok(maybe) = run {
                            maybe
                        } else {
                            warn!("Daily task errored!");
                            warn!("{run:#?}");
                            false
                        };

                        if worked {
                            break;
                        }
                        warn!("Daily task failed...");
                    }

                    if worked {
                        info!("Dailies finished successfully!");
                    } else {
                        error!("All 5 daily task attempts failed!");
                        // TODO: Tell admins
                    }

                    info!("See you tomorrow!");

                    // See you tomorrow!
                    tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;
                }
            });

            // Hourly
            info!("- Hourly tasks...");
            let daily_db_pool = data.db_pool.clone();
            tokio::spawn(async move {
                // Every hour
                loop {
                    // Try at max 5 times
                    let mut worked = false;
                    for _ in 0..5 {
                        info!("Running hourly tasks...");
                        // Get that DB connection
                        let maybe_conn = daily_db_pool.get();

                        let Ok(mut conn) = maybe_conn else {
                            warn!("Failed to get DB connection!");
                            continue;
                        };

                        let run = EventCaller::hourly_events(&mut conn);
                        worked = if let Ok(maybe) = run {
                            maybe
                        } else {
                            warn!("Hourly task errored!");
                            warn!("{run:#?}");
                            false
                        };

                        if worked {
                            break;
                        }
                        warn!("Hourly task failed...");
                    }

                    if worked {
                        info!("Hourly finished successfully!");
                    } else {
                        error!("All 5 hourly task attempts failed!");
                        // TODO: Tell admins
                    }

                    info!("See you in an hour!");

                    // Wait an hour
                    tokio::time::sleep(Duration::from_secs(60 * 60)).await;
                }
            });

            // Minute tasks
            info!("- Minute tasks...");
            let daily_db_pool = data.db_pool.clone();
            tokio::spawn(async move {
                // Every minute
                loop {
                    // Try at max 5 times
                    let mut worked = false;
                    for _ in 0..5 {
                        // Get that DB connection
                        let maybe_conn = daily_db_pool.get();

                        let Ok(mut conn) = maybe_conn else {
                            warn!("Failed to get DB connection!");
                            continue;
                        };

                        let run = EventCaller::minute_events(&mut conn);
                        worked = if let Ok(maybe) = run {
                            maybe
                        } else {
                            warn!("Minute task errored!");
                            warn!("{run:#?}");
                            false
                        };

                        if worked {
                            break;
                        }
                        warn!("Minute task failed...");
                    }

                    if worked {
                        // Cool, but no message since this is noisy in logs
                        // info!("Minute tasks finished successfully!");
                    } else {
                        error!("All 5 hourly task attempts failed!");
                        // TODO: Tell admins
                    }

                    // info!("See you in a minute!");

                    // Wait 1 minute
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });
        }
        serenity::FullEvent::Ratelimit { data } => {
            info!("Ratelimited! [{}]", data.path);
        }
        serenity::FullEvent::Message { new_message } => {
            if !Roles::member_enrolled_in_doints(&new_message.member(ctx).await?) {
                return Ok(());
            }

            ActivityRewardHelper::reward_talking(new_message, data);
        }
        _ => {}
    }
    Ok(())
}
