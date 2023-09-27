// use base64::engine::general_purpose::STANDARD as ENGINE;
// use base64::Engine;
// use bytes::Bytes;

// use std::{collections::HashMap, sync::Arc};

// use axum::{
//     async_trait,
//     extract::{FromRef, FromRequestParts},
//     headers::{self, authorization::Credentials, Authorization},
//     http::{request::Parts, HeaderValue},
//     Extension, RequestPartsExt, TypedHeader,
// };
// use serde::{Deserialize, Serialize};
// use shrinkwraprs::Shrinkwrap;
// use url::Url;

// use crate::{config::Config, error::ReportResposnse};

// pub struct User(pub UserId, pub UserData);

// #[derive(Shrinkwrap, Serialize, Deserialize)]
// #[serde(transparent)]
// pub struct UserId(u64);

// #[derive(Serialize, Deserialize)]
// pub struct UserData {
//     pub first_name: String,
//     pub last_name: Option<String>,
//     pub username: Option<String>,
//     pub photo_url: Option<Url>,
// }

// #[derive(Shrinkwrap)]
// pub struct WebAppData {
//     data_check_string: String,
// }

// impl Credentials for WebAppData {
//     const SCHEME: &'static str = "WebAppData";

//     fn decode(value: &axum::http::HeaderValue) -> Option<Self> {
//         let bytes = &value.as_bytes()["WebAppData ".len()..];
//         let non_space_pos = bytes.iter().position(|b| *b != b' ')?;
//         let bytes = &bytes[non_space_pos..];

//         let bytes = ENGINE.decode(bytes).ok()?;

//         let data_check_string = String::from_utf8(bytes).ok()?;

//         Some(Self { data_check_string })
//     }

//     fn encode(&self) -> axum::http::HeaderValue {
//         let mut encoded = String::from("WebAppData ");
//         ENGINE.encode_string(&self.data_check_string, &mut encoded);
//         let bytes = Bytes::from(encoded);
//         HeaderValue::from_maybe_shared(bytes)
//             .expect("base64 encoding is always a valid HeaderValue")
//     }
// }

// impl WebAppData {
//     pub fn validate(&self, bot_token: &str) -> Option<HashMap<String, String>> {
//         let map = self
//             .data_check_string
//             .split('\n')
//             .flat_map(|str| str.split_once('='))
//             .collect::<HashMap<_, _>>();

//         dbg!(map);

//         todo!()
//     }
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for User
// where
//     // Extension<Arc<Config>>: FromRef<S>,
//     S: Send + Sync,
// {
//     type Rejection = ReportResposnse;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let Extension(config) = parts.extract::<Extension<Arc<Config>>>().await?;

//         let header = parts
//             .extract::<TypedHeader<Authorization<WebAppData>>>()
//             .await?
//             .0
//              .0;

//         header.validate("");

//         todo!()
//     }

//     // If anything goes wrong or no session is found, redirect to the auth page
// }
