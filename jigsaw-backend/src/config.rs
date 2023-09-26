use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub redis_url: Url,
    pub complete_storage_path: PathBuf,
    // pub request_storage_path: PathBuf,
}
