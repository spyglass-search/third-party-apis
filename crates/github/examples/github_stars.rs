use dotenv_codegen::dotenv;

use libauth::helpers::load_credentials;
use libgithub::types::AuthScopes;
use libgithub::GithubClient;

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_id = dotenv!("GITHUB_CLIENT_ID");
    let client_secret = dotenv!("GITHUB_CLIENT_SECRET");

    let mut client = GithubClient::new(client_id, client_secret, REDIRECT_URL, Default::default())?;

    let scopes = vec![AuthScopes::Repo.to_string(), AuthScopes::User.to_string()];
    load_credentials(&mut client, &scopes).await;

    let user = client.get_user().await?;
    println!("Authenticated w/ {}", user.login);

    println!("\nListing starred repos:");
    println!("------------------------------");
    let repos = client.list_starred(None).await?;
    println!("\nnext_page: {:?}", repos.next_page);
    for repo in repos.result.iter().take(5) {
        println!("Name: {}", repo.full_name);
        println!("URL: {}", repo.html_url);
        println!("Desc: {}", repo.description.clone().unwrap_or_default());
        println!("---")
    }

    println!("\nListing user's repos:");
    println!("------------------------------");
    let repos = client.list_repos(None).await?;
    println!("\nnext_page: {:?}", repos.next_page);
    for repo in repos.result.iter().take(5) {
        println!("Name: {}", repo.full_name);
        println!("URL: {}", repo.html_url);
        println!("Desc: {}", repo.description.clone().unwrap_or_default());
        println!("---")
    }

    println!("\nListing users issues:");
    println!("------------------------------");
    let mut page = Some(1);
    while let Ok(issues) = client.list_issues(page).await {
        page = issues.next_page;
        println!("next_page: {:?}", issues.next_page);
        for issue in issues.result.iter().take(5) {
            println!("REPO:\t{}", issue.repository.full_name);
            println!("TITLE:\t{}", issue.title);

            let issue_txt = issue.to_text();
            if !issue_txt.is_empty() {
                let max_len = issue_txt.len() - 1;
                println!("TEXT:\t{}", &issue_txt[..max_len.min(32)]);
                println!("---")
            }
        }

        if page.is_none() {
            break;
        }
    }

    // Example retreiving a single repo
    client
        .get_repo("octocat/Hello-World")
        .await
        .expect("Unable to get repo");
    // Example retrieving a single issue
    client
        .get_issue("octocat/Hello-World/issues/1")
        .await
        .expect("Unable to get issue");

    Ok(())
}
