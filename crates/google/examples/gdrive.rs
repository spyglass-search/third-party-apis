use dotenv::dotenv;
use dotenv_codegen::dotenv;

use libauth::helpers::load_credentials;
use libgoog::{types::AuthScope, ClientType, GoogClient};

const REDIRECT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let scopes = vec![
        AuthScope::Drive.to_string(),
        AuthScope::DriveMetadata.to_string(),
    ];

    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let mut client = GoogClient::new(
        ClientType::Drive,
        google_client_id,
        google_client_secret,
        REDIRECT_URL,
        Default::default(),
    )?;

    load_credentials(&mut client, &scopes).await;

    let files = client.list_files(None, None).await?;

    println!("------------------------------");
    println!("next_page: {:?}", files.next_page_token);
    println!("------------------------------");

    println!("Listing some example files:");
    println!("------------------------------");
    for file in files.files.iter().take(5) {
        println!("{:?}", file);
        match client.get_file_metadata(&file.id).await {
            Ok(content) => {
                println!("details: {:?}", content);
                // if let Ok(content) = client.download_file(&file.id).await {
                //     println!("read {} bytes", content.len());
                // }
                println!("----------")
            }
            Err(err) => println!("Unable to get file data: {}", err),
        }
    }

    Ok(())
}
