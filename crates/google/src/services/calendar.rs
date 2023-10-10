use crate::types;
use crate::GoogClient;
use chrono::{DateTime, Utc};
use libauth::{ApiClient, ApiError};

pub struct Calendar {
    client: GoogClient,
}

/// Retrieve list of calendars for the authenticated user.
impl Calendar {
    pub fn new(client: GoogClient) -> Self {
        Calendar { client }
    }

    pub async fn list_calendars(
        &mut self,
        next_page: Option<String>,
    ) -> Result<types::CalendarListResponse, ApiError> {
        let mut endpoint = self.client.endpoint.to_string();
        endpoint.push_str("/users/me/calendarList");

        let params = if let Some(next_page) = next_page {
            vec![("pageToken".to_string(), next_page)]
        } else {
            Vec::new()
        };

        self.client.call_json(&endpoint, &params).await
    }

    /// Retrieve all events for a calendar.
    /// Use the id "primary" for the user's primary calendar.
    pub async fn list_calendar_events(
        &mut self,
        calendar_id: &str,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
        next_page: Option<String>,
    ) -> Result<types::ListCalendarEventsResponse, ApiError> {
        let mut endpoint = self.client.endpoint.to_string();
        endpoint.push_str(&format!("/calendars/{calendar_id}/events"));

        let mut params = if let Some(next_page) = next_page {
            vec![("pageToken".to_string(), next_page)]
        } else {
            Vec::new()
        };

        params.push(("orderBy".to_string(), "updated".to_string()));
        if let Some(after) = after {
            params.push(("timeMin".into(), after.to_rfc3339()));
        }

        if let Some(before) = before {
            params.push(("timeMax".into(), before.to_rfc3339()));
        }

        self.client.call_json(&endpoint, &params).await
    }

    /// Retrieve a single event from a calendar.
    /// Use the id "primary" for the user's primary calendar.
    pub async fn get_calendar_event(
        &mut self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<types::CalendarEvent, ApiError> {
        let mut endpoint = self.client.endpoint.to_string();
        endpoint.push_str(&format!("/calendars/{calendar_id}/events/{event_id}"));
        self.client.call_json(&endpoint, &Vec::new()).await
    }
}
