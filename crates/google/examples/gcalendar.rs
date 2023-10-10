use dotenv_codegen::dotenv;
use libauth::helpers::load_credentials;
use libgoog::{types::AuthScope, ClientType, GoogClient};
use libgoog::services::calendar::Calendar;

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
    let mut calendar = Calendar::new(client);
    println!("AUTHORIZED USER: {user:?}");

    let cals = calendar.list_calendars(None).await?;
    println!("------------------------------");
    println!("next_page: {:?}", cals.next_page_token);

    println!("\n------------------------------");
    println!("CALENDARS");
    println!("\n------------------------------");
    let calendars = calendar.list_calendars(None).await?;
    for cal in calendars.items.iter() {
        println!(
            "CALENDAR: {} ({}) | {} | {}",
            cal.id, cal.primary, cal.summary, cal.access_role
        )
    }

    let last_month = chrono::Utc::now() - chrono::Duration::days(30);
    let future_month = chrono::Utc::now() + chrono::Duration::days(30);

    println!("\n------------------------------");
    println!("PRIMARY CALENDAR");
    let primary_events = calendar
        .list_calendar_events("primary", Some(last_month), Some(future_month), None)
        .await?;
    for event in primary_events.items.iter().take(10) {
        // Skip recurring dates that don't have a next recurrence within our time
        // period.
        let date = if event.is_recurring() {
            event.next_recurrence().map(|x| x.to_rfc3339())
        } else {
            Some(
                event
                    .start
                    .date_time
                    .map_or(event.start.date.clone(), |d| d.to_rfc3339()),
            )
        };

        if let Some(date) = date {
            println!(
                "EVENT - {}\nDate: {} - {:?}\nTitle: {}\n# of Attendees: {}\nRecurring: {}\n",
                event.id,
                date,
                event.end.date_time,
                event.summary,
                event.attendees.len(),
                event.is_recurring()
            );
        }
    }
    println!("------------------------------");

    for cal in cals.items.iter().take(5) {
        println!("\nCALENDAR: {} ({})", cal.summary, cal.id);
        if let Ok(events) = calendar
            .list_calendar_events(&cal.id, Some(last_month), Some(future_month), None)
            .await
        {
            for event in events.items.iter().take(5) {
                if let Ok(data) = calendar.get_calendar_event(&cal.id, &event.id).await {
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
