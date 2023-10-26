use anyhow::{anyhow, Result};
use async_trait::async_trait;
use libauth::{ApiClient, OAuthParams, Credentials, auth_http_client, oauth_client, AuthorizeOptions, AuthorizationRequest};
use oauth2::{basic::{BasicClient, BasicTokenResponse}, Scope, CsrfToken, AuthorizationCode, reqwest::async_http_client, TokenResponse, RequestTokenError};
use reqwest::Client;
use strum_macros::{AsRefStr, Display};
use tokio::sync::watch;

const AUTH_URL: &str = "https://app.hubspot.com/oauth/authorize";
const TOKEN_URL: &str = "https://api.hubapi.com/oauth/v1/token";

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
        Ok("test".into())
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
        let mut exchange = self.oauth.exchange_code(code);
        exchange = exchange.add_extra_param("client_id", self.oauth.client_id().to_string())
            .add_extra_param("client_secret", self.secret.clone());

        let req = exchange.request_async(async_http_client).await;
        dbg!(&req);

        match req {
            Ok(val) => Ok(val),
            Err(err) => {
                match err {
                    RequestTokenError::Parse(err, og) => {
                        dbg!(&std::str::from_utf8(&og));
                        Err(anyhow!(err.to_string()))
                    }
                    x => Err(anyhow!(x.to_string()))
                }
            },
        }
    }

    async fn refresh_credentials(&mut self) -> Result<()> {
        if let Some(refresh_token) = &self.credentials.refresh_token {
            let new_token = self
                .oauth
                .exchange_refresh_token(refresh_token)
                .request_async(async_http_client)
                .await?;

            self.credentials.refresh_token(&new_token);
            self.http = auth_http_client(new_token.access_token().secret())?;
            // Let any listeners know the credentials have been updated.
            self.on_refresh_tx.send(self.credentials.clone())?;
        }

        Ok(())
    }
}

impl HubspotClient {
    pub fn new(client_id: &str, client_secret: &str, redirect_url: &str, creds: Credentials) -> anyhow::Result<Self> {
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
}