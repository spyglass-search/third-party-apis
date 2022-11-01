use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct File {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub description: String,
    pub starred: bool,
    pub trashed: bool,
    pub parents: Vec<String>,
    pub properties: HashMap<String, String>,
    pub spaces: Vec<String>,
    pub version: String,
    pub web_content_link: String,
    pub web_view_link: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInfo {
    pub kind: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct Files {
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    pub files: Vec<FileInfo>,
}

#[allow(dead_code)]
#[derive(AsRefStr, Debug)]
/// Taken from https://developers.google.com/identity/protocols/oauth2/scopes
pub enum AuthScope {
    #[strum(serialize = "https://www.googleapis.com/auth/calendar.events.readonly")]
    Calendar,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.readonly")]
    Drive,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.activity.readonly")]
    DriveActivity,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.metadata.readonly")]
    DriveMetadata,
    #[strum(serialize = "https://www.googleapis.com/auth/gmail.readonly")]
    Gmail,
    #[strum(serialize = "https://www.googleapis.com/auth/gmail.metadata")]
    GmailMetadata,
    #[strum(serialize = "https://www.googleapis.com/auth/photoslibrary.readonly")]
    Photos,
    #[strum(serialize = "https://www.googleapis.com/auth/youtube.readonly")]
    YouTube
}

#[derive(AsRefStr, Debug, EnumString, PartialEq, Eq)]
pub enum FileType {
    #[strum(serialize = "application/vnd.google-apps.document")]
    Document,
    #[strum(serialize = "application/vnd.google-apps.spreadsheet")]
    Spreadsheet,
    #[strum(serialize = "application/vnd.google-apps.presentation")]
    Presentation
}