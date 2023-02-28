use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use strum_macros::{Display, EnumString};

/// Reddit scopes taken from: https://github.com/reddit-archive/reddit/wiki/OAuth2
/// We only include the ones we're interested in.
#[derive(Debug, Display, EnumString)]
pub enum AuthScopes {
    /// Grants read access to comments, down/upvotes, saved, submitted posts.
    #[strum(serialize = "history")]
    History,
    /// Access to reddit username & signup date.
    #[strum(serialize = "identity")]
    Identity,
    /// Grant read access to list of subreddits the user moderates, contributes to, subscribes to.
    #[strum(serialize = "mysubreddits")]
    MySubreddits,
    /// Grant read access to post informatoin.
    #[strum(serialize = "read")]
    Read,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct User {
    pub id: String,
    pub name: String,
}

/// Converts a Reddit UTC timestamp in seconds to chrono::DateTime<Utc>
fn from_utc_secs<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let f: f64 = Deserialize::deserialize(deserializer)?;
    if let Some(datetime) = NaiveDateTime::from_timestamp_millis((f as i64) * 1000) {
        return Ok(DateTime::from_utc(datetime, Utc));
    }

    Err(D::Error::custom("Unable to deserialize time"))
}

pub struct ApiResponse<T> {
    pub after: Option<String>,
    pub data: T,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Post {
    /// Unique identifier for this post/comment/etc. Used in after/before pagination.
    pub name: String,
    /// Subreddit this post/comment/etc. was found in.
    pub subreddit: String,
    /// Author of the post/comment
    pub author: String,
    /// Title of the post, comment will be empty.
    pub title: Option<String>,
    /// default/self for no thumbnail
    /// Otherwise a URL to the thumbnail
    pub thumbnail: String,
    pub post_hint: String,
    pub permalink: String,
    pub selftext: String,
    pub url: String,

    pub num_comments: i32,
    pub score: i32,

    #[serde(deserialize_with = "from_utc_secs")]
    pub created_utc: DateTime<Utc>,
    pub saved: bool,
    pub is_self: bool,
    pub is_video: bool,
    pub media_only: bool,

    // Only available on comments
    /// Plain-text body of the comment.
    pub body: Option<String>,
    /// Link title the comment is under.
    pub link_title: Option<String>,
}

/// Types are documented here: https://www.reddit.com/dev/api/oauth#fullnames
/// under "type prefixes"
#[derive(Clone, Debug, Display, EnumString)]
pub enum DataType {
    #[strum(serialize = "t1")]
    Comment,
    #[strum(serialize = "t2")]
    Account,
    #[strum(serialize = "t3")]
    Link,
    #[strum(serialize = "t4")]
    Message,
    #[strum(serialize = "t5")]
    Subreddit,
    #[strum(serialize = "t6")]
    Award,
    #[strum(serialize = "")]
    Unknown,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Unknown
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DataWrapper<T> {
    pub kind: String,
    pub data: T,
}

impl<T> DataWrapper<T> {
    pub fn data_type(&self) -> DataType {
        DataType::from_str(&self.kind).unwrap_or_default()
    }
}

/// A "listing"
/// https://www.reddit.com/dev/api/oauth#listings
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Listing<T> {
    pub after: Option<String>,
    pub dist: i32,
    pub children: Vec<T>,
}
