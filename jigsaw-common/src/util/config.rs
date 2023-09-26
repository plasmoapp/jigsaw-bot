// use std::path::Path;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;

pub fn default_extract_config<'a, T: Deserialize<'a>>() -> Result<T, figment::Error> {
    Figment::new()
        .merge(Toml::file("Config.toml"))
        .merge(Env::prefixed("CONFIG."))
        .extract()
}
