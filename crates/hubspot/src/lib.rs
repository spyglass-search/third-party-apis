use anyhow::{anyhow, Result};
use async_trait::async_trait;
use libauth::{
    auth_http_client, oauth_client, ApiClient, ApiError, AuthorizationRequest, AuthorizeOptions,
    Credentials, OAuthParams,
};
use oauth2::{
    basic::{BasicClient, BasicTokenResponse},
    reqwest::async_http_client,
    AuthorizationCode, CsrfToken, RequestTokenError, Scope, TokenResponse,
};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use strum_macros::{Display, EnumString};
use tokio::sync::watch;
use types::HubSpotMetaData;

pub mod types;

const AUTH_URL: &str = "https://app.hubspot.com/oauth/authorize";
const TOKEN_URL: &str = "https://api.hubapi.com/oauth/v1/token";
const API_ENDPOINT: &str = "https://api.hubapi.com";

const DEFAULT_PROPERTIES: &[(CrmObject, &[&str])] = &[
    (
        CrmObject::Calls,
        &[
            "hs_activity_type",
            "hs_attachment_ids",
            "hs_call_body",
            "hs_call_callee_object_id",
            "hs_call_direction",
            "hs_call_disposition",
            "hs_call_duration",
            "hs_call_from_number",
            "hs_call_recording_url",
            "hs_call_status",
            "hs_call_title",
            "hs_call_to_number",
            "hs_createdate",
            "hs_lastmodifieddate",
            "hs_timestamp",
            "hubspot_owner_id",
        ],
    ),
    (
        CrmObject::Notes,
        &[
            "hs_attachment_ids",
            "hs_note_body",
            "hs_timestamp",
            "hubspot_owner_id",
        ],
    ),
    (
        CrmObject::Tasks,
        &[
            "hs_timestamp",
            "hs_task_body",
            "hubspot_owner_id",
            "hs_task_subject",
            "hs_task_status",
            "hs_task_priority",
            "hs_task_type",
        ],
    ),
    (
        CrmObject::Emails,
        &[
            "hs_timestamp",
            "hs_email_direction",
            "hubspot_owner_id",
            "hs_email_html",
            "hs_email_status",
            "hs_email_subject",
            "hs_email_text",
            "hs_attachment_ids",
            "hs_email_from_email",
            "hs_email_from_firstname",
            "hs_email_from_lastname",
            "hs_email_to_email",
            "hs_email_to_firstname",
            "hs_email_to_lastname",
        ],
    ),
];

#[derive(Display, EnumString, PartialEq, Eq, Clone)]
pub enum CrmObject {
    #[strum(serialize = "calls")]
    Calls,
    #[strum(serialize = "contacts")]
    Contacts,
    #[strum(serialize = "emails")]
    Emails,
    #[strum(serialize = "meetings")]
    Meetings,
    #[strum(serialize = "notes")]
    Notes,
    #[strum(serialize = "tasks")]
    Tasks,
}

pub struct HubspotClient {
    http: Client,
    pub oauth: BasicClient,
    pub secret: String,
    pub credentials: Credentials,
    pub on_refresh_tx: watch::Sender<Credentials>,
    pub on_refresh_rx: watch::Receiver<Credentials>,
}

#[async_trait]
impl ApiClient for HubspotClient {
    fn id(&self) -> String {
        String::from("hubspot.com")
    }

    async fn account_id(&mut self) -> anyhow::Result<String> {
        let details = self.account_details().await?;
        Ok(details.portal_id.to_string())
    }

    async fn account_metadata(&mut self) -> Option<Value> {
        self.account_details().await.ok().map(|details| {
            serde_json::to_value(HubSpotMetaData {
                portal_id: details.portal_id,
            })
            .unwrap()
        })
    }

    fn credentials(&self) -> Credentials {
        self.credentials.clone()
    }

    fn http_client(&self) -> Client {
        self.http.clone()
    }

    fn set_credentials(&mut self, credentials: &Credentials) -> anyhow::Result<()> {
        self.credentials = credentials.clone();
        self.http = auth_http_client(credentials.access_token.secret())?;
        Ok(())
    }

    fn watch_on_refresh(&mut self) -> watch::Receiver<Credentials> {
        self.on_refresh_rx.clone()
    }

    fn authorize(&self, scopes: &[String], options: &AuthorizeOptions) -> AuthorizationRequest {
        let scopes = scopes
            .iter()
            .map(|s| Scope::new(s.to_string()))
            .collect::<Vec<Scope>>();

        let mut req = self
            .oauth
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes);

        for (key, value) in &options.extra_params {
            req = req.add_extra_param(key, value)
        }

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = req.url();

        AuthorizationRequest {
            url: authorize_url,
            csrf_token: csrf_state,
            pkce_challenge: None,
            pkce_verifier: None,
        }
    }

    async fn token_exchange(
        &self,
        code: &str,
        _pkce_verifier: Option<String>,
    ) -> anyhow::Result<BasicTokenResponse> {
        let code = AuthorizationCode::new(code.to_owned());
        let exchange = self
            .oauth
            .exchange_code(code)
            .add_extra_param("client_id", self.oauth.client_id().to_string())
            .add_extra_param("client_secret", self.secret.clone());

        match exchange.request_async(async_http_client).await {
            Ok(val) => Ok(val),
            Err(err) => match err {
                RequestTokenError::Parse(err, og) => {
                    let msg = std::str::from_utf8(&og)
                        .map(|x| x.to_string())
                        .unwrap_or(err.to_string());
                    Err(anyhow!(msg))
                }
                x => Err(anyhow!(x.to_string())),
            },
        }
    }

    async fn refresh_credentials(&mut self) -> Result<()> {
        if let Some(refresh_token) = &self.credentials.refresh_token {
            let req = self
                .oauth
                .exchange_refresh_token(refresh_token)
                .add_extra_param("client_id", self.oauth.client_id().to_string())
                .add_extra_param("client_secret", self.secret.clone());

            let new_token = match req.request_async(async_http_client).await {
                Ok(token) => token,
                Err(err) => {
                    return match err {
                        RequestTokenError::Parse(err, og) => {
                            let msg = std::str::from_utf8(&og)
                                .map(|x| x.to_string())
                                .unwrap_or(err.to_string());
                            Err(anyhow!(msg))
                        }
                        x => Err(anyhow!(x.to_string())),
                    };
                }
            };

            self.credentials.refresh_token(&new_token);
            self.http = auth_http_client(new_token.access_token().secret())?;
            // Let any listeners know the credentials have been updated.
            self.on_refresh_tx.send(self.credentials.clone())?;
        }

        Ok(())
    }
}

impl HubspotClient {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        creds: Credentials,
    ) -> anyhow::Result<Self> {
        let params = OAuthParams {
            client_id: client_id.to_string(),
            client_secret: Some(client_secret.to_string()),
            redirect_url: Some(redirect_url.to_string()),
            auth_url: AUTH_URL.to_string(),
            token_url: Some(TOKEN_URL.to_string()),
            revoke_url: None,
        };

        let (tx, rx) = watch::channel(creds.clone());
        Ok(HubspotClient {
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            secret: client_secret.to_string(),
            credentials: creds,
            on_refresh_tx: tx,
            on_refresh_rx: rx,
        })
    }

    pub async fn account_details(&mut self) -> Result<types::AccountDetails, ApiError> {
        let endpoint = format!("{API_ENDPOINT}/account-info/v3/details");
        serde_json::from_value::<types::AccountDetails>(self.call_json(&endpoint, &[]).await?)
            .map_err(ApiError::SerdeError)
    }

    pub async fn get_object<T>(
        &mut self,
        object: CrmObject,
        id: &str,
        properties: &[String],
        associations: &[String],
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let endpoint = format!("{API_ENDPOINT}/crm/v3/objects/{}/{id}", object);

        let props = properties.join(",");
        let default_props = default_prop_as_string(&object);

        let mut query: Vec<(String, String)> = match default_props {
            Some(default_props) => {
                vec![("properties".into(), format!("{default_props},{props}"))]
            }
            None => {
                if !properties.is_empty() {
                    vec![("properties".into(), props.to_string())]
                } else {
                    vec![]
                }
            }
        };

        if !associations.is_empty() {
            query.push(("associations".into(), associations.join(",").to_string()))
        }

        serde_json::from_value(self.call_json(&endpoint, &query).await?)
            .map_err(ApiError::SerdeError)
    }

    pub async fn list_objects<T>(
        &mut self,
        object: CrmObject,
        properties: &[String],
        associations: &[String],
        after: Option<String>,
        limit: Option<usize>,
    ) -> Result<types::PagedResults<T>, ApiError>
    where
        T: DeserializeOwned,
    {
        let endpoint = format!("{API_ENDPOINT}/crm/v3/objects/{}", object);
        let props = properties.to_vec();
        let default_props = default_prop_as_string(&object);

        let mut query: Vec<(String, String)> = match default_props {
            Some(default_props) => {
                let prop_string = props.join(",");
                vec![(
                    "properties".into(),
                    format!("{default_props},{prop_string}"),
                )]
            }
            None => {
                if !properties.is_empty() {
                    let prop_string = props.join(",");
                    vec![("properties".into(), prop_string.to_string())]
                } else {
                    vec![]
                }
            }
        };

        query.push(("limit".into(), limit.unwrap_or(10).to_string()));

        if let Some(after) = after {
            query.push(("after".into(), after.clone()));
        }

        if !associations.is_empty() {
            query.push(("associations".into(), associations.join(",").to_string()))
        }

        serde_json::from_value(self.call_json(&endpoint, &query).await?)
            .map_err(ApiError::SerdeError)
    }
}

pub fn default_prop_as_string(object: &CrmObject) -> Option<String> {
    for (obj, props) in DEFAULT_PROPERTIES {
        if object.eq(obj) {
            return Some(props.join(",").to_string());
        }
    }
    None
}
