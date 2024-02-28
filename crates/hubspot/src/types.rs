use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{AsRefStr, Display};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AccountDetails {
    pub portal_id: i32,
    pub time_zone: String,
    pub company_currency: String,
    pub utc_offset: String,
    pub utc_offset_milliseconds: i32,
    pub ui_domain: String,
    pub data_hosting_location: String,
}

#[derive(AsRefStr, Debug, Display)]
pub enum AuthScope {
    #[strum(serialize = "crm.objects.companies.read")]
    CompaniesRead,
    #[strum(serialize = "crm.objects.companies.write")]
    CompaniesWrite,
    #[strum(serialize = "crm.objects.contacts.read")]
    ContactsRead,
    #[strum(serialize = "crm.objects.contacts.write")]
    ContactsWrite,
    #[strum(serialize = "crm.schemas.contacts.read")]
    ContactsSchemaRead,
    #[strum(serialize = "crm.objects.deals.read")]
    DealsRead,
    #[strum(serialize = "crm.objects.deals.write")]
    DealsWrite,
    #[strum(serialize = "crm.schemas.deals.read")]
    DealsSchemaRead,
    #[strum(serialize = "crm.schemas.deals.write")]
    DealsSchemaWrite,
    #[strum(serialize = "sales-email-read")]
    ReadEmail,
}

impl AuthScope {
    pub fn default_scopes() -> Vec<AuthScope> {
        vec![
            AuthScope::CompaniesRead,
            AuthScope::CompaniesWrite,
            AuthScope::ContactsRead,
            AuthScope::ContactsWrite,
            AuthScope::ContactsSchemaRead,
            AuthScope::DealsRead,
            AuthScope::DealsWrite,
            AuthScope::DealsSchemaRead,
            AuthScope::DealsSchemaWrite,
            AuthScope::ReadEmail,
        ]
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NextPage {
    pub after: String,
    pub link: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Paging {
    pub next: NextPage,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PagedResults<T> {
    pub paging: Option<Paging>,
    pub results: Vec<T>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AssociationResult {
    pub results: Vec<Association>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Association {
    pub id: String,
    #[serde(rename = "type")]
    pub association_type: String,
}

/// Note: That the CRM objects "Call", "Email", "Meeting", etc. all have
/// pretty much the same structure. This is separated out for type safety and
/// in case there's any specific impl details for a particular object (e.g. note
/// convience fn to return the note body).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Call {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

impl Call {
    pub fn title(&self) -> String {
        self.properties
            .get("hs_call_title")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn raw_body(&self) -> String {
        self.properties
            .get("hs_call_body")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn recording_url(&self) -> Option<String> {
        self.properties
            .get("hs_call_recording_url")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Email {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EmailStatus {
    #[serde(rename = "BOUNCED")]
    Bounced,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "SCHEDULED")]
    Scheduled,
    #[serde(rename = "SENDING")]
    Sending,
    #[serde(rename = "SENT")]
    Sent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EmailDirection {
    #[serde(rename = "EMAIL")]
    Email,
    #[serde(rename = "INCOMING_EMAIL")]
    IncomingEmail,
    #[serde(rename = "FORWARDED_EMAIL")]
    ForwardedEmail,
}

impl Email {
    pub fn received(&self) -> String {
        self.properties
            .get("hs_timestamp")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn raw_body(&self) -> String {
        self.properties
            .get("hs_email_text")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn html_body(&self) -> String {
        self.properties
            .get("hs_email_html")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn owner_id(&self) -> String {
        self.properties
            .get("hubspot_owner_id")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn status(&self) -> Option<EmailStatus> {
        self.properties
            .get("hs_email_status")
            .and_then(|s| serde_json::from_value::<EmailStatus>(s.clone()).ok())
    }

    pub fn subject(&self) -> String {
        self.properties
            .get("hs_email_subject")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn email_direction(&self) -> Option<EmailDirection> {
        self.properties
            .get("hs_email_direction")
            .and_then(|s| serde_json::from_value::<EmailDirection>(s.clone()).ok())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Meeting {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

impl Note {
    pub fn raw_body(&self) -> String {
        self.properties
            .get("hs_note_body")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
    pub associations: Option<HashMap<String, AssociationResult>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskStatus {
    #[serde(rename = "NOT_STARTED")]
    NotStarted,
    #[serde(rename = "COMPLETED")]
    Completed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskPriority {
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "NONE")]
    None,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskType {
    #[serde(rename = "EMAIL")]
    Email,
    #[serde(rename = "CALL")]
    Call,
    #[serde(rename = "TODO")]
    Todo,
}

impl Task {
    pub fn due_date(&self) -> String {
        self.properties
            .get("hs_timestamp")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn raw_body(&self) -> String {
        self.properties
            .get("hs_task_body")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn owner_id(&self) -> String {
        self.properties
            .get("hubspot_owner_id")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
            .unwrap_or_default()
    }

    pub fn subject(&self) -> Option<String> {
        self.properties
            .get("hs_task_subject")
            .map(|s| s.as_str().unwrap_or(""))
            .map(|s| s.to_owned())
    }

    pub fn status(&self) -> Option<TaskStatus> {
        self.properties
            .get("hs_task_status")
            .and_then(|s| serde_json::from_value::<TaskStatus>(s.clone()).ok())
    }

    pub fn priority(&self) -> Option<TaskPriority> {
        self.properties
            .get("hs_task_priority")
            .and_then(|s| serde_json::from_value::<TaskPriority>(s.clone()).ok())
    }

    pub fn task_type(&self) -> Option<TaskType> {
        self.properties
            .get("hs_task_type")
            .and_then(|s| serde_json::from_value::<TaskType>(s.clone()).ok())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct HubSpotMetaData {
    pub portal_id: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WebhookEvent {
    pub event_id: usize,
    pub subscription_id: usize,
    pub portal_id: usize,
    pub app_id: usize,
    pub occurred_at: u64,
    pub subscription_type: String,
    pub attempt_number: usize,
    pub object_id: usize,
    pub property_name: String,
    pub property_value: String,
    pub change_source: String,
    pub source_id: String,
}
