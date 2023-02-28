use dotenv_codegen::dotenv;
use libauth::helpers::load_credentials;
use libreddit::types::AuthScopes;
use libreddit::RedditClient;

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_id = dotenv!("REDDIT_CLIENT_ID");
    let client_secret = dotenv!("REDDIT_CLIENT_SECRET");

    let mut client = RedditClient::new(client_id, client_secret, REDIRECT_URL, Default::default())?;
    let scopes = vec![
        AuthScopes::Identity.to_string(),
        AuthScopes::History.to_string(),
        AuthScopes::MySubreddits.to_string(),
    ];
    load_credentials(&mut client, &scopes).await;

    let user = client.get_user().await?;
    dbg!(user);
    Ok(())
}
