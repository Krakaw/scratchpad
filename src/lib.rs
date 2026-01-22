//! Scratchpad - Deploy scratch environments easily
//!
//! This is the library interface for Scratchpad, allowing programmatic
//! access to scratch environment management.

pub mod api;
pub mod cli;
pub mod config;
pub mod docker;
pub mod error;
pub mod nginx;
pub mod scratch;
pub mod services;
pub mod ui;

pub use config::Config;
pub use error::Error;
pub use scratch::Scratch;
