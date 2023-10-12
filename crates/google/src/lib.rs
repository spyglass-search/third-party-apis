use bytes::Bytes;
use libauth::{AuthorizeOptions, OAuthParams};
use std::str::FromStr;
use tokio::sync::watch;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use oauth2::basic::BasicClient;
use oauth2::TokenResponse;
use reqwest::Client;
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope};

use libauth::{
    auth_http_client, oauth_client, ApiClient, ApiError, AuthorizationRequest, Credentials,
};

pub mod services;
pub mod types;

use types::{File, FileType, Files, GoogUser};

pub enum ClientType {
    Calendar,
    Drive,
    Sheets,
}

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";

pub struct GoogClient {
    client_type: ClientType,
    endpoint: String,
    http: Client,
    pub oauth: BasicClient,
    pub credentials: Credentials,
    pub on_refresh_tx: watch::Sender<Credentials>,
    pub on_refresh_rx: watch::Receiver<Credentials>,
}

#[async_trait]
impl ApiClient for GoogClient {
    fn id(&self) -> String {
        match self.client_type {
            ClientType::Calendar => "calendar.google.com".to_string(),
            ClientType::Drive => "drive.google.com".to_string(),
            ClientType::Sheets => "sheets.googleapis.com".to_string(),
        }
    }

    async fn account_id(&mut self) -> Result<String> {
        let user = self.get_user().await?;
        Ok(user.email)
    }

    fn credentials(&self) -> Credentials {
        self.credentials.clone()
    }

    fn http_client(&self) -> Client {
        self.http.clone()
    }

    fn set_credentials(&mut self, credentials: &Credentials) -> Result<()> {
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

        let (pkce_challenge, pkce_verifier) = if options.pkce {
            // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
            // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
            let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
            req = req.set_pkce_challenge(pkce_code_challenge.clone());
            (
                Some(pkce_code_challenge),
                Some(pkce_code_verifier.secret().to_string()),
            )
        } else {
            (None, None)
        };

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = req.url();

        AuthorizationRequest {
            url: authorize_url,
            csrf_token: csrf_state,
            pkce_challenge,
            pkce_verifier,
        }
    }

    async fn token_exchange(
        &self,
        code: &str,
        pkce_verifier: Option<String>,
    ) -> Result<BasicTokenResponse> {
        let code = AuthorizationCode::new(code.to_owned());
        let mut exchange = self.oauth.exchange_code(code);
        if let Some(pkce) = pkce_verifier {
            exchange = exchange.set_pkce_verifier(PkceCodeVerifier::new(pkce));
        }

        match exchange.request_async(async_http_client).await {
            Ok(val) => Ok(val),
            Err(err) => Err(anyhow!(err.to_string())),
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

impl GoogClient {
    pub fn new(
        client_type: ClientType,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        creds: Credentials,
    ) -> anyhow::Result<Self> {
        let endpoint = match client_type {
            ClientType::Calendar => "https://www.googleapis.com/calendar/v3".to_string(),
            ClientType::Drive => "https://www.googleapis.com/drive/v3".to_string(),
            ClientType::Sheets => "https://sheets.googleapis.com/v4".to_string(),
        };

        let params = OAuthParams {
            client_id: client_id.to_string(),
            client_secret: Some(client_secret.to_string()),
            redirect_url: Some(redirect_url.to_string()),
            auth_url: AUTH_URL.to_string(),
            token_url: Some(TOKEN_URL.to_string()),
            revoke_url: Some(REVOKE_URL.to_string()),
        };

        let (tx, rx) = watch::channel(creds.clone());
        Ok(GoogClient {
            client_type,
            endpoint,
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            credentials: creds,
            on_refresh_tx: tx,
            on_refresh_rx: rx,
        })
    }

    pub async fn download_file(&mut self, file_id: &str) -> Result<Bytes> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files/");
        endpoint.push_str(file_id);

        let file_info = self.get_file_metadata(file_id).await?;
        let mut params = Vec::new();
        // If Google specific file, we need to export
        if file_info
            .mime_type
            .starts_with("application/vnd.google-apps")
        {
            endpoint.push_str("/export");
            let export_type = match FileType::from_str(file_info.mime_type.as_str()) {
                Ok(FileType::Document) => "text/plain",
                // Excel
                Ok(FileType::Spreadsheet) => {
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                }
                Ok(FileType::Presentation) => "text/plain",
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

    pub async fn list_files(
        &mut self,
        next_page: Option<String>,
        query: Option<String>,
    ) -> Result<Files, ApiError> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files");

        let mut params = if let Some(next_page) = next_page {
            vec![("pageToken".to_string(), next_page)]
        } else {
            Vec::new()
        };

        if let Some(query) = query {
            params.push(("q".to_string(), query));
        }

        params.push(("orderBy".to_string(), "viewedByMeTime desc".to_string()));
        serde_json::from_value::<Files>(self.call_json(&endpoint, &params).await?)
            .map_err(ApiError::SerdeError)
    }

    pub async fn get_file_metadata(&mut self, id: &str) -> Result<File, ApiError> {
        let mut endpoint = self.endpoint.to_string();
        endpoint.push_str("/files/");
        endpoint.push_str(id);

        let params = vec![(
            "fields".to_string(),
            vec![
                "kind",
                "id",
                "name",
                "mimeType",
                "description",
                "starred",
                "parents",
                "version",
                "sharingUser",
                "lastModifyingUser",
                "webViewLink",
                "createdTime",
                "modifiedTime",
                "sharedWithMeTime",
            ]
            .join(","),
        )];

        serde_json::from_value::<File>(self.call_json(&endpoint, &params).await?)
            .map_err(ApiError::SerdeError)
    }

    /// User associated with this credential
    pub async fn get_user(&mut self) -> Result<GoogUser, ApiError> {
        let endpoint = "https://www.googleapis.com/oauth2/v3/userinfo";
        serde_json::from_value::<GoogUser>(self.call_json(endpoint, &Vec::new()).await?)
            .map_err(ApiError::SerdeError)
    }
}
