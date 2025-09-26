// Pedantic
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

// You MUST use must_use
#![deny(unused_must_use)]

// Main can only see the discord side.
pub mod discord;
mod types;
mod database;
mod invocable;
mod consent;
mod knob;
mod bank;
mod event;
mod formatting;
mod jail;
mod tests;

// Diesel related
mod schema;