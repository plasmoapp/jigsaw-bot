pub mod config;
pub mod error;
pub mod model;
pub mod websocket;


use eyre::{Report};

use axum::{
    Router,
};


use std::{net::SocketAddr};
use tower_http::services::{ServeDir};

use crate::{
    websocket::{route::get_puzzle_websocket, state::AppState},
};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    let state = AppState::new().await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], state.config.port));

    let serve_dir_assets = ServeDir::new(&state.config.complete_storage_path);
    let serve_dir_public = ServeDir::new("public").append_index_html_on_directories(true);

    let router = Router::new()
        .route(
            "/api/puzzle/:puzzle_uuid/websocket",
            axum::routing::get(get_puzzle_websocket),
        )
        .nest_service("/assets", serve_dir_assets)
        .fallback_service(serve_dir_public)
        .with_state(state);

    tracing::debug!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
