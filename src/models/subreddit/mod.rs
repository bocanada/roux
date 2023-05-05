//! # Subreddit
//! A read-only module to read data from a specific subreddit.
//!
//! # Basic Usage
//! ```no_run
//! use roux::Subreddit;
//! #[cfg(feature = "async")]
//! use tokio;
//!
//! #[cfg_attr(feature = "async", tokio::main)]
//! #[maybe_async::maybe_async]
//! async fn main() {
//!     let subreddit = Subreddit::new("rust");
//!     // Now you are able to:
//!
//!     // Get moderators.
//!     let moderators = subreddit.moderators().await;
//!
//!     // Get hot posts with limit = 25.
//!     let hot = subreddit.hot(25, None).await;
//!
//!     // Get rising posts with limit = 30.
//!     let rising = subreddit.rising(30, None).await;
//!
//!     // Get top posts with limit = 10.
//!     let top = subreddit.top(10, None).await;
//!
//!     // Get latest comments.
//!     // `depth` and `limit` are optional.
//!     let latest_comments = subreddit.latest_comments(None, Some(25)).await;
//!
//!     // Get comments from a submission.
//!     let article_id = &hot.unwrap().data.children.first().unwrap().data.id.clone();
//!     let article_comments = subreddit.article_comments(article_id, None, Some(25));
//! }
//! ```
//!
//! # Usage with feed options
//!
//! ```no_run
//! use roux::Subreddit;
//! use roux::util::{FeedOption, TimePeriod};
//! #[cfg(feature = "async")]
//! use tokio;
//!
//! #[cfg_attr(feature = "async", tokio::main)]
//! #[maybe_async::maybe_async]
//! async fn main() {
//!     let subreddit = Subreddit::new("astolfo");
//!
//!     // Gets top 10 posts from this month
//!     let options = FeedOption::new().period(TimePeriod::ThisMonth);
//!     let top = subreddit.top(25, Some(options)).await;
//!
//!     // Gets hot 10
//!     let hot = subreddit.hot(25, None).await;
//!
//!     // Get after param from `hot`
//!     let after = hot.unwrap().data.after.unwrap();
//!     let after_options = FeedOption::new().after(&after);
//!
//!     // Gets next 25
//!     let next_hot = subreddit.hot(25, Some(after_options)).await;
//! }
//! ```
pub mod response;
extern crate serde_json;

use reqwest::Url;

use crate::models::subreddit::response::{SubredditData, SubredditResponse, SubredditsData};

use crate::client::Client;
use crate::util::defaults::default_client;
use crate::util::{url::JoinSegmentsExt, FeedOption, RouxError};

use crate::models::{Comments, Moderators, Submissions};

/// Access subreddits API
pub struct Subreddits;

impl Subreddits {
    /// Search subreddits
    #[maybe_async::maybe_async]
    pub async fn search(
        name: &str,
        limit: Option<u32>,
        options: Option<FeedOption>,
    ) -> Result<SubredditsData, RouxError> {
        let mut url = Url::parse_with_params(
            "https://www.reddit.com/subreddits/search.json",
            &[("q", name), ("raw_json", "1")],
        )
        .unwrap();

        if let Some(limit) = limit {
            url.query_pairs_mut()
                .append_pair("limit", &limit.to_string());
        }

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        let client = default_client();

        Ok(client
            .get(url)
            .send()
            .await?
            .json::<SubredditsData>()
            .await?)
    }
}

/// Subreddit
pub struct Subreddit {
    /// Name of subreddit.
    pub name: String,
    url: Url,
    client: Client,
}

impl Subreddit {
    /// Create a new `Subreddit` instance.
    pub fn new(name: &str) -> Subreddit {
        let subreddit_url = Url::parse_with_params(
            &format!("https://www.reddit.com/r/{name}/"),
            &[("raw_json", "1")],
        )
        .unwrap();

        Subreddit {
            name: name.to_owned(),
            url: subreddit_url,
            client: default_client(),
        }
    }

    /// Create a new authenticated `Subreddit` instance using an oauth client
    /// from the `Reddit` module.
    pub fn new_oauth(name: &str, client: &Client) -> Subreddit {
        let subreddit_url = Url::parse_with_params(
            &format!("https://www.reddit.com/r/{name}/"),
            &[("raw_json", "1")],
        )
        .unwrap();

        Subreddit {
            name: name.to_owned(),
            url: subreddit_url,
            client: client.to_owned(),
        }
    }

    /// Get moderators (requires authentication)
    #[maybe_async::maybe_async]
    pub async fn moderators(&self) -> Result<Moderators, RouxError> {
        let url = self.url.join_segments(&["about", "moderators", ".json"]);
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<Moderators>()
            .await?)
    }

    /// Get subreddit data.
    #[maybe_async::maybe_async]
    pub async fn about(&self) -> Result<SubredditData, RouxError> {
        let url = self.url.join_segments(&["about", ".json"]);
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<SubredditResponse>()
            .await?
            .data)
    }

    #[maybe_async::maybe_async]
    async fn get_feed(
        &self,
        ty: &str,
        limit: u32,
        options: Option<FeedOption>,
    ) -> Result<Submissions, RouxError> {
        let mut url = self.url.join(&format!("{ty}.json")).unwrap();
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());

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

    #[maybe_async::maybe_async]
    async fn get_comment_feed(
        &self,
        ty: &str,
        depth: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Comments, RouxError> {
        let mut url = self.url.join(&format!("{ty}.json")).unwrap();

        if let Some(depth) = depth {
            url.query_pairs_mut()
                .append_pair("depth", &depth.to_string());
        }

        if let Some(limit) = limit {
            url.query_pairs_mut()
                .append_pair("limit", &limit.to_string());
        }

        let is_post_comments = url.path().contains("comments/");

        let resp = self.client.get(url).send().await?.error_for_status()?;

        // This is one of the dumbest APIs I've ever seen.
        // The comments for a subreddit are stored in a normal hash map
        // but for posts the comments are in an array with the ONLY item
        // being same hash map as the one for subreddits...
        if is_post_comments {
            let mut comments = resp.json::<Vec<Comments>>().await?;
            Ok(comments.pop().unwrap())
        } else {
            Ok(resp.json::<Comments>().await?)
        }
    }

    /// Get hot posts.
    #[maybe_async::maybe_async]
    pub async fn hot(
        &self,
        limit: u32,
        options: Option<FeedOption>,
    ) -> Result<Submissions, RouxError> {
        self.get_feed("hot", limit, options).await
    }

    /// Get rising posts.
    #[maybe_async::maybe_async]
    pub async fn rising(
        &self,
        limit: u32,
        options: Option<FeedOption>,
    ) -> Result<Submissions, RouxError> {
        self.get_feed("rising", limit, options).await
    }

    /// Get top posts.
    #[maybe_async::maybe_async]
    pub async fn top(
        &self,
        limit: u32,
        options: Option<FeedOption>,
    ) -> Result<Submissions, RouxError> {
        self.get_feed("top", limit, options).await
    }

    /// Get latest posts.
    #[maybe_async::maybe_async]
    pub async fn latest(
        &self,
        limit: u32,
        options: Option<FeedOption>,
    ) -> Result<Submissions, RouxError> {
        self.get_feed("new", limit, options).await
    }

    /// Get latest comments.
    #[maybe_async::maybe_async]
    pub async fn latest_comments(
        &self,
        depth: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Comments, RouxError> {
        self.get_comment_feed("comments", depth, limit).await
    }

    /// Get comments from article.
    #[maybe_async::maybe_async]
    pub async fn article_comments(
        &self,
        article: &str,
        depth: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Comments, RouxError> {
        self.get_comment_feed(&format!("comments/{}", article), depth, limit)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::Subreddit;
    use super::Subreddits;

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_no_auth() {
        let subreddit = Subreddit::new("astolfo");

        // Test feeds
        let hot = subreddit.hot(25, None).await;
        assert!(hot.is_ok());

        let rising = subreddit.rising(25, None).await;
        assert!(rising.is_ok());

        let top = subreddit.top(25, None).await;
        assert!(top.is_ok());

        let latest_comments = subreddit.latest_comments(None, Some(25)).await;
        assert!(latest_comments.is_ok());

        let article_id = &hot.unwrap().data.children.first().unwrap().data.id.clone();
        let article_comments = subreddit.article_comments(article_id, None, Some(25)).await;
        assert!(article_comments.is_ok());

        // Test subreddit data.
        let data_res = subreddit.about().await;
        assert!(data_res.is_ok());

        let data = data_res.unwrap();
        assert!(data.title == Some(String::from("Rider of Black, Astolfo")));
        assert!(data.subscribers.is_some());
        assert!(data.subscribers.unwrap() > 1000);

        // Test subreddit search
        let subreddits_limit = 3u32;
        let subreddits = Subreddits::search("rust", Some(subreddits_limit), None).await;
        assert!(subreddits.is_ok());
        assert!(subreddits.unwrap().data.children.len() == subreddits_limit as usize);
    }
}
