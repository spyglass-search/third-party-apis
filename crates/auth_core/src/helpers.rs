use anyhow::anyhow;
use oauth2::basic::BasicTokenResponse;
use oauth2::CsrfToken;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use url::Url;

use crate::AuthorizeOptions;

use super::{ApiClient, Credentials};

const SAVED_CREDS_DIR: &str = "credentials";

/// Helper function to load saved credentials from the filesystem.
/// SHOULD ONLY BE USED FOR EXAMPLES AND TESTS
pub async fn load_credentials(client: &mut impl ApiClient, scopes: &[String]) {
    let dir = Path::new(SAVED_CREDS_DIR);
    if !dir.exists() {
        let _ = std::fs::create_dir_all(SAVED_CREDS_DIR);
    }

    let file_name = format!("{}.json", client.id());
    let path = dir.join(file_name);

    // Setup a refresh callback when a new token is needed.
    // When called this will save the new credentials to disk
    {
        let path = path.clone();
        client.set_on_refresh(move |new_creds| {
            println!(
                "Received new access code: {}",
                new_creds.access_token.secret()
            );
            // save new credentials.
            let _ = new_creds.save_to_file(path.to_path_buf());
        });
    }

    // Load from file system (if exists) or run through token authorization process.
    let credentials = if path.exists() {
        let saved: Credentials = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Unable to deserialize token");

        saved
    } else {
        let token = get_token(client, scopes)
            .await
            .expect("Unable to request token");

        let mut saved = Credentials::default();
        saved.refresh_token(&token);
        let _ = saved.save_to_file(path.to_path_buf());
        saved
    };

    let _ = client.set_credentials(&credentials);
}

/// Runs a ephemeral HTTP server that waits for OAuth to call the redirect_url
pub async fn get_token(
    client: &impl ApiClient,
    scopes: &[String],
) -> anyhow::Result<BasicTokenResponse> {
    let options = AuthorizeOptions {
        pkce: true,
        ..Default::default()
    };
    let request = client.authorize(scopes, &options);
    println!("Open this URL in your browser:\n{}\n", request.url);

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    if let Some(mut stream) = listener.incoming().flatten().next() {
        let code;
        let state;
        {
            let mut reader = BufReader::new(&stream);

            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let redirect_url = request_line.split_whitespace().nth(1).unwrap();
            let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

            let code_pair = url
                .query_pairs()
                .find(|pair| {
                    let (key, _) = pair;
                    key == "code"
                })
                .expect("`code` was not found in query");

            let (_, value) = code_pair;
            code = value.into_owned();

            let state_pair = url
                .query_pairs()
                .find(|pair| {
                    let (key, _) = pair;
                    key == "state"
                })
                .expect("`state` was not found in query");

            let (_, value) = state_pair;
            state = CsrfToken::new(value.into_owned());
        }

        let message = "Go back to your terminal :)";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes()).unwrap();

        println!("{} returned the following code:\n{}\n", client.id(), code);
        println!(
            "{} returned the following state:\n{} (expected `{}`)\n",
            client.id(),
            state.secret(),
            request.csrf_token.secret()
        );

        client.token_exchange(&code, request.pkce_verifier).await
    } else {
        Err(anyhow!("Invalid request"))
    }
}
