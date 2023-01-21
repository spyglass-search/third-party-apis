use oauth2::basic::BasicClient;
use oauth2::TokenResponse;
use reqwest::Client;

use libauth::{auth_http_client, oauth_client, Credentials, OAuthParams};

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
pub type OnRefreshFn = Box<dyn FnMut(&Credentials) + Send + Sync + 'static>;

pub struct GithubClient {
    pub credentials: Credentials,
    http: Client,
    pub oauth: BasicClient,
    pub on_refresh: OnRefreshFn,
}

impl GithubClient {
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
            ..Default::default()
        };

        Ok(GithubClient {
            credentials: creds.clone(),
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            on_refresh: Box::new(|_| {}),
        })
    }

    pub fn set_on_refresh(&mut self, callback: impl FnMut(&Credentials) + Send + Sync + 'static) {
        self.on_refresh = Box::new(callback);
    }
}
