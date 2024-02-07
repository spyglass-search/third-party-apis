use anyhow::{anyhow, Result};
use async_trait::async_trait;
use libauth::{
    auth_http_client, oauth_client, ApiClient, ApiError, AuthorizationRequest, AuthorizeOptions,
    Credentials, OAuthParams,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};

use reqwest::Client;
use serde_json::Value;
use tokio::sync::watch;

pub mod types;

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

const API_ENDPOINT: &str = "https://graph.microsoft.com/v1.0";

pub struct MicrosoftClient {
    pub credentials: Credentials,
    http: Client,
    pub oauth: BasicClient,
    pub on_refresh_tx: watch::Sender<Credentials>,
    pub on_refresh_rx: watch::Receiver<Credentials>,
    pub username: Option<String>,
}

#[async_trait]
impl ApiClient for MicrosoftClient {
    fn id(&self) -> String {
        "graph.microsoft.com".to_string()
    }

    async fn account_id(&mut self) -> Result<String> {
        if let Some(username) = &self.username {
            Ok(username.clone())
        } else {
            let name = self.get_user().await?.display_name;
            self.username = Some(name.clone());
            Ok(name)
        }
    }

    async fn account_metadata(&mut self) -> Option<Value> {
        None
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

    fn authorize(&self, scopes: &[String], _: &AuthorizeOptions) -> AuthorizationRequest {
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let scopes = scopes
            .iter()
            .map(|s| Scope::new(s.to_string()))
            .collect::<Vec<Scope>>();

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = self
            .oauth
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_pkce_challenge(pkce_code_challenge.clone())
            .url();

        AuthorizationRequest {
            url: authorize_url,
            csrf_token: csrf_state,
            pkce_challenge: Some(pkce_code_challenge),
            pkce_verifier: Some(pkce_code_verifier.secret().to_string()),
        }
    }

    async fn token_exchange(
        &self,
        code: &str,
        pkce_verifier: Option<String>,
    ) -> Result<BasicTokenResponse> {
        let code = AuthorizationCode::new(code.to_owned());

        let mut exchange = self.oauth.exchange_code(code);
        if let Some(pkce_verifier) = pkce_verifier {
            exchange = exchange.set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier));
        }

        match exchange.request_async(Self::http_client).await {
            Ok(val) => Ok(val),
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    async fn refresh_credentials(&mut self) -> Result<()> {
        if let Some(refresh_token) = &self.credentials.refresh_token {
            let new_token = self
                .oauth
                .exchange_refresh_token(refresh_token)
                .request_async(Self::http_client)
                .await?;

            self.credentials.refresh_token(&new_token);
            self.http = auth_http_client(new_token.access_token().secret())?;
            // Let any listeners know the credentials have been updated.
            self.on_refresh_tx.send(self.credentials.clone())?;
        }

        Ok(())
    }
}

impl MicrosoftClient {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        creds: Credentials,
    ) -> anyhow::Result<Self> {
        let params = OAuthParams {
            client_id: client_id.to_owned(),
            client_secret: Some(client_secret.to_owned()),
            redirect_url: Some(redirect_url.to_owned()),
            auth_url: AUTH_URL.to_string(),
            token_url: Some(TOKEN_URL.to_string()),
            ..Default::default()
        };

        let (tx, rx) = watch::channel(creds.clone());

        Ok(MicrosoftClient {
            credentials: creds.clone(),
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            on_refresh_tx: tx,
            on_refresh_rx: rx,
            username: None,
        })
    }

    pub async fn get_user(&mut self) -> Result<types::User, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/me");

        let resp = self.call_json(&endpoint, &Vec::new()).await?;
        serde_json::from_value::<types::User>(resp).map_err(ApiError::SerdeError)
    }

    pub async fn http_client(
        request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
        oauth2::reqwest::async_http_client(request).await
    }

    pub async fn get_task_lists(&mut self) -> Result<types::TaskLists, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/me/todo/lists");

        let resp = self.call_json(&endpoint, &Vec::new()).await?;
        serde_json::from_value::<types::TaskLists>(resp).map_err(ApiError::SerdeError)
    }

    pub async fn get_tasks(
        &mut self,
        task_list_id: &str,
    ) -> Result<types::TaskListTasks, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str(format!("/me/todo/lists/{}/tasks", task_list_id).as_str());

        let resp = self.call_json(&endpoint, &Vec::new()).await?;
        serde_json::from_value::<types::TaskListTasks>(resp).map_err(ApiError::SerdeError)
    }

    pub async fn add_task(
        &mut self,
        task_list_id: &str,
        task: types::Task,
    ) -> Result<types::Task, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str(format!("/me/todo/lists/{}/tasks", task_list_id).as_str());

        let resp = self
            .post_json(&endpoint, serde_json::to_value(&task).unwrap())
            .await?;
        serde_json::from_value::<types::Task>(resp).map_err(ApiError::SerdeError)
    }

    pub async fn create_task_list(
        &mut self,
        task_list: types::CreateTaskList,
    ) -> Result<types::TaskListsDef, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/me/todo/lists");

        let resp = self
            .post_json(&endpoint, serde_json::to_value(&task_list).unwrap())
            .await?;
        serde_json::from_value::<types::TaskListsDef>(resp).map_err(ApiError::SerdeError)
    }
}
