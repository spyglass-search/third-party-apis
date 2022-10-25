use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;

use dotenv::dotenv;
use dotenv_codegen::dotenv;
use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier};
use url::Url;

use libgoog::{Credentials, GoogClient};

const SAVED_CREDS: &str = "saved.json";
const REDIRECT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let path = Path::new(SAVED_CREDS);

    let mut client = GoogClient::new(
        google_client_id,
        google_client_secret,
        REDIRECT_URL,
        Default::default(),
    )?;

    let credentials = if path.exists() {
        let saved: Credentials = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Unable to deserialize token");

        saved
    } else {
        let token = get_token(&client).await.expect("Unable to request token");

        let mut saved = Credentials::default();
        saved.refresh_token(&token);
        let _ = saved.save_to_file(path.to_path_buf());

        saved
    };
    let _ = client.set_credentials(&credentials);

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

pub async fn get_token(client: &GoogClient) -> Option<BasicTokenResponse> {
    let request = client.authorize();
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
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .unwrap();

            let (_, value) = code_pair;
            code = AuthorizationCode::new(value.into_owned());

            let state_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "state"
                })
                .unwrap();

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

        println!("Google returned the following code:\n{}\n", code.secret());
        println!(
            "Google returned the following state:\n{} (expected `{}`)\n",
            state.secret(),
            request.csrf_token.secret()
        );

        // Exchange the code with a token.
        let token_resp = client
            .oauth
            .exchange_code(code)
            .set_pkce_verifier(PkceCodeVerifier::new(request.pkce_verifier.clone()))
            .request_async(async_http_client);

        token_resp.await.ok()
    } else {
        None
    }
}
