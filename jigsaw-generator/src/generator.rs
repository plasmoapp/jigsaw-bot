use eyre::Report;
use jigsaw_common::model::{puzzle::JigsawPuzzle, request::generate_puzzle::GeneratePuzzleRequest};
use path_macro::path;

use crate::{
    config::Config,
    jigsaw::RawJigsawPuzzle,
    storage::{JigsawImageStorage, JigsawStateStorage, JigsawStorage},
};

pub struct JigsawGenerator<I, S>
where
    I: JigsawImageStorage + Sync + Send,
    S: JigsawStateStorage + Sync + Send,
{
    pub config: Config,
    pub storage: JigsawStorage<I, S>,
}

impl<I, S> JigsawGenerator<I, S>
where
    I: JigsawImageStorage + Sync + Send,
    S: JigsawStateStorage + Sync + Send,
{
    pub fn new(config: Config, storage: JigsawStorage<I, S>) -> Self {
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
