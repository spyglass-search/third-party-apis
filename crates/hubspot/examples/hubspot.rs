use dotenv_codegen::dotenv;

use libauth::helpers::load_credentials;
use libhubspot::{types::AuthScope, HubspotClient};

const REDIRECT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scopes = AuthScope::default_scopes()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let client_id = dotenv!("HUBSPOT_CLIENT_ID");
    let secret = dotenv!("HUBSPOT_CLIENT_SECRET");

    let mut client = HubspotClient::new(client_id, secret, REDIRECT_URL, Default::default())?;

    load_credentials(&mut client, &scopes, false).await;

    let notes = client.list_notes(&[], None, None).await?;
    for (idx, note) in notes.results.iter().enumerate() {
        println!("{idx}: {note:?}");
    }

    Ok(())
}
