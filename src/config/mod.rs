//! Configuration management for Scratchpad

pub mod loader;
mod schema;

pub use loader::{load_config, save_config};
pub use schema::*;
