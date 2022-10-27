use bytes::Bytes;

use anyhow::{anyhow, Result};
use oauth2::basic::BasicClient;
use oauth2::TokenResponse;
use reqwest::Client;
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope};
use url::Url;

mod auth;
pub use auth::{auth_http_client, oauth_client, AuthorizationRequest, Credentials};
mod types;
use types::{File, FileInfo, Files};

const API_ENDPOINT: &str = "https://www.googleapis.com/drive/v3";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";

pub struct GoogClient {
    endpoint: Url,
    http: Client,
    pub oauth: BasicClient,
    pub credentials: Credentials,
}

impl GoogClient {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        creds: Credentials,
    ) -> anyhow::Result<Self> {
        Ok(GoogClient {
            endpoint: Url::parse(API_ENDPOINT).expect("Invalid API endpoint"),
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(client_id, client_secret, redirect_url),
            credentials: creds,
        })
    }

    pub async fn call(
        &mut self,
        endpoint: &str,
        query: &Vec<(String, String)>,
    ) -> Result<reqwest::Response> {
        // See if the token is expired
        if self.credentials.is_expired() {
            log::debug!("Refreshing expired token");
            if let Err(err) = self.refresh_credentials().await {
                return Err(anyhow!("Unable to refresh credentials: {}", err));
            }
        }

        match self.http.get(endpoint).query(query).send().await {
            Ok(resp) => Ok(resp),
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    pub async fn refresh_credentials(&mut self) -> Result<()> {
        if let Some(refresh_token) = &self.credentials.refresh_token {
            let new_token = self
                .oauth
                .exchange_refresh_token(refresh_token)
                .request_async(async_http_client)
                .await?;

            self.credentials.refresh_token(&new_token);
            self.http = auth_http_client(new_token.access_token().secret())?;
        }

        Ok(())
    }

    pub fn set_credentials(&mut self, credentials: &Credentials) -> Result<()> {
        self.credentials = credentials.clone();
        self.http = auth_http_client(credentials.access_token.secret())?;
        Ok(())
    }

    pub async fn download_file(&mut self, file: &FileInfo) -> Result<Bytes> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files/");
        endpoint.push_str(&file.id);

        let mut params = Vec::new();
        // If Google specific file, we need to export
        if file.mime_type.starts_with("application/vnd.google-apps") {
            endpoint.push_str("/export");
            let export_type = match file.mime_type.as_str() {
                "application/vnd.google-apps.document" => "text/plain",
                // Excel
                "application/vnd.google-apps.spreadsheet" => {
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                }
                "application/vnd.google-apps.presentation" => "text/plain",
                _ => {
                    return Err(anyhow!("Unsupported file type"));
                }
            };

            params.push(("mimeType".to_string(), export_type.into()));
        } else {
            params.push(("alt".to_string(), "media".to_string()));
        }

        let resp = self.call(&endpoint, &params).await?;
        Ok(resp.bytes().await?)
    }

    pub async fn list_files(&mut self) -> Result<Files> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files");

        let resp = self.call(&endpoint, &Vec::new()).await?;

        match resp.error_for_status() {
            Ok(resp) => match resp.json::<Files>().await {
                Ok(res) => Ok(res),
                Err(err) => Err(anyhow!(err.to_string())),
            },
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    pub async fn get_file_metadata(&mut self, id: &str) -> Result<File> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files/");
        endpoint.push_str(id);

        let params = vec![("fields".to_string(), "*".to_string())];
        let resp = self.call(&endpoint, &params).await?;
        match resp.error_for_status() {
            Ok(resp) => match resp.json::<File>().await {
                Ok(res) => Ok(res),
                Err(err) => Err(anyhow!(err.to_string())),
            },
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    pub fn authorize(&self) -> AuthorizationRequest {
        // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = self
            .oauth
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive.readonly".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive.metadata.readonly".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive.activity.readonly".to_string(),
            ))
            .set_pkce_challenge(pkce_code_challenge.clone())
            .url();

        AuthorizationRequest {
            url: authorize_url,
            csrf_token: csrf_state,
            pkce_challenge: pkce_code_challenge,
            pkce_verifier: pkce_code_verifier.secret().to_string(),
        }
    }

    pub async fn token_exchange(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Option<BasicTokenResponse> {
        let code = AuthorizationCode::new(code.to_owned());

        self.oauth
            .exchange_code(code)
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_owned()))
            .request_async(async_http_client)
            .await
            .ok()
    }
}
