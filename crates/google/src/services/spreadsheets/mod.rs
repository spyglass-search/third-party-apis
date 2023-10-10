use libauth::{ApiClient, ApiError};
use reqwest::StatusCode;
use a1_notation::{A1, RangeOrCell, Address};

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

    pub async fn append(
        &mut self,
        spreadsheet_id: &str,
        sheet_id: &str,
        values: &[String],
        update_options: &types::UpdateRangeOptions,
    ) -> Result<types::AppendValuesResponse, ApiError> {
        // Determine the cell range based on the number of values
        let notation = A1 {
            sheet_name: Some(sheet_id.to_string()),
            reference: RangeOrCell::Range {
                from: Address::new(0, 0),
                to: Address::new(0, values.len())
            }
        };

        let notation = notation.to_string();
        let mut endpoint = self.client.endpoint.clone();
        endpoint.push_str(&format!(
            "/spreadsheets/{spreadsheet_id}/values/{notation}:append"
        ));

        let updates: Vec<Vec<String>> = vec![values.to_vec()];
        let body = ValueRange::with_values(updates);

        let client = self.client.get_check_client().await?;
        let resp = client
            .post(&endpoint)
            .query(update_options)
            .json(&body)
            .send()
            .await?;

        match resp.error_for_status() {
            Ok(resp) => match resp.json::<types::AppendValuesResponse>().await {
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
