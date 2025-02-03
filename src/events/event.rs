use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NotificationMethod {
    Email,
    SMS,
    Push,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub notify_before: i64, // Time in minutes before the event to send the notification
    pub method: NotificationMethod, // Method of notification (e.g., email, SMS, push)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recurrence {
    pub frequency: RecurrenceFrequency, // Frequency of recurrence (e.g., daily, weekly, monthly)
    pub interval: i32,                  // Interval between occurrences (e.g., every 2 weeks)
    pub days_of_week: Vec<String>, // Days of the week for weekly events (e.g., ["Monday", "Wednesday"])
    pub start_date: NaiveDateTime, // Start date for the recurrence
    pub end_date: Option<NaiveDateTime>, // End date for the recurrence (optional)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub attendee_id: String, // Unique identifier for the attendee
    pub name: String,        // Name of the attendee
    pub email: String,       // Email of the attendee
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_id: String,                         // Unique identifier for the event
    pub title: String,                            // Title of the event
    pub description: String,                      // Description of the event
    pub location: String,                         // Location of the event
    pub start_time: NaiveDateTime,                // Start time of the event
    pub end_time: NaiveDateTime,                  // End time of the event
    pub is_recurring: bool,                       // Flag to indicate if the event is recurring
    pub recurrence: Option<Recurrence>,           // Recurrence details (if applicable)
    pub attendees: Vec<Attendee>,                 // List of attendees
    pub created_at: NaiveDateTime,                // Timestamp when the event was created
    pub updated_at: NaiveDateTime,                // Timestamp when the event was last updated
    pub notification_settings: Vec<Notification>, // Notification settings
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!("{:?}", self))
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            event_id: Default::default(),
            title: Default::default(),
            description: Default::default(),
            location: Default::default(),
            start_time: Default::default(),
            end_time: Default::default(),
            is_recurring: Default::default(),
            recurrence: Default::default(),
            attendees: Default::default(),
            created_at: Default::default(),
            updated_at: Default::default(),
            notification_settings: Default::default(),
        }
    }
}
