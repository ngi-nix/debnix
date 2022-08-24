use thiserror::Error;

#[derive(Error, Debug)]
pub enum DebNixError {
    // Io Error
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    // Deserialization Error
    #[error("Deserialization Error: {0}")]
    Serde(#[from] serde_json::Error),
}
