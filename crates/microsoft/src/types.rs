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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskLists {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub value: Vec<TaskListsDef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLink {
    pub web_url: Option<String>,
    pub application_name: String,
    pub display_name: String,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl From<&GenericMessage> for ResourceLink {
    fn from(value: &GenericMessage) -> Self {
        ResourceLink {
            web_url: Some(value.web_link.to_string()),
            application_name: "Outlook".to_string(),
            display_name: value.subject.to_string(),
            external_id: value.conversation_id.to_string(),
            id: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskList {
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskListTasks {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub value: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "@odata.etag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub odata_etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub body: Option<TaskBody>,
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    #[default]
    NotStarted,
    InProgress,
    Completed,
    WaitingOnOthers,
    Deferred,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskDateTime {
    // example:  "dateTime": "2024-02-05T08:00:00.0000000",
    pub date_time: String,
    // example: "timeZone": "UTC"
    pub time_zone: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskBody {
    pub content: String,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TaskImportance {
    Low,
    #[default]
    Normal,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatternedRecurrence {
    pattern: RecurrencePattern,
    range: RecurrenceRange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RecurrenceRange {
    end_date: Option<String>,
    number_of_occurrences: Option<i32>,
    recurrence_time_zone: Option<String>,
    start_date: String,
    recurrence_range_type: RecurrenceRangeType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum RecurrenceRangeType {
    EndDate,
    NoEnd,
    Numbered,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum WeekIndex {
    First,
    Second,
    Third,
    Fourth,
    Last,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum RecurrencePatternType {
    Daily,
    Weekly,
    AbsoluteMonthly,
    RelativeMonthly,
    AbsoluteYearly,
    RelativeYearly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageAddress {
    pub email_address: EmailAddress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    pub flag_status: FlagStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FlagStatus {
    NotFlagged,
    Flagged,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub content_type: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageRemovedReason {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericMessage {
    #[serde(rename = "@odata.etag")]
    pub odata_etag: String,
    pub created_date_time: String,
    pub last_modified_date_time: String,
    pub change_key: String,
    pub categories: Vec<String>,
    pub received_date_time: String,
    pub sent_date_time: String,
    pub has_attachments: bool,
    pub internet_message_id: String,
    pub subject: String,
    pub body_preview: String,
    pub importance: String,
    pub parent_folder_id: String,
    pub conversation_id: String,
    pub conversation_index: String,
    pub is_delivery_receipt_requested: bool,
    pub is_read_receipt_requested: bool,
    pub is_read: bool,
    pub is_draft: bool,
    pub web_link: String,
    pub inference_classification: String,
    pub body: Body,
    pub sender: MessageAddress,
    pub from: MessageAddress,
    pub to_recipients: Vec<MessageAddress>,
    pub cc_recipients: Vec<MessageAddress>,
    pub bcc_recipients: Vec<MessageAddress>,
    pub reply_to: Vec<MessageAddress>,
    pub flag: Flag,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "@odata.type")]
    pub odata_type: String,
    #[serde(rename = "@removed")]
    pub removed: Option<MessageRemovedReason>,
    pub id: String,
    #[serde(flatten)]
    pub message: Option<GenericMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageCollection {
    #[serde(rename = "@odata.context")]
    pub odata_context: Option<String>,
    pub value: Vec<Message>,
    #[serde(rename = "@odata.deltaLink")]
    pub odata_delta_link: Option<String>,
    #[serde(rename = "@odata.nextLink")]
    pub odata_next_link: Option<String>,
}
