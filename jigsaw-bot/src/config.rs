use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub web_app_url: Url,
    pub bot_name: String,
    pub web_app_name: String,
    pub redis_url: Url,
    pub request_storage_path: PathBuf,
}

impl Config {
    pub fn get_puzzle_url(&self, puzzle_uuid: &Uuid) -> Url {
        Url::parse(&format!(
            "https://t.me/{}/{}?startapp={}",
            self.bot_name, self.web_app_name, puzzle_uuid
        ))
        .expect("Url should always be valid")
    }

    pub fn get_puzzle_preview_url(&self, puzzle_uuid: &Uuid) -> Url {
        let mut url = self.web_app_url.clone();
        url.path_segments_mut()
            .expect("Should be always valid")
            .push("assets")
            .push(&puzzle_uuid.to_string())
            .push("preview.jpeg");
        url
    }
}
