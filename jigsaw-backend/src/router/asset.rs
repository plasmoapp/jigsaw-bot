use std::sync::Arc;

use axum::{extract::Path, Extension};
use uuid::Uuid;

use crate::{config::Config, error::ReportResposnse};

// pub async fn puzzle_asset_handle(
//     Path((puzzle_uuid, image)): Path<(Uuid, String)>,
//     // Extension(config): Extension<Arc<Config>>,
// ) -> Result<(), ReportResposnse> {
//     // let mut path = config.complete_storage_path.clone();
//     // path.push(puzzle_uuid.to_string());
//     // path.push(image);

//     todo!()
// }
