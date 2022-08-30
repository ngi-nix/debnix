use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
/// The debnix error type
pub enum DebNixError {
    /// Io Error
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    /// Deserialization Error
    #[error("Deserialization Error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Utf8 Conversion Error")]
    Utf8(#[from] Utf8Error),
    /// Reqwest Error
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("DebControl Error")]
    DebControl(String),
    #[error("Nix Error")]
    Nix(String),
    #[error("Nothing to Match: {0}")]
    NoMatches(String),
}
