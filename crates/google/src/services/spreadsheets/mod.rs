use libauth::{ApiClient, ApiError};
use reqwest::StatusCode;

pub mod types;

use crate::GoogClient;

use self::types::ValueRange;

pub struct Sheets {
    client: GoogClient,
}

impl Sheets {
    pub fn new(client: GoogClient) -> Self {
        Sheets { client }
    }

    pub async fn get(&mut self, spreadsheet_id: &str) -> Result<types::Spreadsheet, ApiError> {
        let mut endpoint = self.client.endpoint.clone();
        endpoint.push_str(&format!("/spreadsheets/{spreadsheet_id}"));
        self.client.call_json(&endpoint, &[]).await
    }

    /// Grab cell values using A1 notation (see: https://developers.google.com/sheets/api/guides/concepts#cell)
    /// sheet_id and cell_range are combined together to create the notation.
    pub async fn read_range(
        &mut self,
        spreadsheet_id: &str,
        sheet_id: &str,
        cell_range: &str,
    ) -> Result<types::ValueRange, ApiError> {
        let mut endpoint = self.client.endpoint.clone();
        endpoint.push_str(&format!(
            "/spreadsheets/{spreadsheet_id}/values/{sheet_id}!{cell_range}"
        ));
        self.client.call_json(&endpoint, &[]).await
    }

    pub async fn update_range(
        &mut self,
        spreadsheet_id: &str,
        sheet_id: &str,
        cell_range: &str,
        values: &[String],
        update_options: &types::UpdateRangeOptions,
    ) -> Result<types::UpdateValuesResponse, ApiError> {
        let mut endpoint = self.client.endpoint.clone();
        endpoint.push_str(&format!(
            "/spreadsheets/{spreadsheet_id}/values/{sheet_id}!{cell_range}"
        ));

        let updates: Vec<Vec<String>> = vec![values.to_vec()];
        let body = ValueRange::with_values(updates);

        let client = self.client.get_check_client().await?;
        let resp = client
            .put(&endpoint)
            .query(update_options)
            .json(&body)
            .send()
            .await?;

        match resp.error_for_status() {
            Ok(resp) => match resp.json::<types::UpdateValuesResponse>().await {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            },
            // Any status code from 400..599
            Err(err) => {
                if let Some(StatusCode::UNAUTHORIZED) = err.status() {
                    Err(ApiError::AuthError("Unauthorized".to_owned()))
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
