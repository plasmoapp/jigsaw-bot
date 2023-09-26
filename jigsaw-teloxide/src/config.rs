use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub web_app_url: Url,
    pub redis_url: Url,
    pub request_storage_path: PathBuf,
}
