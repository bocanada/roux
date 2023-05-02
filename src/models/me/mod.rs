//! # Me
//! Me module.

pub mod response;

extern crate reqwest;
extern crate serde_json;
use ::url::Url;
use serde::Serialize;

use crate::client::{Client, Response};
use crate::config::Config;
use crate::models::me::response::MeData;
use crate::models::{Friend, Inbox, Saved};
use crate::util::url::JoinSegmentsExt;
use crate::util::{url, FeedOption, RouxError};

/// Me
#[derive(Debug, Clone)]
pub struct Me {
    /// Config
    pub config: Config,
    /// Client
    pub client: Client,
    base_url: Url,
}

impl Me {
    /// Create a new `me`
    pub fn new(config: &Config, client: &Client) -> Me {
        Me {
            config: config.to_owned(),
            client: client.to_owned(),
            base_url: Url::parse("https://oauth.reddit.com/").unwrap(),
        }
    }

    #[maybe_async::maybe_async]
    async fn get(&self, url: Url) -> Result<Response, RouxError> {
        Ok(self.client.get(url).send().await?)
    }

    #[maybe_async::maybe_async]
    async fn post<T: Serialize>(&self, url: &str, form: T) -> Result<Response, RouxError> {
        let post_url = url::build_oauth(url).to_owned();

        match self.client.post(&post_url[..]).form(&form).send().await {
            Ok(response) => Ok(response),
            Err(e) => Err(e.into()),
        }
    }

    /// Get me
    #[maybe_async::maybe_async]
    pub async fn me(&self) -> Result<MeData, RouxError> {
        let url = self.base_url.join_segments(&["api", "v1", "me"]);
        match self.get(url).await {
            Ok(res) => Ok(res.json::<MeData>().await?),
            Err(e) => Err(e),
        }
    }

    /// Submit link
    #[maybe_async::maybe_async]
    pub async fn submit_link(
        &self,
        title: &str,
        link: &str,
        sr: &str,
    ) -> Result<Response, RouxError> {
        let form = [
            ("kind", "link"),
            ("title", title),
            ("url", link),
            ("sr", sr),
        ];

        self.post("api/submit", &form).await
    }

    /// Submit text
    #[maybe_async::maybe_async]
    pub async fn submit_text(
        &self,
        title: &str,
        text: &str,
        sr: &str,
    ) -> Result<Response, RouxError> {
        let form = [
            ("kind", "self"),
            ("title", title),
            ("text", text),
            ("sr", sr),
        ];

        self.post("api/submit", &form).await
    }

    /// Submit richtext
    #[maybe_async::maybe_async]
    pub async fn submit_richtext(
        &self,
        title: &str,
        richtext: &str,
        sr: &str,
    ) -> Result<Response, RouxError> {
        let form = [
            ("kind", "self"),
            ("title", title),
            ("richtext_json", richtext),
            ("sr", sr),
        ];

        self.post("api/submit", &form).await
    }

    /// Adds a friend to a subreddit with the specified type
    #[maybe_async::maybe_async]
    pub async fn add_subreddit_friend(
        &self,
        username: &str,
        typ: &str,
        sub: &str,
    ) -> Result<bool, RouxError> {
        let form = [("name", username), ("type", typ)];
        Ok(self
            .post(format!("r/{}/api/friend", sub).as_str(), form)
            .await?
            .json::<Friend>()
            .await?
            .success)
    }

    /// Removes a friend to a subreddit with the specified type
    #[maybe_async::maybe_async]
    pub async fn remove_subreddit_friend(
        &self,
        username: &str,
        typ: &str,
        sub: &str,
    ) -> Result<bool, RouxError> {
        let form = [("name", username), ("type", typ)];
        Ok(self
            .post(format!("r/{}/api/unfriend", sub).as_str(), form)
            .await?
            .json::<Friend>()
            .await?
            .success)
    }

    /// Compose message
    #[maybe_async::maybe_async]
    pub async fn compose_message(
        &self,
        username: &str,
        subject: &str,
        body: &str,
    ) -> Result<Response, RouxError> {
        let form = [
            ("api_type", "json"),
            ("subject", subject),
            ("text", body),
            ("to", username),
        ];

        self.post("api/compose", &form).await
    }

    /// Get user's submitted posts.
    #[maybe_async::maybe_async]
    pub async fn inbox(&self) -> Result<Inbox, RouxError> {
        let url = self.base_url.join_segments(&["message", "inbox"]);
        Ok(self.get(url).await?.json::<Inbox>().await?)
    }

    /// Get saved
    #[maybe_async::maybe_async]
    pub async fn saved(&self, options: Option<FeedOption>) -> Result<Saved, RouxError> {
        let mut url = self.base_url.join_segments(&[
            "user",
            self.config.username.as_ref().unwrap(),
            "saved",
            ".json",
        ]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self.get(url).await?.json::<Saved>().await?)
    }

    /// Get upvoted
    #[maybe_async::maybe_async]
    pub async fn upvoted(&self, options: Option<FeedOption>) -> Result<Saved, RouxError> {
        let mut url = self.base_url.join_segments(&[
            "user",
            self.config.username.as_ref().unwrap(),
            "upvoted",
            ".json",
        ]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self.get(url).await?.json::<Saved>().await?)
    }

    /// Get downvoted
    #[maybe_async::maybe_async]
    pub async fn downvoted(&self, options: Option<FeedOption>) -> Result<Saved, RouxError> {
        let mut url = self.base_url.join_segments(&[
            "user",
            self.config.username.as_ref().unwrap(),
            "downvoted",
            ".json",
        ]);

        if let Some(options) = options {
            options.build_url(&mut url);
        }

        Ok(self.get(url).await?.json::<Saved>().await?)
    }

    /// Get users unread messages
    #[maybe_async::maybe_async]
    pub async fn unread(&self) -> Result<Inbox, RouxError> {
        let url = self.base_url.join_segments(&["message", "unread"]);
        Ok(self.get(url).await?.json::<Inbox>().await?)
    }

    /// Mark messages as read
    #[maybe_async::maybe_async]
    pub async fn mark_read(&self, ids: &str) -> Result<Response, RouxError> {
        let form = [("id", ids)];
        self.post("api/read_message", &form).await
    }

    /// Mark messages as unread
    #[maybe_async::maybe_async]
    pub async fn mark_unread(&self, ids: &str) -> Result<Response, RouxError> {
        let form = [("id", ids)];
        self.post("api/unread_message", &form).await
    }

    /// Comment
    #[maybe_async::maybe_async]
    pub async fn comment(&self, text: &str, parent: &str) -> Result<Response, RouxError> {
        let form = [("text", text), ("parent", parent)];
        self.post("api/comment", &form).await
    }

    /// Edit a 'thing'
    #[maybe_async::maybe_async]
    pub async fn edit(&self, text: &str, parent: &str) -> Result<Response, RouxError> {
        let form = [("text", text), ("thing_id", parent)];
        self.post("api/editusertext", &form).await
    }

    /// Logout
    #[maybe_async::maybe_async]
    pub async fn logout(self) -> Result<(), RouxError> {
        let url = "https://www.reddit.com/api/v1/revoke_token";

        let form = [("access_token", self.config.access_token.to_owned())];

        let response = self
            .client
            .post(url)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&form)
            .send()
            .await?;

        if response.status() == 204 {
            Ok(())
        } else {
            Err(RouxError::Status(response))
        }
    }
}
