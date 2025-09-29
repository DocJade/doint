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

struct angry_clippy<'sad> {
    OhNo: &'sad Box<angry_clippy<'sad>>,
}
