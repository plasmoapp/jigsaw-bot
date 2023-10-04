use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use url::Url;

#[derive(Clone)]
pub struct User(pub UserId, pub UserData);

impl User {
    pub fn test() -> Self {
        Self(UserId(0), UserData::new("User".into()))
    }
}

#[derive(Shrinkwrap, Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
pub struct UserId(u64);

#[derive(Serialize, Deserialize, Clone)]
pub struct UserData {
    pub name: String,
    // pub last_name: Option<String>,
    // pub username: Option<String>,
    // pub _url: Option<Url>,
}

impl UserData {
    pub fn new(
        name: String,
        // last_name: Option<String>,
        // username: Option<String>,
        // photo_url: Option<Url>,
    ) -> Self {
        Self {
            name, // first_name,
                  // last_namce,
                  // username,
                  // photo_url,
        }
    }
}

#[derive(Deserialize)]
pub struct TelegramUser {
    pub id: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<Url>,
}

impl From<TelegramUser> for User {
    fn from(value: TelegramUser) -> Self {
        Self(
            UserId(value.id),
            UserData {
                name: value.first_name, // first_name: value.first_name,
                                        // last_name: value.last_name,
                                        // username: value.username,
                                        // photo_url: value.photo_url,
            },
        )
    }
}
