use dotenv_codegen::dotenv;
use libauth::helpers::load_credentials;
use libreddit::types::{AuthScopes, Post};
use libreddit::RedditClient;

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

fn print_posts(posts: &[Post]) {
    for post in posts.iter() {
        println!(
            "{}\n{:?}\nScore: {}\nhttps://www.reddit.com{}",
            post.created_utc, post.title, post.score, post.permalink
        );
        println!("----------------")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_id = dotenv!("REDDIT_CLIENT_ID");
    let client_secret = dotenv!("REDDIT_CLIENT_SECRET");

    let mut client = RedditClient::new(client_id, client_secret, REDIRECT_URL, Default::default())?;
    let scopes = vec![
        AuthScopes::Identity.to_string(),
        AuthScopes::History.to_string(),
        AuthScopes::MySubreddits.to_string(),
        AuthScopes::Read.to_string(),
    ];
    load_credentials(&mut client, &scopes, true).await;

    let user = client.get_user().await?;
    println!("Authenticated as user: {}", user.name);

    println!("\nSaved Posts");
    println!("=====================");
    let saved = client.list_saved(None, 2).await?;
    print_posts(&saved.data);

    let saved = client.list_saved(saved.after, 2).await?;
    print_posts(&saved.data);

    println!("\n\nUpvoted Posts");
    println!("=====================");
    let upvoted = client.list_upvoted(None, 5).await?;
    print_posts(&upvoted.data);

    println!("\n\nGrabbing Single Post");
    println!("=====================");
    let post = client
        .get_post("t3_yc0bee")
        .await?
        .expect("Post should exist");
    print_posts(&[post]);

    Ok(())
}
