#![deny(missing_docs)]

//! # roux.rs
//! This crate provides simple access to the Reddit API.
//!
//! ## Using OAuth
//! To create an OAuth client with the reddit API, use the `Reddit` class.
//! ```no_run
//! use roux::Reddit;
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() {
//! let client = Reddit::new("USER_AGENT", "CLIENT_ID", "CLIENT_SECRET")
//!     .username("USERNAME")
//!     .password("PASSWORD")
//!     .login()
//!     .await;
//! let me = client.unwrap();
//! }
//! ```
//!
//! It is important that you pick a good user agent. The ideal format is
//! `platform:program:version (by /u/yourname)`, e.g. `macos:roux:v0.3.0 (by /u/beanpup_py)`.
//!
//! This will authticate you as the user given in the username function.
//!
//!
//! ## Usage
//! Using the OAuth client, you can:
//!
//! ### Submit A Text Post
//! ```no_run
//! use roux::Reddit;
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() {
//! let client = Reddit::new("USER_AGENT", "CLIENT_ID", "CLIENT_SECRET")
//!     .username("USERNAME")
//!     .password("PASSWORD")
//!     .login()
//!     .await;
//! let me = client.unwrap();
//!
//! me.submit_text("TEXT_TITLE", "TEXT_BODY", "SUBREDDIT");
//! }
//! ```
//!
//! ### Submit A Link Post
//! ```no_run
//! use roux::Reddit;
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() {
//! let client = Reddit::new("USER_AGENT", "CLIENT_ID", "CLIENT_SECRET")
//!     .username("USERNAME")
//!     .password("PASSWORD")
//!     .login()
//!     .await;
//! let me = client.unwrap();
//!
//! me.submit_link("LINK_TITLE", "LINK", "SUBREDDIT");
//! }
//! ```

use serde::Deserialize;

use reqwest::{header, IntoUrl, RequestBuilder};

mod config;

mod client;
use client::Client;

mod models;
pub use models::*;

/// Utils for requests.
pub mod util;
use util::url;

/// Client to use OAuth with Reddit.
pub struct Reddit {
    client: RedditClient,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum AuthResponse {
    AuthData { access_token: String },
    ErrorData { error: String },
}

impl Reddit {
    /// Creates a `Reddit` instance with user_agent, client_id, and client_secret.
    pub fn new(user_agent: &str, client_id: &str, client_secret: &str) -> Reddit {
        Reddit {
            client: RedditClient::new(
                Client::new(),
                config::Config::new(user_agent, client_id, client_secret),
            ),
        }
    }

    /// Sets the internal `reqwest::Client` to make requests with.
    pub fn with_client(mut self, client: Client) -> Reddit {
        self.client.inner = client;
        self
    }

    /// Sets username.
    pub fn username(mut self, username: &str) -> Reddit {
        self.client.cfg.username = Some(username.to_owned());
        self
    }

    /// Sets password.
    pub fn password(mut self, password: &str) -> Reddit {
        self.client.cfg.password = Some(password.to_owned());
        self
    }

    async fn create_client(mut self) -> Result<Reddit, util::RouxError> {
        self.client = self.client.login().await?;
        Ok(self)
    }

    /// Login as a user.
    pub async fn login(self) -> Result<me::Me, util::RouxError> {
        let reddit = self.create_client().await?;
        Ok(me::Me::new(&reddit.client))
    }

    /// Create a new authenticated `Subreddit` instance.
    pub async fn subreddit(self, name: &str) -> Result<models::Subreddit, util::RouxError> {
        let reddit = self.create_client().await?;
        Ok(models::Subreddit::new_oauth(name, &reddit.client))
    }

    /// Login the app.
    pub async fn client_login(self) -> Result<Self, util::RouxError> {
        self.create_client().await
    }

    /// Create a new authenticated `Subreddit` instance.
    /// This allows you to re-use the same `Reddit` instance over multiple `Subreddit`.
    pub async fn auth_subreddit(&self, name: &str) -> Result<models::Subreddit, util::RouxError> {
        Ok(models::Subreddit::new_oauth(name, &self.client))
    }
}

/// `RedditClient` wraps `reqwest::Client` to refresh tokens automatically.
#[derive(Clone)]
pub struct RedditClient {
    inner: Client,

    cfg: config::Config,
}

impl RedditClient {
    fn new(client: Client, cfg: config::Config) -> Self {
        Self { inner: client, cfg }
    }

    fn get(&self, url: impl IntoUrl) -> RequestBuilder {
        let user_agent = header::HeaderValue::from_str(&self.cfg.user_agent).unwrap();
        let mut builder = self.inner.get(url).header(header::USER_AGENT, user_agent);

        if let Some(ref token) = self.cfg.access_token {
            builder = builder.bearer_auth(token);
        }

        builder
    }

    fn post(&self, url: impl IntoUrl) -> RequestBuilder {
        let user_agent = header::HeaderValue::from_str(&self.cfg.user_agent).unwrap();
        let mut builder = self.inner.post(url).header(header::USER_AGENT, user_agent);

        if let Some(ref token) = self.cfg.access_token {
            builder = builder.bearer_auth(token);
        }

        builder
    }

    async fn login(mut self) -> Result<RedditClient, util::RouxError> {
        let url = &url::build_url("api/v1/access_token")[..];
        let form = [
            ("grant_type", "password"),
            ("username", &self.cfg.username.to_owned().unwrap()),
            ("password", &self.cfg.password.to_owned().unwrap()),
        ];

        let request = self
            .post(url)
            .basic_auth(&self.cfg.client_id, Some(&self.cfg.client_secret))
            .form(&form);

        let response = request.send().await?;

        if response.status() == 200 {
            let auth_data = response.json::<AuthResponse>().await?;

            let access_token = match auth_data {
                AuthResponse::AuthData { access_token, .. } => access_token,
                AuthResponse::ErrorData { error } => return Err(util::RouxError::Auth(error)),
            };

            self.cfg.access_token = Some(access_token);
            Ok(self)
        } else {
            Err(util::RouxError::Status(response))
        }
    }
}
