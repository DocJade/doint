// Force run taxes

use diesel::{Connection, RunQueryDsl};
use poise::CreateReply;

use crate::bank::bank_struct::BankInterface;
use crate::database::tables::bank::BankInfo;
use crate::guards;
use crate::schema::bank::dsl::bank;
use crate::types::serenity_types::{Context, Error};

/// Forcibly collect taxes immediately.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR", // Only admins can run/see this command.
    check = guards::in_doints_category
    )
]
pub(crate) async fn admin_tax_now(ctx: Context<'_>) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Do the taxes.
    let collected = BankInterface::collect_taxes(&mut conn)?;

    // Assemble a response
    let response = CreateReply::default()
        .ephemeral(true)
        .content(format!("Taxation collected {collected} doints."));

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}

/// Get all information about the bank.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR", // Only admins can run/see this command.
    check = guards::in_doints_category
    )
]
pub(crate) async fn admin_bank_info(ctx: Context<'_>) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Read in the bank row
    let bank_info: BankInfo = conn.transaction(|conn| bank.first(conn))?;

    // deconstruct it and print it nicely.
    let BankInfo {
        doints_on_hand,
        total_doints,
        tax_rate,
        ..
    } = bank_info;

    // format the tax rate better
    let formatted_tax_rate: String = format!("{:.1}% [{tax_rate}]", f64::from(tax_rate) / 10.0);

    let response_text: String = format!(
        "Bank:\
        \n- Doints in bank: {doints_on_hand}\
        \n- Doints in circulation: {total_doints}\
        \n- Current tax rate {formatted_tax_rate}\
        "
    );

    // Assemble a response
    let response = CreateReply::default()
        .ephemeral(true)
        .content(response_text);

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}

/// Set the bank tax rate.
///
/// This may be automatically overridden later by other tax calculations.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR", // Only admins can run/see this command.
    check = guards::in_doints_category
    )
]
pub(crate) async fn admin_set_tax_rate(
    ctx: Context<'_>,
    #[description = "The new tax rate. Needs to be between 0 and 1000 inclusive."] new_rate: u16,
) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Change the tax rate directly.
    // If user provides a bad rate, it'll fail.
    let was_set = BankInterface::set_tax_rate(&mut conn, new_rate);

    let response_text = if was_set {
        "Rate set."
    } else {
        "Failed to set rate."
    };

    // Assemble a response
    let response = CreateReply::default()
        .ephemeral(true)
        .content(response_text);

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}

/// Set the universal income payout rate.
///
/// This may be automatically overridden later by other tax calculations.
#[poise::command(slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR", // Only admins can run/see this command.
    check = guards::in_doints_category
    )
]
pub(crate) async fn admin_set_ubi_rate(
    ctx: Context<'_>,
    #[description = "The new UBI rate. Needs to be between 0 and 1000 inclusive."] new_rate: u16,
) -> Result<(), Error> {
    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // Change the tax rate directly.
    // If user provides a bad rate, it'll fail.
    let was_set = BankInterface::set_ubi_rate(&mut conn, new_rate);

    let response_text = if was_set {
        "Rate set."
    } else {
        "Failed to set rate."
    };

    // Assemble a response
    let response = CreateReply::default()
        .ephemeral(true)
        .content(response_text);

    // Send it.
    let _ = ctx.send(response).await?;
    Ok(())
}
