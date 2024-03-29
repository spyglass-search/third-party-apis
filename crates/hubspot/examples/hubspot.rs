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

    println!("--- NOTES ---");
    let notes = client
        .list_objects::<libhubspot::types::Note>(libhubspot::CrmObject::Notes, &[], &[], None, None)
        .await?;
    for (idx, note) in notes.results.iter().enumerate() {
        println!("{idx}: {}", note.raw_body());
    }

    println!("\n--- CALLS ---");
    let calls = client
        .list_objects::<libhubspot::types::Call>(
            libhubspot::CrmObject::Calls,
            &["hs_call_recording_url".into()],
            &[],
            None,
            None,
        )
        .await?;
    for (idx, call) in calls.results.iter().enumerate() {
        println!(
            "{idx}: {}\nbody:{}\nurl: {:?}",
            call.title(),
            call.raw_body(),
            call.recording_url()
        );
        println!("---")
    }

    println!("--- Tasks ---");
    let tasks = client
        .list_objects::<libhubspot::types::Task>(libhubspot::CrmObject::Tasks, &[], &[], None, None)
        .await?;
    for (idx, task) in tasks.results.iter().enumerate() {
        println!(
            "{idx}: {:?} {:?} {:?} {:?} {:?}",
            task.subject(),
            task.raw_body(),
            task.task_type(),
            task.status(),
            task.priority()
        );
    }

    println!("--- Emails ---");
    let emails = client
        .list_objects::<libhubspot::types::Email>(
            libhubspot::CrmObject::Emails,
            &[],
            &[],
            None,
            None,
        )
        .await?;
    for (idx, email) in emails.results.iter().enumerate() {
        println!(
            "{idx}: {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            email.email_direction(),
            email.html_body(),
            email.raw_body(),
            email.owner_id(),
            email.received(),
            email.status(),
            email.subject()
        );
    }

    Ok(())
}
