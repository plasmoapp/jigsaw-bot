use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use url::Url;

pub struct User(pub UserId, pub UserData);

impl User {
    pub fn test() -> Self {
        Self(UserId(0), UserData::new("User".into(), None, None, None))
    }
}

#[derive(Shrinkwrap, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(u64);

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<Url>,
}

impl UserData {
    pub fn new(
        first_name: String,
        last_name: Option<String>,
        username: Option<String>,
        photo_url: Option<Url>,
    ) -> Self {
        Self {
            first_name,
            last_name,
            username,
            photo_url,
        }
    }
}
