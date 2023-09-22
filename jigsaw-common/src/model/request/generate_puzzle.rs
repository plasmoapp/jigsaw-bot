use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct GeneratePuzzleRequest {
    pub uuid: Uuid,
    pub image_path: PathBuf,
}
