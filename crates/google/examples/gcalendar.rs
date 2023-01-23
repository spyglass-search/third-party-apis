use dotenv_codegen::dotenv;

use libauth::helpers::load_credentials;
use libgoog::{types::AuthScope, ClientType, GoogClient};

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scopes = vec![
        AuthScope::Email.to_string(),
        AuthScope::Calendar.to_string(),
    ];
    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let mut client = GoogClient::new(
        ClientType::Calendar,
        google_client_id,
        google_client_secret,
        REDIRECT_URL,
        Default::default(),
    )?;

    load_credentials(&mut client, &scopes).await;

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
