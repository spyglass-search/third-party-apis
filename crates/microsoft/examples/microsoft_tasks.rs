use chrono::{DateTime, Utc};
use dotenv_codegen::dotenv;
use libauth::helpers::load_credentials;
use libmicrosoft::{
    types::{AuthScopes, CreateTaskList, Task, TaskBody},
    MicrosoftClient,
};

const REDIRECT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_id = dotenv!("MICROSOFT_CLIENT_ID");
    let client_secret = dotenv!("MICROSOFT_CLIENT_SECRET");

    let mut client = MicrosoftClient::new(
        client_id,
        client_secret,
        REDIRECT_URL,
        "graph.ahh",
        Default::default(),
    )?;
    let scopes = vec![
        AuthScopes::UserRead.to_string(),
        AuthScopes::TasksReadWrite.to_string(),
        AuthScopes::OfflineAccess.to_string(),
        AuthScopes::MailRead.to_string(),
    ];
    load_credentials(&mut client, &scopes, false).await;

    let user = client.get_user().await?;
    println!(
        "Authenticated as user: {}",
        serde_json::to_string_pretty(&user).unwrap()
    );

    let task_lists = client.get_task_lists().await?;
    println!(
        "Task Lists: {}",
        serde_json::to_string_pretty(&task_lists).unwrap()
    );
    for list in task_lists.value {
        let tasks = client.get_tasks(list.id.as_str()).await?;
        println!("Tasks: {}", serde_json::to_string_pretty(&tasks).unwrap());

        let added = client
            .add_task(
                &list.id,
                Task {
                    body: TaskBody {
                        content: "My Test Content".to_string(),
                        content_type: "text".to_string(),
                    },
                    body_last_modified_date_time: None,
                    last_modified_date_time: None,
                    odata_etag: None,
                    categories: Vec::new(),
                    completed_date_time: None,
                    due_date_time: None,
                    reminder_date_time: None,
                    start_date_time: None,
                    created_date_time: None,
                    has_attachments: false,
                    is_reminder_on: false,
                    id: "Asdf".to_string(),
                    title: "My Fancy Title Test".to_string(),
                    importance: libmicrosoft::types::TaskImportance::Low,
                    recurrence: None,
                    status: libmicrosoft::types::TaskStatus::NotStarted,
                },
            )
            .await?;

        println!(
            "Added task {}",
            serde_json::to_string_pretty(&added).unwrap()
        );
    }

    let created_list = client
        .create_task_list(CreateTaskList {
            display_name: "My Super New Task List".to_string(),
        })
        .await?;

    println!(
        "Created List {}",
        serde_json::to_string_pretty(&created_list).unwrap()
    );

    let emails = client.get_new_emails(None).await?;

    println!(
        "Response Email {}",
        serde_json::to_string_pretty(&emails).unwrap()
    );

    let emails = client
        .get_new_emails(Some(
            DateTime::parse_from_rfc3339("2024-02-13T10:00:00-08:00")
                .unwrap()
                .with_timezone(&Utc),
        ))
        .await?;

    println!(
        "Response Email {}",
        serde_json::to_string_pretty(&emails).unwrap()
    );

    let emails = client
        .get_new_emails(Some(
            DateTime::parse_from_rfc3339("2024-02-13T16:00:00-08:00")
                .unwrap()
                .with_timezone(&Utc),
        ))
        .await?;

    // let emails = client.get_delta_email_page("https://graph.microsoft.com/v1.0/me/mailFolders('inbox')/messages/delta?$deltatoken=LztZwWjo5IivWBhyxw5rAOaF1aEpmIIXoTpgdnuLDugJJcwY-HKfZ3v_-5_2IYETBwWhHtMQ0h601TKBsp82L98T6l9U8bA4uixRm4jUqfY.g_m7pG94df2RVzL9ZiLwx2YlssVSx2U7V0pAvWrRq_8").await?;

    println!(
        "Response Email {}",
        serde_json::to_string_pretty(&emails).unwrap()
    );

    Ok(())
}
