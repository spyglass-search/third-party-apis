use serde::{Deserialize, Serialize};
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
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Post {
    pub subreddit: String,
    pub author: String,
    pub title: String,
    /// default/self for no thumbnail
    /// Otherwise a URL to the thumbnail
    pub thumbnail: String,
    pub post_hint: String,
    pub permalink: String,
    pub selftext: String,
    pub url: String,

    /// Only available on comments
    pub body: Option<String>,

    pub num_comments: i32,
    pub score: i32,

    pub created_utc: f32,
    pub saved: bool,
    pub is_self: bool,
    pub is_video: bool,
    pub media_only: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PostData {
    pub kind: String,
    pub data: Post
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PostListingData {
    pub after: Option<String>,
    pub dist: i32,
    pub children: Vec<PostData>
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PostListing {
    pub kind: String,
    pub data: PostListingData
}