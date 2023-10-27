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

/// Note: That the CRM objects "Call", "Email", "Meeting", etc. all have
/// pretty much the same structure. This is separated out for type safety and
/// in case there's any specific impl details for a particular object (e.g. note
/// convience fn to return the note body).
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Call {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
}

impl Call {
    pub fn title(&self) -> String {
        self.properties.get("hs_call_title")
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Email {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Meeting {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, Value>,
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub archived_at: Option<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WebhookEvent {
    event_id: usize,
    subscription_id: usize,
    portal_id: usize,
    app_id: usize,
    occurred_at: u64,
    subscription_type: String,
    attempt_number: usize,
    object_id: usize,
    property_name: String,
    property_value: String,
    change_source: String,
    source_id: String,
}