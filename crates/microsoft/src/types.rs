use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Scopes pulled from graph explorer: https://developer.microsoft.com/en-us/graph/graph-explorer
#[derive(Debug, Display, EnumString)]
pub enum AuthScopes {
    /// Grant access to read users full profiles
    #[strum(serialize = "User.Read")]
    UserRead,
    /// Grants access to microsoft todo read and write
    #[strum(serialize = "Tasks.ReadWrite")]
    TasksReadWrite,
    /// Grant access to the outlook calendar
    #[strum(serialize = "Calendars.ReadWrite")]
    CalendarReadWrite,
    /// Grant read access to outlook emails.
    #[strum(serialize = "Mail.Read")]
    MailRead,
    /// Grants access when user is offline (refresh token given)
    #[strum(serialize = "offline_access")]
    OfflineAccess,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub business_phones: Vec<String>,
    pub display_name: String,
    pub given_name: Option<String>,
    pub job_title: Option<String>,
    pub mail: Option<String>,
    pub mobile_phone: Option<String>,
    pub office_location: Option<String>,
    pub preferred_language: Option<String>,
    pub surname: Option<String>,
    pub user_principal_name: Option<String>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLists {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub value: Vec<TaskListsDef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListsDef {
    #[serde(rename = "@odata.etag")]
    pub odata_etag: String,
    pub display_name: String,
    pub is_owner: bool,
    pub is_shared: bool,
    pub wellknown_list_name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskList {
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListTasks {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub value: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "@odata.etag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub odata_etag: Option<String>,
    pub body: TaskBody,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub categories: Vec<String>,
    // ISO 8601
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub created_date_time: Option<String>,
    // ISO 8601
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub last_modified_date_time: Option<String>,
    // ISO 8601
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub body_last_modified_date_time: Option<String>,
    pub has_attachments: bool,
    pub id: String,
    pub importance: TaskImportance,
    pub is_reminder_on: bool,
    pub status: TaskStatus,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub recurrence: Option<TaskPatternedRecurrence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reminder_date_time: Option<TaskDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub start_date_time: Option<TaskDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub completed_date_time: Option<TaskDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub due_date_time: Option<TaskDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    WaitingOnOthers,
    Deferred,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDateTime {
    // example:  "dateTime": "2024-02-05T08:00:00.0000000",
    pub date_time: String,
    // example: "timeZone": "UTC"
    pub time_zone: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBody {
    pub content: String,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskImportance {
    Low,
    Normal,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatternedRecurrence {
    pattern: RecurrencePattern,
    range: RecurrenceRange,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurrenceRange {
    end_date: Option<String>,
    number_of_occurrences: Option<i32>,
    recurrence_time_zone: Option<String>,
    start_date: String,
    recurrence_range_type: RecurrenceRangeType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum RecurrenceRangeType {
    EndDate,
    NoEnd,
    Numbered,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurrencePattern {
    day_of_month: Option<i32>,
    days_of_week: Option<Vec<DayOfWeek>>,
    first_day_of_week: Option<DayOfWeek>,
    index: Option<WeekIndex>,
    interval: i32,
    month: Option<i32>,
    #[serde(rename = "type")]
    recurrence_pattern_type: RecurrencePatternType,
}

#[derive(Debug, Serialize, Deserialize)]
enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Serialize, Deserialize)]
enum WeekIndex {
    First,
    Second,
    Third,
    Fourth,
    Last,
}

#[derive(Debug, Serialize, Deserialize)]
enum RecurrencePatternType {
    Daily,
    Weekly,
    AbsoluteMonthly,
    RelativeMonthly,
    AbsoluteYearly,
    RelativeYearly,
}
