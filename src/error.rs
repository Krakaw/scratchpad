//! Error types for Scratchpad

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Scratch '{0}' not found")]
    ScratchNotFound(String),

    #[error("Scratch '{0}' already exists")]
    ScratchAlreadyExists(String),

    #[error("Service '{0}' not found")]
    ServiceNotFound(String),

    #[error("Invalid scratch name: {0}")]
    InvalidScratchName(String),

    #[error("Config file not found. Run 'scratchpad init' first.")]
    ConfigNotFound,

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
