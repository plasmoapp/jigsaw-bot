use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub redis_url: Url,
    pub complete_storage_path: PathBuf,
    pub bot_token: String,
    // pub request_storage_path: PathBuf,
}

impl Config {
    pub fn get_telegram_web_secret(&self) -> ring::hmac::Key {
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, b"WebAppData");
        let tag = ring::hmac::sign(&key, self.bot_token.as_ref());
        ring::hmac::Key::new(ring::hmac::HMAC_SHA256, tag.as_ref())
    }
}
