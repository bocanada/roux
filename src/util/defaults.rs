use crate::{client::ClientBuilder, config, RedditClient};

/// String function for serde defaults.
pub fn default_string() -> String {
    "".to_string()
}

/// Default headers must contain user agent
// pub fn default_headers() -> HeaderMap {
//     vec![(USER_AGENT, HeaderValue::from_static("roux/rust"))]
//         .into_iter()
//         .collect()
// }

/// Default client
pub fn default_client() -> RedditClient {
    let c = ClientBuilder::new()
        .build()
        .expect("Error creating default client ");
    RedditClient::new(c, config::Config::new("roux/rust", "", ""))
}
