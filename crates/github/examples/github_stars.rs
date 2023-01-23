use dotenv::dotenv;
use dotenv_codegen::dotenv;

use libauth::helpers::load_credentials;
use libgithub::{AuthScopes, GithubClient};

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let client_id = dotenv!("GITHUB_CLIENT_ID");
    let client_secret = dotenv!("GITHUB_CLIENT_SECRET");

    let mut client = GithubClient::new(client_id, client_secret, REDIRECT_URL, Default::default())?;

    let scopes = vec![AuthScopes::Repo.to_string(), AuthScopes::User.to_string()];
    load_credentials(&mut client, &scopes).await;

    client.get_user().await?;

    let repos = client.list_stars().await?;

    println!("Listing some example repos:");
    println!("------------------------------");
    for repo in repos.iter().take(5) {
        println!("{:?}", repo);
    }

    Ok(())
}
