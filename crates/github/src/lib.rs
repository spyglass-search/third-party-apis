use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use libauth::auth_http_client;
use libauth::{oauth_client, ApiClient, AuthorizationRequest, Credentials, OAuthParams};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use reqwest::Client;
use strum_macros::{Display, EnumString};

mod types;

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

const API_ENDPOINT: &str = "https://api.github.com";

pub type OnRefreshFn = Box<dyn FnMut(&Credentials) + Send + Sync + 'static>;

/// Github scopes taken from: https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps
#[derive(Debug, Display, EnumString)]
pub enum AuthScopes {
    /// Full access to repo
    #[strum(serialize = "repo")]
    Repo,
    /// Grants read/write access to commit statuses in public/private repos
    #[strum(serialize = "repo:status")]
    RepoStatus,
    /// Grants access to deployment statuses for public/private repos.
    #[strum(serialize = "repo_deployment")]
    RepoDeployment,
    /// Limits access to public repos. Read/write access to code, commit statuses,
    /// projects, collaborators, and deployment statuses. Also required for starring
    /// public repos.
    #[strum(serialize = "public_repo")]
    PublicRepo,
    /// Grants read/write access to profile info only.
    #[strum(serialize = "user")]
    User,
}

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

#[derive(Debug)]
pub struct GithubRepo {
    pub name: String,
    pub num_stars: u32,
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

    pub async fn get_user(&mut self) -> Result<()> {
        let resp = self.call("https://api.github.com/user", &Vec::new()).await?;
        match resp.text().await {
            Ok(res) => println!("{:?}", res),
            Err(err) => println!("Unable to make request: {}", err),
        }

        Ok(())
    }

    pub async fn list_stars(&self) -> Result<Vec<GithubRepo>> {
        Ok(Vec::new())
    }
}
