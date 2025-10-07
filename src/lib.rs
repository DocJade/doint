#![allow(clippy::result_large_err)]

// Main can only see the discord side.
mod bank;
mod consent;
mod database;
pub mod discord;
mod event;
mod formatting;
mod guards;
mod invocable;
mod jail;
mod knob;
mod tests;
mod types;

// Diesel related
mod schema;
