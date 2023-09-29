use std::string::FromUtf8Error;

use axum::http::header::ToStrError;

use super::unauthorized_handler::PROTOCOLS;

#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    #[error(
        "No protocol specified. Please specify a protocol. Supported protocoles: {:#?}",
        PROTOCOLS
    )]
    NoProtocol,
    #[error(
        "Protocol '{0}' is not supported. Supported protocoles: {:#?}",
        PROTOCOLS
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
    #[error("Failed to UTF8:")]
    FromUTF8(#[from] FromUtf8Error),
}
