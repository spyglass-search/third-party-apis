use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::TokenResponse;
pub use oauth2::{AccessToken, RefreshToken};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl};
use oauth2::{CsrfToken, PkceCodeChallenge};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{AUTH_URL, REVOKE_URL, TOKEN_URL};

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
    let value = header::HeaderValue::from_str(&format!("Bearer {}", token))?;
    headers.insert("Authorization", value);

    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

pub fn oauth_client(client_id: &str, client_secret: &str, redirect_url: &str) -> BasicClient {
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id.into()),
        Some(ClientSecret::new(client_secret.into())),
        auth_url,
        Some(token_url),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string()).expect("Invalid redirect URL"))
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new(REVOKE_URL.to_string()).expect("Invalid revocation endpoint URL"),
    )
}
