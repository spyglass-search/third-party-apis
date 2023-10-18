use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_dimension: Option<String>,
    pub values: Vec<Vec<String>>,
}

impl ValueRange {
    pub fn with_values(values: Vec<Vec<String>>) -> Self {
        ValueRange {
            range: None,
            major_dimension: None,
            values,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spreadsheet {
    pub spreadsheet_id: String,
    pub sheets: Vec<Sheet>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    pub properties: SheetProperties,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetProperties {
    pub sheet_id: usize,
    pub title: String,
    pub index: usize,
    pub sheet_type: String,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRangeOptions {
    value_input_option: ValueInputOption,
    include_values_in_response: bool,
    response_value_render_option: ValueRenderOption,
    response_date_time_render_option: DateTimeRenderOption,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendValuesResponse {
    spreadsheet_id: String,
    table_range: String,
    updates: UpdateValuesResponse,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateValuesResponse {
    spreadsheet_id: String,
    updated_range: String,
    updated_rows: usize,
    updated_columns: usize,
    updated_cells: usize,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DateTimeRenderOption {
    #[default]
    SerialNumber,
    FormattedString,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValueInputOption {
    InputValueUnspecified,
    Raw,
    #[default]
    UserEntered,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]

pub enum ValueRenderOption {
    #[default]
    FormattedValue,
    UnformattedValue,
    Formula,
}
