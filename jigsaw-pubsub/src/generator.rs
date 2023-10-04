use eyre::Report;
use jigsaw_common::model::{puzzle::JigsawPuzzle, request::generate_puzzle::GeneratePuzzleRequest};
use path_macro::path;

use crate::{
    config::Config,
    jigsaw::RawJigsawPuzzle,
    storage::{JigsawImageStorage, JigsawStorage},
};

pub struct JigsawGenerator {
    pub config: Config,
    pub storage: JigsawStorage,
}

impl JigsawGenerator {
    pub fn new(config: Config, storage: JigsawStorage) -> Self {
        Self { config, storage }
    }

    pub async fn generate_from_request(
        &self,
        request: &GeneratePuzzleRequest,
    ) -> Result<JigsawPuzzle, Report> {
        let image_path = path!(self.config.request_storage_path / request.image_path);
        let image = image::io::Reader::open(image_path)?.decode()?;

        let raw_puzzle = RawJigsawPuzzle::try_from_image(image)?;
        let puzzle = self.storage.store(raw_puzzle).await?;

        Ok(puzzle)
    }
}
