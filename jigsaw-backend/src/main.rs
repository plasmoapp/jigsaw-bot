#[macro_use]
extern crate log;

pub mod auth;
pub mod config;
pub mod error;
pub mod model;
pub mod router;
pub mod websocket;
pub mod ws_state;

use error::ReportResposnse;
use eyre::{bail, Report};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use jigsaw_common::util::config::default_extract_config;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};

use crate::{config::Config, router::router, websocket::state::WebSocketState};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;
    env_logger::init();

    let config = default_extract_config::<Config>()?;

    let redis_connection = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_tokio_connection()
        .await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let ws_state = Arc::new(WebSocketState::default());

    let router = router(&config)
        .layer(Extension(redis_connection))
        .layer(Extension(ws_state));

    // .layer(Extension(Arc::new(config)));

    info!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
