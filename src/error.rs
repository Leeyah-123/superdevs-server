use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),

    #[error("Solana error")]
    SolanaError,

    #[error("Token error: {0}")]
    TokenError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Base58 decode error: {0}")]
    Base58DecodeError(#[from] bs58::decode::Error),

    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("Internal server error")]
    InternalError,
}

// Type alias for Result with ServerError
pub type Result<T> = std::result::Result<T, ServerError>;
