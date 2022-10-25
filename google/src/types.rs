use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Serialize)]
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
    pub next_page_token: String,
    pub files: Vec<FileInfo>,
}
