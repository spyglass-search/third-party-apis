use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use libauth::{
    auth_http_client, oauth_client, ApiClient, AuthorizationRequest, Credentials, OAuthParams,
    OnRefreshFn,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::http::HeaderMap;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use reqwest::Client;

pub mod types;
use serde::de::DeserializeOwned;
use types::ApiResponse;

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

const API_ENDPOINT: &str = "https://api.github.com";

pub struct GithubClient {
    pub credentials: Credentials,
    http: Client,
    pub oauth: BasicClient,
    pub on_refresh: OnRefreshFn,
}

#[async_trait]
impl ApiClient for GithubClient {
    fn id(&self) -> String {
        "api.github.com".to_string()
    }

    async fn account_id(&mut self) -> Result<String> {
        let user = self.get_user().await?;
        Ok(user.login)
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

    fn set_on_refresh(&mut self, callback: impl FnMut(&Credentials) + Send + Sync + 'static) {
        self.on_refresh = Box::new(callback);
    }

    fn authorize(&self, scopes: &[String]) -> AuthorizationRequest {
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
            pkce_challenge: pkce_code_challenge,
            pkce_verifier: pkce_code_verifier.secret().to_string(),
        }
    }

    async fn token_exchange(&self, code: &str, pkce_verifier: &str) -> Result<BasicTokenResponse> {
        let code = AuthorizationCode::new(code.to_owned());

        match self
            .oauth
            .exchange_code(code)
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_owned()))
            .request_async(async_http_client)
            .await
        {
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
            (self.on_refresh)(&self.credentials);
        }

        Ok(())
    }
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
            auth_url: AUTH_URL.to_string(),
            token_url: Some(TOKEN_URL.to_string()),
            ..Default::default()
        };

        Ok(GithubClient {
            credentials: creds.clone(),
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            on_refresh: Box::new(|_| {}),
        })
    }

    fn has_next(&self, headers: &HeaderMap) -> bool {
        if let Some(link) = headers.get("link") {
            let value = link.to_str().unwrap_or_default();
            return value.contains("rel=\"next\"");
        }

        false
    }

    /// Handle pagination through Github API results
    async fn paginate<T>(
        &mut self,
        endpoint: &str,
        page: Option<u32>,
        query: &Vec<(String, String)>,
    ) -> Result<ApiResponse<T>>
    where
        T: DeserializeOwned,
    {
        let mut query = query.to_owned();
        query.push(("page".to_string(), page.unwrap_or(1).to_string()));

        let resp = self.call(endpoint, &query).await?;
        let next_page = if self.has_next(resp.headers()) {
            Some(page.unwrap_or(1) + 1)
        } else {
            None
        };

        match resp.json().await {
            Ok(result) => Ok(ApiResponse { next_page, result }),
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    pub async fn get_issue(&mut self, issue_or_url: &str) -> Result<types::Issue> {
        let endpoint = if issue_or_url.starts_with("https://api.github.com/repos") {
            issue_or_url.to_string()
        } else {
            format!("{API_ENDPOINT}/repos/{issue_or_url}")
        };

        self.call_json::<types::Issue>(&endpoint, &Vec::new()).await
    }

    pub async fn get_repo(&mut self, repo_or_url: &str) -> Result<types::Repo> {
        let endpoint = if repo_or_url.starts_with("https://api.github.com/repos") {
            repo_or_url.to_string()
        } else {
            format!("{API_ENDPOINT}/repos/{repo_or_url}")
        };

        self.call_json::<types::Repo>(&endpoint, &Vec::new()).await
    }

    pub async fn get_user(&mut self) -> Result<types::User> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/user");
        self.call_json::<types::User>(&endpoint, &Vec::new()).await
    }

    pub async fn list_issues(
        &mut self,
        page: Option<u32>,
    ) -> Result<ApiResponse<Vec<types::Issue>>> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/issues");
        let params = vec![("filter".to_string(), "all".to_string())];

        self.paginate(&endpoint, page, &params).await
    }

    pub async fn list_repos(&mut self, page: Option<u32>) -> Result<ApiResponse<Vec<types::Repo>>> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/user/repos");
        self.paginate(&endpoint, page, &Vec::new()).await
    }

    pub async fn list_starred(
        &mut self,
        page: Option<u32>,
    ) -> Result<ApiResponse<Vec<types::Repo>>> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/user/starred");
        self.paginate(&endpoint, page, &Vec::new()).await
    }
}
