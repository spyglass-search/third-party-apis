use dotenv_codegen::dotenv;
use std::env;

use libauth::helpers::load_credentials;
use libgoog::{services::spreadsheets::Sheets, types::AuthScope, ClientType, GoogClient};

const REDIRECT_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let sheet_id: String = if args.len() <= 1 {
        println!("Past in the spreadsheet ID you want to run the example on.");
        return Ok(());
    } else {
        args.get(1).cloned().unwrap_or_default()
    };

    let scopes = vec![AuthScope::Sheets.to_string()];

    let google_client_id = dotenv!("GOOGLE_CLIENT_ID");
    let google_client_secret = dotenv!("GOOGLE_CLIENT_SECRET");

    let mut client = GoogClient::new(
        ClientType::Sheets,
        google_client_id,
        google_client_secret,
        REDIRECT_URL,
        Default::default(),
    )?;

    load_credentials(&mut client, &scopes).await;
    let mut spreadsheet = Sheets::new(client);

    let sheet_data = spreadsheet.get(&sheet_id).await?;
    println!("\n------------------------------");
    print!("Sheets");
    println!("\n------------------------------");
    for (idx, sheet) in sheet_data.sheets.iter().enumerate() {
        println!("Sheet {idx}: {}", sheet.properties.title);
    }

    println!("\n------------------------------");
    print!("Data");
    println!("\n------------------------------");
    let first_sheet = sheet_data.sheets.first().unwrap();
    let results = spreadsheet
        .read_range(&sheet_id, &first_sheet.properties.title, "A1:AA5")
        .await?;

    for (idx, x) in results.values.iter().enumerate() {
        print!("{idx}: ");
        for y in x.iter() {
            print!(" {y},");
        }
        println!("");
    }

    let updated_values: Vec<String> = vec![
        "test1".into(),
        "test2".into(),
        "test3".into(),
        "test4".into(),
        "test5".into(),
    ];

    spreadsheet
        .append(
            &sheet_id,
            &first_sheet.properties.title,
            &updated_values,
            &Default::default(),
        )
        .await?;

    Ok(())
}
