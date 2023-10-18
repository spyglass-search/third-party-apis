use std::collections::HashMap;

use a1_notation::{Address, RangeOrCell, A1};
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
        serde_json::from_value::<types::Spreadsheet>(self.client.call_json(&endpoint, &[]).await?)
            .map_err(ApiError::SerdeError)
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
        // Parse and validate cell range
        let mut notation = a1_notation::new(cell_range)
            .map_err(|_| ApiError::BadRequest("Invalid cell range".to_string()))?;
        notation = notation.with_sheet_name(sheet_id);

        endpoint.push_str(&format!(
            "/spreadsheets/{spreadsheet_id}/values/{}",
            notation
        ));
        serde_json::from_value::<types::ValueRange>(self.client.call_json(&endpoint, &[]).await?)
            .map_err(ApiError::SerdeError)
    }

    pub async fn read_rows_as_map(
        &mut self,
        spreadsheet_id: &str,
        sheet_id: &str,
        start: usize,
        end: usize,
    ) -> Result<Vec<HashMap<String, String>>, ApiError> {
        // Make sure cell_range doesn't include the first row, that is always the header
        let start = if start <= 1 { 2 } else { start };

        let notation = a1_notation::new(&format!("{start}:{end}"))
            .map_err(|_| ApiError::BadRequest("Invalid cell range".to_string()))?;

        // Read header
        let headers = self
            .read_range(spreadsheet_id, sheet_id, "1:1")
            .await?
            .values
            .first()
            .cloned()
            .ok_or(ApiError::BadRequest("No headers found".to_string()))?;

        // Read rows
        let rows: ValueRange = self
            .read_range(spreadsheet_id, sheet_id, &notation.to_string())
            .await?;

        // Map rows to headers
        let mut results: Vec<HashMap<String, String>> = Vec::new();
        for (idx, row) in rows.values.iter().enumerate() {
            let mut row_data = HashMap::new();
            row_data.insert("_idx".to_string(), (start + idx).to_string());
            for (idx, col) in row.iter().enumerate() {
                let default = idx.to_string();
                let header = headers.get(idx).unwrap_or(&default);
                row_data.insert(header.to_owned(), col.to_owned());
            }
            results.push(row_data);
        }

        // Create mapping
        Ok(results)
    }

    pub async fn append(
        &mut self,
        spreadsheet_id: &str,
        sheet_id: &str,
        values: &Vec<Vec<String>>,
        update_options: &types::UpdateRangeOptions,
    ) -> Result<types::AppendValuesResponse, ApiError> {
        // Determine the cell range based on the number of values
        let notation = A1 {
            sheet_name: Some(sheet_id.to_string()),
            reference: RangeOrCell::Range {
                from: Address::new(0, 0),
                to: Address::new(0, values.len()),
            },
        };

        let notation = notation.to_string();
        let mut endpoint = self.client.endpoint.clone();
        endpoint.push_str(&format!(
            "/spreadsheets/{spreadsheet_id}/values/{notation}:append"
        ));

        let updates: Vec<Vec<String>> = values.clone();
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
