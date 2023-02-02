use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarAttendee {
    pub id: String,
    pub email: String,
    pub display_name: String,
    #[serde(rename = "organizer")]
    pub is_organizer: bool,
    #[serde(rename = "self")]
    pub is_self: bool,
    #[serde(rename = "optional")]
    pub is_optional: bool,
    pub response_status: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarTime {
    pub date: String,
    pub date_time: Option<DateTime<Utc>>,
    pub time_zone: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub etag: String,
    pub attendees: Vec<CalendarAttendee>,
    pub status: String,
    pub html_link: String,
    /// Creation time of the event (as a RFC3339 timestamp). Read-only.
    pub created: DateTime<Utc>,
    /// Description of the event. Can contain HTML. Optional.
    pub description: Option<String>,
    /// Geographic location of the event as free-form text. Optional.
    pub location: Option<String>,
    /// Title of the event.
    pub summary: String,
    /// The (inclusive) start time of the event. For a recurring event, this is the
    /// start time of the first instance.
    pub start: CalendarTime,
    /// The (exclusive) end time of the event. For a recurring event, this is
    /// the end time of the first instance.
    pub end: CalendarTime,
    pub recurrence: Vec<String>,
    pub recurring_event_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ListCalendarEventsResponse {
    pub etag: String,
    pub items: Vec<CalendarEvent>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarList {
    pub access_role: String,
    pub description: String,
    pub etag: String,
    pub id: String,
    pub kind: String,
    pub primary: bool,
    pub secondary: bool,
    pub summary: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CalendarListResponse {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
    pub items: Vec<CalendarList>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FileUser {
    pub display_name: String,
    // The email address of the user. This may not be present in certain contexts if
    // the user has not made their email address visible to the user.
    pub email_address: Option<String>,
    #[serde(rename = "me")]
    pub is_me: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct File {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub description: String,
    pub starred: bool,
    pub parents: Vec<String>,
    pub version: String,
    pub owners: Vec<FileUser>,
    pub sharing_user: FileUser,
    pub last_modifying_user: FileUser,
    pub web_view_link: String,
    pub created_time: DateTime<Utc>,
    pub modified_time: Option<DateTime<Utc>>,
    pub shared_with_me_time: Option<DateTime<Utc>>,
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
#[derive(AsRefStr, Debug, Display)]
/// Taken from https://developers.google.com/identity/protocols/oauth2/scopes
pub enum AuthScope {
    #[strum(serialize = "https://www.googleapis.com/auth/calendar.readonly")]
    Calendar,
    #[strum(serialize = "https://www.googleapis.com/auth/calendar.events.readonly")]
    CalendarEvents,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.readonly")]
    Drive,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.activity.readonly")]
    DriveActivity,
    #[strum(serialize = "https://www.googleapis.com/auth/drive.metadata.readonly")]
    DriveMetadata,
    /// Email associated w/ the account.
    #[strum(serialize = "email")]
    Email,
    #[strum(serialize = "https://www.googleapis.com/auth/gmail.readonly")]
    Gmail,
    #[strum(serialize = "https://www.googleapis.com/auth/gmail.metadata")]
    GmailMetadata,
    #[strum(serialize = "https://www.googleapis.com/auth/photoslibrary.readonly")]
    Photos,
    #[strum(serialize = "https://www.googleapis.com/auth/youtube.readonly")]
    YouTube,
}

#[derive(AsRefStr, Debug, EnumString, PartialEq, Eq)]
pub enum FileType {
    #[strum(serialize = "application/vnd.google-apps.document")]
    Document,
    #[strum(serialize = "application/vnd.google-apps.spreadsheet")]
    Spreadsheet,
    #[strum(serialize = "application/vnd.google-apps.presentation")]
    Presentation,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GoogUser {
    pub email: String,
}
