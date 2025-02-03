use std::collections::HashMap;

use chrono::{DateTime, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};

fn parse_weekday(value: &str) -> Option<Weekday> {
    match value.to_lowercase().as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None, // Return None for invalid input
    }
}

fn parse_weekday_default(value: &str) -> Weekday {
    match parse_weekday(value) {
        Some(weekday) => weekday,
        _ => Weekday::Mon,
    }
}

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

#[allow(dead_code)]
impl Notification {
    pub fn from_args(args: &[String]) -> Self {
        let notify_before = args[0].parse().expect("Invalid notify_before");
        let method = match args[1].as_str() {
            "Email" => NotificationMethod::Email,
            "SMS" => NotificationMethod::SMS,
            "Push" => NotificationMethod::Push,
            _ => panic!("Invalid notification method"),
        };
        Notification {
            notify_before,
            method,
        }
    }
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
    pub days_of_week: Vec<Weekday>, // Days of the week for weekly events (e.g., ["Monday", "Wednesday"])
    pub start_date: DateTime<Local>, // Start date for the recurrence
    pub end_date: Option<DateTime<Local>>, // End date for the recurrence (optional)
}

#[allow(dead_code)]
impl Recurrence {
    pub fn from_args(args: &[String]) -> Self {
        let frequency = match args[0].as_str() {
            "Daily" => RecurrenceFrequency::Daily,
            "Weekly" => RecurrenceFrequency::Weekly,
            "Monthly" => RecurrenceFrequency::Monthly,
            "Yearly" => RecurrenceFrequency::Yearly,
            _ => panic!("Invalid recurrence frequency"),
        };
        let interval = args[1].parse().expect("Invalid interval");
        let days_of_week = args[2].split(',').map(parse_weekday_default).collect();
        let start_date = DateTime::parse_from_rfc3339(&args[3])
            .expect("Invalid start date")
            .with_timezone(&Local);
        let end_date = if args.len() > 4 {
            Some(
                DateTime::parse_from_rfc3339(&args[4])
                    .expect("Invalid end date")
                    .with_timezone(&Local),
            )
        } else {
            None
        };
        Recurrence {
            frequency,
            interval,
            days_of_week,
            start_date,
            end_date,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub attendee_id: String, // Unique identifier for the attendee
    pub name: String,        // Name of the attendee
    pub email: String,       // Email of the attendee
}

#[allow(dead_code)]
impl Attendee {
    pub fn from_args(args: &[String]) -> Self {
        let attendee_id = args[0].clone();
        let name = args[1].clone();
        let email = args[2].clone();
        Attendee {
            attendee_id,
            name,
            email,
        }
    }
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
        let start_time = Local::now();
        Self {
            event_id: Default::default(),
            title: Default::default(),
            description: Default::default(),
            location: Default::default(),
            start_time,
            end_time: start_time + Duration::hours(1),
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
    pub fn from_args(args: &[String]) -> Self {
        let len = args.len();

        // Initialize a new Event instance with default values
        let mut event = Event::default();

        // Set the event ID, title, description, and start time
        if let Some(id) = args.get(0) {
            event = event.set_event_id(id.clone());
        }
        if let Some(title) = args.get(1) {
            event = event.set_title(title.clone());
        }
        if let Some(description) = args.get(2) {
            event = event.set_description(description.clone());
        }
        if let Some(start_time_str) = args.get(3) {
            if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                event = event.set_start_time(start_time.with_timezone(&Local));
            }
        }

        // Determine if the event is recurring
        if len > 4 {
            let is_recurring = args[4].parse().expect("Invalid is_recurring");
            event = event.set_is_recurring(is_recurring);
        }

        event
    }

    pub fn parse_kwargs(kwargs: HashMap<String, String>) -> Self {
        let mut event = Event::default();

        // Extract values from the HashMap
        if let Some(event_id) = kwargs.get("event_id") {
            event.event_id = event_id.clone();
        }
        if let Some(title) = kwargs.get("title") {
            event.title = title.clone();
        }
        if let Some(description) = kwargs.get("description") {
            event.description = description.clone();
        }
        if let Some(start_time_str) = kwargs.get("start_time") {
            if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                event.start_time = start_time.with_timezone(&Local);
            }
        }
        if let Some(end_time_str) = kwargs.get("end_time") {
            if let Ok(end_time) = DateTime::parse_from_rfc3339(end_time_str) {
                event.end_time = end_time.with_timezone(&Local);
            }
        }
        if let Some(location) = kwargs.get("location") {
            event.location = location.clone();
        }
        if let Some(is_recurring_str) = kwargs.get("is_recurring") {
            event.is_recurring = is_recurring_str.parse().unwrap_or(false);
        }

        event
    }

    pub fn update_from_kwargs(mut self, kwargs: HashMap<String, String>) -> Self {
        // Update values from the HashMap
        if let Some(event_id) = kwargs.get("event_id") {
            self.event_id = event_id.clone();
        }
        if let Some(title) = kwargs.get("title") {
            self.title = title.clone();
        }
        if let Some(description) = kwargs.get("description") {
            self.description = description.clone();
        }
        if let Some(start_time_str) = kwargs.get("start_time") {
            if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                self.start_time = start_time.with_timezone(&Local);
            }
        }
        if let Some(end_time_str) = kwargs.get("end_time") {
            if let Ok(end_time) = DateTime::parse_from_rfc3339(end_time_str) {
                self.end_time = end_time.with_timezone(&Local);
            }
        }
        if let Some(location) = kwargs.get("location") {
            self.location = location.clone();
        }
        if let Some(is_recurring_str) = kwargs.get("is_recurring") {
            self.is_recurring = is_recurring_str.parse().unwrap_or(self.is_recurring);
        }

        self
    }

    fn set_event_id(mut self, event_id: String) -> Event {
        self.event_id = event_id;
        self
    }
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
