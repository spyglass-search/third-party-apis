use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::TokenResponse;
pub use oauth2::{AccessToken, RefreshToken};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl};
use oauth2::{CsrfToken, PkceCodeChallenge};
use reqwest::{header, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub mod helpers;
const DEFAULT_USER_AGENT: &str = "spyglass-search";

pub type OnRefreshFn = Box<dyn FnMut(&Credentials) + Send + Sync + 'static>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait ApiClient {
    /// Unique identifier for this API client.
    fn id(&self) -> String;
    /// Authenticated account/user identifier.
    async fn account_id(&mut self) -> Result<String>;
    /// Begin OAuth process w/ list of scopes
    fn authorize(&self, scopes: &[String]) -> AuthorizationRequest;
    /// Get the current credentials
    fn credentials(&self) -> Credentials;
    // Get an HTTP client primed with the current credentials
    fn http_client(&self) -> Client;

    /// Update credentials used by this ApiClient
    fn set_credentials(&mut self, credentials: &Credentials) -> Result<()>;
    fn set_on_refresh(&mut self, callback: impl FnMut(&Credentials) + Send + Sync + 'static);

    /// Handle a token exchange
    async fn token_exchange(&self, code: &str, pkce_verifier: &str) -> Result<BasicTokenResponse>;
    async fn refresh_credentials(&mut self) -> Result<()>;

    // Utility functions to call RESTful api endpoints
    async fn call(
        &mut self,
        endpoint: &str,
        query: &Vec<(String, String)>,
    ) -> Result<reqwest::Response, ApiError> {
        // See if the token is expired
        if self.credentials().is_expired() {
            log::debug!("Refreshing expired token");
            if let Err(err) = self.refresh_credentials().await {
                return Err(ApiError::AuthError(format!(
                    "Unable to refresh credentials: {err}"
                )));
            }
        }

        let client = self.http_client();
        let mut req = client.get(endpoint);
        if !query.is_empty() {
            req = req.query(query);
        }

        match req.send().await {
            Ok(resp) => Ok(resp),
            Err(err) => Err(err.into()),
        }
    }

    async fn call_json<T: DeserializeOwned>(
        &mut self,
        endpoint: &str,
        query: &Vec<(String, String)>,
    ) -> anyhow::Result<T, ApiError> {
        let resp = self.call(endpoint, query).await?;
        match resp.error_for_status() {
            Ok(resp) => match resp.json::<T>().await {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            },
            // Any status code from 400..599
            Err(err) => {
                if let Some(StatusCode::UNAUTHORIZED) = err.status() {
                    Err(ApiError::AuthError("Unauthorized".to_owned()))
                } else {
                    Err(err.into())
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub requested_at: DateTime<Utc>,
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
    pub expires_in: Option<std::time::Duration>,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            requested_at: Utc::now(),
            access_token: AccessToken::new("".into()),
            refresh_token: None,
            expires_in: None,
        }
    }
}

#[derive(Clone)]
pub struct AuthorizationRequest {
    pub url: Url,
    pub csrf_token: CsrfToken,
    pub pkce_challenge: PkceCodeChallenge,
    pub pkce_verifier: String,
}

impl Credentials {
    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.expires_in {
            let now = Utc::now();
            let dur = chrono::Duration::from_std(duration).expect("Unable to convert duration");
            return (now - self.requested_at) > dur;
        }

        false
    }

    pub fn refresh_token(&mut self, resp: &BasicTokenResponse) {
        self.requested_at = Utc::now();
        self.access_token = resp.access_token().clone();
        self.refresh_token = resp.refresh_token().cloned();
        self.expires_in = resp.expires_in();
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        std::fs::write(path, serde_json::to_string(self)?)?;
        Ok(())
    }
}

pub fn auth_http_client(token: &str) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    let value = header::HeaderValue::from_str(&format!("Bearer {token}"))?;
    headers.insert("Authorization", value);

    Ok(reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .default_headers(headers)
        .build()?)
}

#[derive(Default)]
pub struct OAuthParams {
    pub auth_url: String,
    pub token_url: Option<String>,
    pub revoke_url: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_url: Option<String>,
}

pub fn oauth_client(params: &OAuthParams) -> BasicClient {
    let auth_url =
        AuthUrl::new(params.auth_url.clone()).expect("Invalid authorization endpoint URL");

    let client_secret = params.client_secret.clone();
    let token_url = params.token_url.clone();

    let mut client = BasicClient::new(
        ClientId::new(params.client_id.clone()),
        client_secret.map(ClientSecret::new),
        auth_url,
        token_url.map(|url| TokenUrl::new(url).expect("Invalid token endpoint URL")),
    );

    if let Some(redirect_url) = &params.redirect_url {
        client = client.set_redirect_uri(
            RedirectUrl::new(redirect_url.to_string()).expect("Invalid redirect URL"),
        );
    }

    if let Some(revoke_url) = &params.revoke_url {
        client = client.set_revocation_uri(
            RevocationUrl::new(revoke_url.to_string()).expect("Invalid revocation endpoint URL"),
        );
    }

    client
}
