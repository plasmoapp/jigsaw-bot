use std::path::PathBuf;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub redis_url: Url,
    pub complete_storage_path: PathBuf,
    pub request_storage_path: PathBuf,
}

impl Config {
    pub fn extract() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("Config.toml"))
            .merge(Env::prefixed("APP."))
            .extract()
    }
}
