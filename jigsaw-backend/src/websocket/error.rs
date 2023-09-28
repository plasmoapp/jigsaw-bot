use axum::http::header::ToStrError;

use super::handler::SocketHandler;

#[derive(thiserror::Error, Debug)]
pub enum SocketAuthError {
    #[error(
        "No protocol specified. Please specify a protocol. Supported protocoles: {:#?}",
        SocketHandler::PROTOCOLS
    )]
    NoProtocol,
    #[error(
        "Protocol '{0}' is not supported. Supported protocoles: {:#?}",
        SocketHandler::PROTOCOLS
    )]
    UnsupportedProtocol(Box<str>),
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Invalid request. Expected: {0}")]
    InvalidRequest(Box<str>),
    #[error("Invalid Header Data: {0}")]
    ToStr(#[from] ToStrError),
    #[error("Socket Closed")]
    SocketClosed,
    #[error("Axum Error: {0}")]
    Axum(#[from] axum::Error),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
}
