use serde::{Deserialize, Serialize};
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
    pub data_hosting_location: String
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
    DealsSchemaWrite
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
            AuthScope::DealsSchemaWrite
        ]
    }
}
