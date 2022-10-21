use std::path::Path;

use chrono::Utc;
use libgoog::{request_token, Credentials, GoogClient};

const GOOGLE_CLIENT_ID: &str =
    "621713166215-621sdvu6vhj4t03u536p3b2u08o72ndh.apps.googleusercontent.com";
const GOOGLE_CLIENT_SECRET: &str = "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";

const SAVED_CREDS: &str = "saved.json";

// fn revoke(client: BasicClient, access_token: AccessToken, refresh_token: Option<RefreshToken>) {
//     // Revoke the obtained token
//     let token_to_revoke: StandardRevocableToken = match refresh_token {
//         Some(token) => token.into(),
//         None => access_token.into(),
//     };

//     client
//         .revoke_token(token_to_revoke)
//         .unwrap()
//         .request(http_client)
//         .expect("Failed to revoke token");
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = Path::new(SAVED_CREDS);
    let credentials = if path.exists() {
        let saved: Credentials = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Unable to deserialize token");

        saved
    } else {
        let client = GoogClient::oauth_client(GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET);
        let token = request_token(client)
            .await
            .expect("Unable to request token");

        let saved = Credentials {
            requested_at: Utc::now(),
            token,
        };
        let _ = saved.save();

        saved
    };

    let mut client = GoogClient::new(GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, credentials)?;
    let files = client.list_files().await?;

    let mut count = 0;
    for file in files.files {
        // NOTE: None of the timestamps seem like they're available but the sort
        // works. Name of the file exists, and being able to pull data about the file
        // works as well.
        // println!("{}- {}", file.name, file.mime_type);
        // if let Ok(content) = client.get_file("adfadf").await {
        //     println!("{:?}", content);
        // }
        println!("{:?}", file);
        match client.get_file(&file.id).await {
            Ok(content) => println!("details: {:?}", content),
            Err(err) => println!("{}", err.to_string())
        }

        count += 1;
        if count >= 5 {
            break;
        }
    }

    println!("Number of files: {}", count);

    Ok(())
}
