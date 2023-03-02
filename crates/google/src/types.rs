use std::str::FromStr;

use anyhow::anyhow;
use chrono::{DateTime, NaiveDate, Utc};
pub use rrule::Tz;
use rrule::{RRule, RRuleSet};
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

impl CalendarEvent {
    pub fn is_recurring(&self) -> bool {
        !self.recurrence.is_empty()
    }

    pub fn next_recurrence(&self) -> Option<DateTime<Tz>> {
        self.list_recurrences(1, None, None)
            .map(|x| x.get(0).map(|x| x.to_owned()))
            .unwrap_or_default()
    }

    /// If this is a recurring event, this will return the next <num_recurrences>
    /// after the date specified. If <after> is set to None, the current date and time
    /// will be used.
    pub fn list_recurrences(
        &self,
        num_recurrences: u16,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Vec<DateTime<Tz>>> {
        let start = if let Some(date) = self.start.date_time {
            date
        } else if let Ok(date) = NaiveDate::parse_from_str(&self.start.date, "%Y-%m-%d") {
            let date = date.and_hms_opt(0, 0, 0).expect("Invalid hms");
            DateTime::from_utc(date, Utc)
        } else {
            return Err(anyhow!("Invalid date"));
        };

        // Adjust the timezone to UTC
        let start = start.with_timezone(&Tz::UTC);
        let mut rrules = RRuleSet::new(start);
        for recur in self.recurrence.iter() {
            if let Ok(recur) = RRule::from_str(recur) {
                let validated = recur.validate(start)?;
                rrules = rrules.rrule(validated);
            }
        }

        // Only show dates after <after> or today by default
        let after = after.unwrap_or(Utc::now());
        // By default, if <before> is not set, only show dates 1 year in advance
        let before = before.unwrap_or(after + chrono::Duration::days(365));
        rrules = rrules
            .after(after.with_timezone(&Tz::UTC))
            .before(before.with_timezone(&Tz::UTC));

        let (result, _) = rrules.all(num_recurrences);
        let result = result
            .iter()
            .map(|x| x.with_timezone(&Tz::UTC))
            .collect::<Vec<_>>();

        Ok(result)
    }
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

#[cfg(test)]
mod test {
    use crate::types::{CalendarEvent, CalendarTime};
    use chrono::TimeZone;

    #[test]
    fn test_next_recurrence_yearly() {
        let event = CalendarEvent {
            start: CalendarTime {
                date: "2019-11-12".into(),
                date_time: None,
                time_zone: "America/Los_Angeles".into(),
            },
            recurrence: vec!["RRULE:FREQ=YEARLY;INTERVAL=1".into()],
            ..Default::default()
        };

        let today = chrono::Utc.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap();
        let recurrences = event
            .list_recurrences(1, Some(today), None)
            .expect("Unable to get next recurrences");
        assert_eq!(recurrences.len(), 1);

        let next = event.next_recurrence();
        assert!(next.is_some());
        assert_eq!(next.unwrap().to_rfc3339(), "2023-11-12T00:00:00+00:00");
    }

    #[test]
    fn test_next_recurrence_none() {
        let event = CalendarEvent {
            start: CalendarTime {
                date: "".into(),
                date_time: Some("2007-10-01T13:00:00-07:00".parse().expect("Invalid date")),
                time_zone: "America/Los_Angeles".into(),
            },
            recurrence: vec!["RRULE:FREQ=WEEKLY;WKST=SU;COUNT=30;INTERVAL=1;BYDAY=MO,WE,FR".into()],
            ..Default::default()
        };

        let recurrences = event
            .list_recurrences(5, None, None)
            .expect("Unable to get next recurrences");
        assert_eq!(recurrences.len(), 0);
    }

    #[test]
    fn test_next_recurrence_until() {
        let datetime = chrono::DateTime::parse_from_rfc3339("2023-01-03T09:30:00-08:00").unwrap();

        let event = CalendarEvent {
            start: CalendarTime {
                date: "".into(),
                date_time: Some(datetime.with_timezone(&chrono::Utc)),
                time_zone: "America/Los_Angeles".into(),
            },
            recurrence: vec![
                "RRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20230308T000000Z;INTERVAL=3;BYDAY=TU".into(),
            ],
            ..Default::default()
        };

        let recurrences = event
            .list_recurrences(5, None, None)
            .expect("Unable to get next recurrences");

        dbg!(&recurrences);
        assert_eq!(recurrences.len(), 1);
    }
}
