// Pedantic
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// You MUST use must_use
#![deny(unused_must_use)]

// Main can only see the discord side.
mod bank;
mod consent;
mod database;
pub mod discord;
mod event;
mod formatting;
mod invocable;
mod jail;
mod knob;
mod tests;
mod types;

// Diesel related
mod schema;
