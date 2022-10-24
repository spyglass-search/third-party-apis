use dotenv::dotenv;
use dotenv_codegen::dotenv;
use std::path::Path;

use chrono::Utc;
use libgoog::{request_token, Credentials, GoogClient};

const SAVED_CREDS: &str = "saved.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let path = Path::new(SAVED_CREDS);
    let credentials = if path.exists() {
        let saved: Credentials = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Unable to deserialize token");

        saved
    } else {
        let client = GoogClient::oauth_client(google_client_id, google_client_secret);
        let token = request_token(client)
            .await
            .expect("Unable to request token");

        let saved = Credentials {
            requested_at: Utc::now(),
            token,
        };
        let _ = saved.save_to_file(path.to_path_buf());

        saved
    };

    let mut client = GoogClient::new(google_client_id, google_client_secret, credentials)?;
    let files = client.list_files().await?;

    let mut count = 0;
    for file in files.files {
        println!("{:?}", file);
        match client.get_file_metadata(&file.id).await {
            Ok(content) => {
                println!("details: {:?}", content);
                if let Ok(content) = client.download_file(&file).await {
                    println!("read {} bytes", content.len());
                }
            }
            Err(err) => println!("{}", err),
        }

        count += 1;
        if count >= 5 {
            break;
        }
    }

    println!("Number of files: {}", count);

    Ok(())
}
