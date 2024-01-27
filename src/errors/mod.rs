//! This module contains all the errors that can be returned by the library.

#[derive(thiserror::Error, Debug)]
pub enum VariantError {
    #[error("io experienced an error: {0}")]
    IO(String),
    #[error("shell command experienced an error: {0}")]
    Shell(String),
    #[error("internal error: {0}")]
    System(String),
}

impl From<std::io::Error> for VariantError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e.to_string())
    }
}

impl From<serde_json::Error> for VariantError {
    fn from(e: serde_json::Error) -> Self {
        Self::System(e.to_string())
    }
}
