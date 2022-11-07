use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;

use dotenv::dotenv;
use dotenv_codegen::dotenv;
use oauth2::CsrfToken;
use url::Url;

use libgoog::{types::AuthScope, ClientType, Credentials, GoogClient};

const SAVED_CREDS: &str = "saved-calendar.json";
const REDIRECT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = Path::new(SAVED_CREDS);

    dotenv().ok();

    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let mut client = GoogClient::new(
        ClientType::Calendar,
        google_client_id,
        google_client_secret,
        REDIRECT_URL,
        Default::default(),
    )?;
    client.set_on_refresh(move |new_creds| {
        println!(
            "Received new access code: {}",
            new_creds.access_token.secret()
        );
        // save new credentials.
        let _ = new_creds.save_to_file(path.to_path_buf());
    });

    load_credentials(&mut client).await;

    let user = client.get_user().await;
    println!("AUTHORIZED USER: {:?}", user);

    let cals = client.list_calendars(None).await?;
    println!("------------------------------");
    println!("next_page: {:?}", cals.next_page_token);

    println!("\n------------------------------");
    println!("PRIMARY CALENDAR");
    let primary_events = client.list_calendar_events("primary", None).await?;
    for event in primary_events.items.iter().take(5) {
        println!(
            "EVENT: {} {} ({} attendees)",
            event
                .start
                .date_time
                .map_or(event.start.date.clone(), |d| d.to_rfc3339()),
            event.summary,
            event.attendees.len()
        );
    }
    println!("------------------------------");

    for cal in cals.items.iter().take(5) {
        println!("CALENDAR: {} ({})", cal.summary, cal.id);
        if let Ok(events) = client.list_calendar_events(&cal.id, None).await {
            for event in events.items.iter().take(5) {
                if let Ok(data) = client.get_calendar_event(&cal.id, &event.id).await {
                    println!(
                        "EVENT: {} {} ({} attendees)",
                        data.start
                            .date_time
                            .map_or(event.start.date.clone(), |d| d.to_rfc3339()),
                        data.summary,
                        data.attendees.len()
                    );
                }
            }
        }
        println!("\n------------------------------");
    }

    Ok(())
}

pub async fn load_credentials(client: &mut GoogClient) {
    let path = Path::new(SAVED_CREDS);

    // Load from file system (if exists) or run through token authorization process.
    let credentials = if path.exists() {
        let saved: Credentials = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Unable to deserialize token");

        saved
    } else {
        let (code, pkce_verifier) = get_token(client).await.expect("Unable to request token");
        let token = client
            .token_exchange(&code, &pkce_verifier)
            .await
            .expect("Unable to exchange code for token");

        let mut saved = Credentials::default();
        saved.refresh_token(&token);
        let _ = saved.save_to_file(path.to_path_buf());

        saved
    };

    let _ = client.set_credentials(&credentials);
}

pub async fn get_token(client: &GoogClient) -> Option<(String, String)> {
    let scopes = vec![AuthScope::Email, AuthScope::Calendar];

    let request = client.authorize(&scopes);
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
            code = value.into_owned();

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

        println!("Google returned the following code:\n{}\n", code);
        println!(
            "Google returned the following state:\n{} (expected `{}`)\n",
            state.secret(),
            request.csrf_token.secret()
        );

        Some((code, request.pkce_verifier))
    } else {
        None
    }
}
