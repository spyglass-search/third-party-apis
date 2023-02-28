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
    id: String,
    name: String,
}
