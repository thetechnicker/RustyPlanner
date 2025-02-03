use chrono::{DateTime, Local};
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
    pub start_date: DateTime<Local>, // Start date for the recurrence
    pub end_date: Option<DateTime<Local>>, // End date for the recurrence (optional)
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
    pub start_time: DateTime<Local>,              // Start time of the event
    pub end_time: DateTime<Local>,                // End time of the event
    pub is_recurring: bool,                       // Flag to indicate if the event is recurring
    pub recurrence: Option<Recurrence>,           // Recurrence details (if applicable)
    pub attendees: Vec<Attendee>,                 // List of attendees
    pub created_at: DateTime<Local>,              // Timestamp when the event was created
    pub updated_at: DateTime<Local>,              // Timestamp when the event was last updated
    pub notification_settings: Vec<Notification>, // Notification settings
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&serde_json::to_string_pretty(&self).unwrap_or("Error".to_string()))
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

#[allow(dead_code)]
impl Event {
    pub fn set_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn set_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn set_location(mut self, location: String) -> Self {
        self.location = location;
        self
    }

    pub fn set_start_time(mut self, start_time: DateTime<Local>) -> Self {
        self.start_time = start_time;
        self
    }

    pub fn set_end_time(mut self, end_time: DateTime<Local>) -> Self {
        self.end_time = end_time;
        self
    }

    pub fn set_is_recurring(mut self, is_recurring: bool) -> Self {
        self.is_recurring = is_recurring;
        self
    }

    pub fn set_recurrence(mut self, recurrence: Option<Recurrence>) -> Self {
        self.recurrence = recurrence;
        self
    }

    pub fn set_attendees(mut self, attendees: Vec<Attendee>) -> Self {
        self.attendees = attendees;
        self
    }

    pub fn set_notification_settings(mut self, notification_settings: Vec<Notification>) -> Self {
        self.notification_settings = notification_settings;
        self
    }
}

#[allow(dead_code)]
impl Event {
    // Update the title of the event
    pub fn update_title(&mut self, new_title: String) {
        self.title = new_title;
        self.updated_at = chrono::Local::now();
    }

    // Update the description of the event
    pub fn update_description(&mut self, new_description: String) {
        self.description = new_description;
        self.updated_at = chrono::Local::now();
    }

    // Update the location of the event
    pub fn update_location(&mut self, new_location: String) {
        self.location = new_location;
        self.updated_at = chrono::Local::now();
    }

    // Update the start time of the event
    pub fn update_start_time(&mut self, new_start_time: DateTime<Local>) {
        self.start_time = new_start_time;
        self.updated_at = chrono::Local::now();
    }

    // Update the end time of the event
    pub fn update_end_time(&mut self, new_end_time: DateTime<Local>) {
        self.end_time = new_end_time;
        self.updated_at = chrono::Local::now();
    }

    // Update the recurring status of the event
    pub fn update_is_recurring(&mut self, is_recurring: bool) {
        self.is_recurring = is_recurring;
        self.updated_at = chrono::Local::now();
    }

    // Update the recurrence details of the event
    pub fn update_recurrence(&mut self, new_recurrence: Option<Recurrence>) {
        self.recurrence = new_recurrence;
        self.updated_at = chrono::Local::now();
    }

    // Add an attendee to the event
    pub fn add_attendee(&mut self, attendee: Attendee) {
        self.attendees.push(attendee);
        self.updated_at = chrono::Local::now();
    }

    // Remove an attendee from the event by index
    pub fn remove_attendee(&mut self, index: usize) -> Option<Attendee> {
        if index < self.attendees.len() {
            self.updated_at = chrono::Local::now();
            Some(self.attendees.remove(index))
        } else {
            None // Return None if the index is out of bounds
        }
    }

    // Add a notification setting to the event
    pub fn add_notification(&mut self, notification: Notification) {
        self.notification_settings.push(notification);
        self.updated_at = chrono::Local::now();
    }

    // Remove a notification setting from the event by index
    pub fn remove_notification(&mut self, index: usize) -> Option<Notification> {
        if index < self.notification_settings.len() {
            self.updated_at = chrono::Local::now();
            Some(self.notification_settings.remove(index))
        } else {
            None // Return None if the index is out of bounds
        }
    }
}
