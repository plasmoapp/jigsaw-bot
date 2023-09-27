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
    #[error("Invalid Header Data: {0}")]
    ToStr(#[from] ToStrError),
}
