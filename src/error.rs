use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CliError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Configuration failed: {0}")]
    Configuration(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Local node info not received during configuration")]
    NoLocalNodeInfo,

    #[error("Device disconnected unexpectedly")]
    Disconnected,

    #[error(transparent)]
    Meshtastic(#[from] meshtastic::errors::Error),

    #[error("Serial error: {0}")]
    Serial(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}
