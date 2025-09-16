// Pedantic
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

// Main can only see the discord side.
pub mod discord;
mod types;
mod database;
mod invokable;

// Diesel related
mod schema;