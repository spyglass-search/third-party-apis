use chrono::{DateTime, Utc};
use markdown::{CompileOptions, Options};
use scraper::Html;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Github scopes taken from: https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps
#[derive(Debug, Display, EnumString)]
pub enum AuthScopes {
    /// Full access to repo
    #[strum(serialize = "repo")]
    Repo,
    /// Grants read/write access to commit statuses in public/private repos
    #[strum(serialize = "repo:status")]
    RepoStatus,
    /// Grants access to deployment statuses for public/private repos.
    #[strum(serialize = "repo_deployment")]
    RepoDeployment,
    /// Limits access to public repos. Read/write access to code, commit statuses,
    /// projects, collaborators, and deployment statuses. Also required for starring
    /// public repos.
    #[strum(serialize = "public_repo")]
    PublicRepo,
    /// Grants read/write access to profile info only.
    #[strum(serialize = "user")]
    User,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct User {
    pub login: String,
    pub id: u32,
    #[serde(rename(serialize = "user", deserialize = "user"))]
    pub user_type: String,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub watchers_count: u32,
    pub visibility: String,
    pub owner: User,

    /// API accessible url
    pub url: String,
    /// URL on Github website.
    pub html_url: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Issue {
    pub title: String,
    pub body: Option<String>,
    pub user: User,
    pub state: String,
    /// API Accessible url
    pub url: String,
    /// URL on GitHub website
    pub html_url: String,
    pub repository: Repo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Issue {
    pub fn to_text(&self) -> String {
        let html = self.to_html();
        if html.is_empty() {
            return html;
        }

        let html = Html::parse_fragment(&html);
        let mut buffer = String::new();
        let root = html.tree.nodes();
        for node in root {
            if let scraper::Node::Text(text) = node.value() {
                buffer.push_str(text);
            }
        }

        buffer
    }

    pub fn to_html(&self) -> String {
        if let Some(body) = &self.body {
            let html = markdown::to_html_with_options(
                body.as_str(),
                &Options {
                    compile: CompileOptions {
                        allow_dangerous_html: true,
                        allow_dangerous_protocol: true,
                        gfm_tagfilter: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );

            if let Ok(res) = html {
                res
            } else {
                markdown::to_html(body)
            }
        } else {
            String::new()
        }
    }
}

pub struct ApiResponse<T> {
    pub next_page: Option<u32>,
    pub result: T,
}

#[cfg(test)]
mod test {
    use super::Issue;

    #[test]
    pub fn test_to_text() {
        let body = include_str!("../fixtures/issue_body.md");
        let issue = Issue {
            title: "title".to_string(),
            body: Some(body.to_string()),
            ..Default::default()
        };

        // println!("issue body: {}", body);
        let expected = include_str!("../fixtures/issue_body.txt");
        assert_eq!(issue.to_text(), expected);
    }
}
