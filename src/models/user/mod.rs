//! # User
//! A read-only module to read data from for a specific user.
//!
//! # Usage
//! ```no_run
//! use roux::User;
//! use roux::util::FeedOption;
//! #[cfg(feature = "async")]
//! use tokio;
//!
//! #[cfg_attr(feature = "async", tokio::main)]
//! #[maybe_async::maybe_async]
//! async fn main() {
//!     let user = User::new("kasuporo");
//!     // Now you are able to:
//!
//!     // Get overview
//!     let overview = user.overview(None).await;
//!
//!     // Get submitted posts.
//!     let submitted = user.submitted(None).await;
//!
//!     // Get comments.
//!     let comments = user.comments(None).await;
//! }
//! ```

extern crate serde_json;

use url::Url;

use crate::client::Client;
use crate::util::{defaults::default_client, url::JoinSegmentsExt};
use crate::util::{FeedOption, RouxError};

use crate::models::{About, Comments, Overview, Submissions};

/// User.
pub struct User {
    /// User's name.
    pub user: String,
    client: Client,
    base_url: Url,
}

impl User {
    /// Create a new `User` instance.
    pub fn new(user: &str) -> User {
        User {
            user: user.to_owned(),
            base_url: Url::parse("https://www.reddit.com/user/").unwrap(),
            client: default_client(),
        }
    }

    /// Get user's overview.
    #[maybe_async::maybe_async]
    pub async fn overview(&self, options: Option<FeedOption>) -> Result<Overview, RouxError> {
        let mut url = self
            .base_url
            .join_segments(&[&self.user, "overview", ".json"]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        let resp = self.client.get(url).send().await?;

        Ok(resp.json::<Overview>().await?)
    }

    /// Get user's submitted posts.
    #[maybe_async::maybe_async]
    pub async fn submitted(&self, options: Option<FeedOption>) -> Result<Submissions, RouxError> {
        let mut url = self
            .base_url
            .join_segments(&[&self.user, "submitted", ".json"]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<Submissions>()
            .await?)
    }

    /// Get user's submitted comments.
    #[maybe_async::maybe_async]
    pub async fn comments(&self, options: Option<FeedOption>) -> Result<Comments, RouxError> {
        let mut url = self
            .base_url
            .join_segments(&[&self.user, "comments", ".json"]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<Comments>()
            .await?)
    }

    /// Get user's about page
    #[maybe_async::maybe_async]
    pub async fn about(&self, options: Option<FeedOption>) -> Result<About, RouxError> {
        let mut url = self.base_url.join_segments(&[&self.user, "about", ".json"]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self.client.get(url).send().await?.json::<About>().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    use crate::util::FeedOption;

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_no_auth() {
        let user = User::new("beneater");

        // Test overview
        let overview = user.overview(None).await;
        assert!(overview.is_ok());

        // Test submitted
        let submitted = user.submitted(None).await;
        assert!(submitted.is_ok());

        // Test comments
        let comments = user.comments(None).await;
        assert!(comments.is_ok());

        // Test about
        let about = user.about(None).await;
        assert!(about.is_ok());

        // Test feed options
        let after = comments.unwrap().data.after.unwrap();
        let after_options = FeedOption::new().after(&after);
        let next_comments = user.comments(Some(after_options)).await;
        assert!(next_comments.is_ok());
    }
}
