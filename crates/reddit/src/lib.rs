use anyhow::{anyhow, Result};
use async_trait::async_trait;
use libauth::{
    auth_http_client, oauth_client, ApiClient, ApiError, AuthorizationRequest, AuthorizeOptions,
    Credentials, OAuthParams, OnRefreshFn,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};

use reqwest::Client;
use types::{ApiResponse, DataWrapper, Listing, Post};

pub mod types;

const AUTH_URL: &str = "https://www.reddit.com/api/v1/authorize";
const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";

const API_ENDPOINT: &str = "https://oauth.reddit.com";

pub struct RedditClient {
    pub credentials: Credentials,
    http: Client,
    pub oauth: BasicClient,
    pub on_refresh: OnRefreshFn,
    pub username: Option<String>,
}

#[async_trait]
impl ApiClient for RedditClient {
    fn id(&self) -> String {
        "oauth.reddit.com".to_string()
    }

    async fn account_id(&mut self) -> Result<String> {
        if let Some(username) = &self.username {
            Ok(username.clone())
        } else {
            let name = self.get_user().await?.name;
            self.username = Some(name.clone());
            Ok(name)
        }
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
            // request a refresh token from Reddit
            .add_extra_param("duration", "permanent")
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
            (self.on_refresh)(&self.credentials);
        }

        Ok(())
    }
}

impl RedditClient {
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

        Ok(RedditClient {
            credentials: creds.clone(),
            http: auth_http_client(creds.access_token.secret())?,
            oauth: oauth_client(&params),
            on_refresh: Box::new(|_| {}),
            username: None,
        })
    }

    pub async fn get_user(&mut self) -> Result<types::User, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/api/v1/me");

        self.call_json::<types::User>(&endpoint, &Vec::new()).await
    }

    pub async fn http_client(
        mut request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
        request.headers.insert(
            "User-Agent",
            "desktop:com.athlabs.spyglass:v0.0.1 (by /u/andyndino)"
                .parse()
                .unwrap(),
        );
        oauth2::reqwest::async_http_client(request).await
    }

    async fn paginate(
        &mut self,
        endpoint: &str,
        query: &[(String, String)],
    ) -> Result<ApiResponse<Vec<Post>>, ApiError> {
        let listing = self
            .call_json::<types::DataWrapper<Listing<DataWrapper<Post>>>>(endpoint, query)
            .await?;

        let after = listing.data.after;
        let posts = listing
            .data
            .children
            .iter()
            .map(|x| x.data.to_owned())
            .collect::<Vec<_>>();

        Ok(ApiResponse { after, data: posts })
    }

    pub async fn get_post(&mut self, id: &str) -> Result<Option<Post>, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        endpoint.push_str("/api/info");

        let query = vec![("id".into(), id.into())];

        let resp = self.paginate(&endpoint, &query).await?;
        if let Some(post) = resp.data.get(0) {
            Ok(Some(post.to_owned()))
        } else {
            Ok(None)
        }
    }

    pub async fn list_saved(
        &mut self,
        after: Option<String>,
        limit: usize,
    ) -> Result<ApiResponse<Vec<Post>>, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        let username = self.account_id().await?;
        endpoint.push_str(&format!("/user/{}/saved", username));

        let mut query = vec![
            // for all time
            ("t".into(), "all".into()),
            // Make sure limit is at least 1 & at most 100
            ("limit".into(), limit.max(1).min(100).to_string()),
        ];
        if let Some(after) = after {
            query.push(("after".into(), after));
        }

        self.paginate(&endpoint, &query).await
    }

    pub async fn list_upvoted(
        &mut self,
        after: Option<String>,
        limit: usize,
    ) -> Result<ApiResponse<Vec<Post>>, ApiError> {
        let mut endpoint = API_ENDPOINT.to_string();
        let username = self.account_id().await?;
        endpoint.push_str(&format!("/user/{}/upvoted", username));

        let mut query = vec![
            // for all time
            ("t".into(), "all".into()),
            // Make sure limit is at least 1 & at most 100
            ("limit".into(), limit.max(1).min(100).to_string()),
        ];

        if let Some(after) = after {
            query.push(("after".into(), after));
        }

        self.paginate(&endpoint, &query).await
    }
}
