use bytes::Bytes;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenResponse, TokenUrl,
};
use reqwest::{header, Client};
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope};
use serde::{Deserialize, Serialize};
use url::Url;

const API_ENDPOINT: &str = "https://www.googleapis.com/drive/v3";

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub requested_at: DateTime<Utc>,
    pub token: BasicTokenResponse,
}

impl Credentials {
    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.token.expires_in() {
            let now = Utc::now();
            let dur = chrono::Duration::from_std(duration).expect("Unable to convert duration");
            return (now - self.requested_at) > dur;
        }

        false
    }

    pub fn refresh_token(&mut self, token: &BasicTokenResponse) {
        self.requested_at = Utc::now();
        self.token = token.clone();
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        std::fs::write(path, serde_json::to_string(self)?)?;
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct File {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub description: String,
    pub starred: bool,
    pub trashed: bool,
    pub parents: Vec<String>,
    pub properties: HashMap<String, String>,
    pub spaces: Vec<String>,
    pub version: String,
    pub web_content_link: String,
    pub web_view_link: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileInfo {
    pub kind: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct Files {
    #[serde(rename = "nextPageToken")]
    pub next_page_token: String,
    pub files: Vec<FileInfo>,
}

pub fn auth_http_client(token: &str) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    let value = header::HeaderValue::from_str(&format!("Bearer {}", token))?;
    headers.insert("Authorization", value);

    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

pub struct GoogClient {
    endpoint: Url,
    http: Client,
    oauth: BasicClient,
    credentials: Credentials,
}

impl GoogClient {
    pub fn new(client_id: &str, client_secret: &str, creds: Credentials) -> anyhow::Result<Self> {
        Ok(GoogClient {
            endpoint: Url::parse(API_ENDPOINT).expect("Invalid API endpoint"),
            http: auth_http_client(creds.token.access_token().secret())?,
            oauth: Self::oauth_client(client_id, client_secret),
            credentials: creds,
        })
    }

    pub fn oauth_client(client_id: &str, client_secret: &str) -> BasicClient {
        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL");

        let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
            .expect("Invalid token endpoint URL");

        BasicClient::new(
            ClientId::new(client_id.into()),
            Some(ClientSecret::new(client_secret.into())),
            auth_url,
            Some(token_url),
        )
        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_uri(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        )
    }

    pub async fn call(
        &mut self,
        endpoint: &str,
        query: &Vec<(String, String)>,
    ) -> Result<reqwest::Response> {
        // See if the token is expired
        if self.credentials.is_expired() {
            println!("Refreshing expired token");
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
        if let Some(refresh_token) = self.credentials.token.refresh_token() {
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
}

pub async fn request_token(client: BasicClient) -> Option<BasicTokenResponse> {
    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
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
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!("Open this URL in your browser:\n{}\n", authorize_url);

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    if let Some(mut stream) = listener.incoming().flatten().next() {
        let code;
        let state;
        {
            let mut reader = BufReader::new(&stream);

            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let redirect_url = request_line.split_whitespace().nth(1).unwrap();
            let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

            let code_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .unwrap();

            let (_, value) = code_pair;
            code = AuthorizationCode::new(value.into_owned());

            let state_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "state"
                })
                .unwrap();

            let (_, value) = state_pair;
            state = CsrfToken::new(value.into_owned());
        }

        let message = "Go back to your terminal :)";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes()).unwrap();

        println!("Google returned the following code:\n{}\n", code.secret());
        println!(
            "Google returned the following state:\n{} (expected `{}`)\n",
            state.secret(),
            csrf_state.secret()
        );

        // Exchange the code with a token.
        let token_resp = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
            .request_async(async_http_client);

        token_resp.await.ok()
    } else {
        None
    }
}
