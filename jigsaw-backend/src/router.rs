use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

use crate::{config::Config, error::ReportResposnse};

// use self::api::puzzle::post_place;
use self::ws::get_ws;

pub mod api;
pub mod asset;
pub mod ws;

pub fn router(config: &Config) -> Router {
    let serve_dir = ServeDir::new(&config.complete_storage_path);
    let serve_dir_public = ServeDir::new("public").append_index_html_on_directories(true);
    // let index_html = ServeFile::new("public/index.html");

    Router::new()
        // .route("/", get(root))
        // .route("/api/puzzle/:puzzle_uuid", get(get_puzzle_state))
        // .route(
        //     "/api/puzzle/:puzzle_uuid/tile/:tile_uuid/place",
        //     post(post_place),
        // )
        .route("/api/puzzle/:puzzle_uuid/websocket", get(get_ws))
        // .roter("/asset/:puzzle_uuid/:image", get(puzzle_asset_handle))
        .nest_service("/assets", serve_dir)
        // .nest_service("/", index_html)
        // .nest_service("/", serve_file)
        .fallback_service(serve_dir_public)
}

// async fn root() -> Result<&'static str, ReportResposnse> {
//     // Ok("Hello, World!")

//     // Err(eyre::eyre!("Pog"))?;

//     Ok("hello world")
// }
