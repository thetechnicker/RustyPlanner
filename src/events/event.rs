use chrono::Datelike;
use chrono::{DateTime, Duration, Local, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::miscs::{
    arg_parsing::Data,
    utils::{date_from_str, parse_duration, time_from_str},
};

pub static CATEGORIES: Mutex<Vec<String>> = Mutex::new(vec![]);

pub fn load_categories(path: &PathBuf) {
    let mut categories = CATEGORIES.lock().unwrap();
    if !std::path::Path::new(path).exists() {
        categories.append(&mut vec![
            "Work".to_string(),
            "Personal".to_string(),
            "Family".to_string(),
            "Health".to_string(),
            "Education".to_string(),
            "Entertainment".to_string(),
            "Other".to_string(),
        ]);
    } else {
        let categories_str = std::fs::read_to_string(path).unwrap();
        for category in categories_str.lines() {
            categories.push(category.trim().to_string());
        }
    }
}

pub fn save_categories(path: &PathBuf) {
    let categories = CATEGORIES.lock().unwrap();
    let categories_str = categories.join("\n");
    std::fs::write(path, categories_str).unwrap();
}

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
    Sms,
    Push,
}

impl std::fmt::Display for NotificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            NotificationMethod::Email => "Email",
            NotificationMethod::Sms => "SMS",
            NotificationMethod::Push => "Push",
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub notify_before: i64, // Time in minutes before the event to send the notification
    pub method: NotificationMethod, // Method of notification (e.g., email, SMS, push)
    pub has_notified: bool,
}
impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(
            format!(
                "Notify Before: {}, Method: {}",
                self.notify_before, self.method
            )
            .as_str(),
        )
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            notify_before: Default::default(),
            method: NotificationMethod::Push,
            has_notified: false,
        }
    }
}

impl Notification {
    pub fn from_data(data: &Data) -> Result<Self, String> {
        match data {
            Data::Object(data_object) => {
                let mut notification = Self::default();
                if let Some(Data::String(duration_str)) = data_object.get("remind-before") {
                    notification.notify_before = duration_str.parse::<i64>().unwrap_or(10);
                }
                if let Some(Data::String(method_str)) = data_object.get("method") {
                    notification.method = match method_str.to_lowercase().as_str() {
                        "email" => NotificationMethod::Email,
                        "sms" => NotificationMethod::Sms,
                        "push" => NotificationMethod::Push,
                        _ => NotificationMethod::Push,
                    };
                }
                Ok(notification)
            }
            _ => Err("Data must be an Object".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RecurrenceFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceFrequency {
    pub fn from_str(string: &str) -> Self {
        match string.to_lowercase().as_str() {
            "hourly" => Self::Hourly,
            "daily" => Self::Daily,
            "weekly" => Self::Weekly,
            "monthly" => Self::Monthly,
            "yearly" => Self::Yearly,
            _ => Self::Daily,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recurrence {
    pub frequency: RecurrenceFrequency, // Frequency of recurrence (e.g., daily, weekly, monthly)
    pub interval: i64,                  // Interval between occurrences (e.g., every 2 weeks)
    pub start_date: DateTime<Local>,    // Start date for the recurrence
    pub end_date: Option<DateTime<Local>>, // End date for the recurrence (optional)

    // Fields for specific timing
    pub minute: Option<u32>,
    pub hour: Option<u32>,
    pub day: Option<u32>,
    pub week_day: Option<Weekday>,
    pub month: Option<u32>,
    pub year: Option<u32>,
}

#[allow(dead_code)]
impl Recurrence {
    pub fn from_data(data: &Data) -> Result<Self, String> {
        //data.print(0);
        match data {
            Data::Object(_data) => {
                let mut recurrence = Self {
                    frequency: RecurrenceFrequency::Daily,
                    interval: 1,
                    start_date: Local::now(),
                    end_date: None,

                    minute: None,
                    hour: None,
                    day: None,
                    week_day: None,
                    month: None,
                    year: None,
                };
                if let Some(Data::String(frequency)) = _data.get("frequency") {
                    recurrence.frequency = RecurrenceFrequency::from_str(frequency);
                }

                if let Some(Data::Int(intervall)) = _data.get("intervall") {
                    recurrence.interval = *intervall;
                }

                if let Some(Data::String(start_time)) = _data.get("start-date") {
                    let start_time_naive =
                        date_from_str(start_time).and_time(time_from_str("00:00"));
                    recurrence.start_date = DateTime::from_naive_utc_and_offset(
                        start_time_naive,
                        *Local::now().offset(),
                    );
                }

                if let Some(Data::String(end_time)) = _data.get("end-") {
                    let end_time_naive = date_from_str(end_time).and_time(time_from_str("24:00"));
                    recurrence.end_date = Some(DateTime::from_naive_utc_and_offset(
                        end_time_naive,
                        *Local::now().offset(),
                    ));
                }

                if let Some(Data::Int(minute)) = _data.get("minute") {
                    recurrence.minute = Some(*minute as u32);
                }
                if let Some(Data::Int(hour)) = _data.get("hour") {
                    recurrence.hour = Some(*hour as u32);
                }
                if let Some(Data::Int(day)) = _data.get("day") {
                    recurrence.day = Some(*day as u32);
                }
                if let Some(Data::String(week_day)) = _data.get("week-day") {
                    recurrence.week_day = parse_weekday(week_day);
                }
                if let Some(Data::Int(month)) = _data.get("month") {
                    recurrence.month = Some(*month as u32);
                }
                if let Some(Data::Int(year)) = _data.get("year") {
                    recurrence.year = Some(*year as u32);
                }

                Ok(recurrence)
            }
            _ => Err("Data must be Type Object".to_string()),
        }
    }

    pub fn is_now(&self, now: DateTime<Local>) -> bool {
        let is_minute = self
            .minute
            .map_or(self.frequency != RecurrenceFrequency::Hourly, |minute| {
                minute == now.minute()
            });
        let is_hour = self
            .hour
            .map_or(self.frequency != RecurrenceFrequency::Daily, |hour| {
                hour == now.hour()
            });
        let is_day = self
            .day
            .map_or(self.frequency != RecurrenceFrequency::Monthly, |day| {
                day == now.day()
            });
        let is_week_day = self
            .week_day
            .map_or(self.frequency != RecurrenceFrequency::Weekly, |week_day| {
                week_day == now.weekday()
            });
        let is_month = self
            .month
            .map_or(self.frequency != RecurrenceFrequency::Yearly, |month| {
                month == now.month()
            });

        match self.frequency {
            RecurrenceFrequency::Hourly => {
                is_minute
                    && self.start_date <= now
                    && self.end_date.unwrap_or(now) >= now
                    && (now - self.start_date).num_hours() % self.interval == 0
            }
            RecurrenceFrequency::Daily => {
                !(!is_minute
                    || !is_hour
                    || self.start_date > now
                    || self.end_date.unwrap_or(now) < now
                    || (now - self.start_date).num_days() % self.interval != 0)
            }
            RecurrenceFrequency::Weekly => {
                is_minute
                    && is_hour
                    && is_week_day
                    && self.start_date <= now
                    && self.end_date.unwrap_or(now) >= now
                    && (now - self.start_date).num_days() % (self.interval * 7) == 0
            }
            RecurrenceFrequency::Monthly => {
                is_minute
                    && is_hour
                    && is_day
                    && self.start_date > now
                    && self.end_date.unwrap_or(now) >= now
                    && (now - self.start_date).num_days() % self.interval == 0
            }
            RecurrenceFrequency::Yearly => {
                !(!is_minute
                    || !is_hour
                    || !is_day
                    || !is_month
                    || self.start_date > now
                    || self.end_date.unwrap_or(now) < now
                    || (now - self.start_date).num_days() % self.interval != 0)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub attendee_id: String, // Unique identifier for the attendee
    pub name: String,        // Name of the attendee
    pub email: String,       // Email of the attendee
}

impl std::fmt::Display for Attendee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(format!("Name: {}, Email: {}", self.name, self.email).as_str())
    }
}

impl Attendee {
    pub fn from_data(data: &Data) -> Result<Self, String> {
        match data {
            Data::Object(attendee_data) => {
                if let Some(Data::String(name)) = attendee_data.get("name") {
                    if let Some(Data::String(email)) = attendee_data.get("email") {
                        Ok(Self {
                            attendee_id: "None".to_string(),
                            name: name.to_string(),
                            email: email.to_string(),
                        })
                    } else {
                        Err("Email must be given".to_string())
                    }
                } else {
                    Err("Name must be given".to_string())
                }
            }
            _ => Err("Data must be object".to_string()),
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
    pub is_all_day: bool,                         // Some comment for astetic reasons
    pub categories: Vec<String>,                  // Categories for the event
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
            created_at: Local::now(),
            updated_at: Local::now(),
            notification_settings: Default::default(),
            is_all_day: false,
            categories: Vec::new(),
        }
    }
}

impl Event {
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

    pub fn from_data(data: Data) -> Result<Self, String> {
        match data {
            Data::Object(fields) => {
                let mut event = Event::default();

                // Extract fields from the HashMap
                if let Some(Data::String(event_id)) = fields.get("event_id") {
                    event.event_id = event_id.clone();
                }
                if let Some(Data::String(title)) = fields.get("title") {
                    event.title = title.clone();
                }
                if let Some(Data::String(description)) = fields.get("description") {
                    event.description = description.clone();
                }

                let date = if let Some(Data::String(date)) = fields.get("date") {
                    date_from_str(date)
                } else {
                    Local::now().naive_local().date()
                };
                let time = if let Some(Data::String(time)) = fields.get("time") {
                    time_from_str(time)
                } else if let Some(Data::String(_)) = fields.get("whole_day") {
                    event.is_all_day = true;
                    time_from_str("0:00")
                } else {
                    Local::now().naive_utc().time()
                };
                let naive_datetime = date.and_time(time);

                event.start_time =
                    DateTime::from_naive_utc_and_offset(naive_datetime, *Local::now().offset());

                let duration = if !event.is_all_day {
                    Duration::days(1)
                } else if let Some(Data::String(duration)) = fields.get("duration") {
                    match parse_duration(duration) {
                        Ok(d) => d,
                        Err(_) => {
                            println!("duration invalid");
                            Duration::hours(2)
                        }
                    }
                } else {
                    Duration::hours(2)
                };

                event.end_time = event.start_time + duration;

                if let Some(Data::String(start_time_str)) = fields.get("start_time") {
                    if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                        event.start_time = start_time.with_timezone(&Local);
                    } else {
                        return Err("Invalid start_time format".to_string());
                    }
                }
                if let Some(Data::String(end_time_str)) = fields.get("end_time") {
                    if let Ok(end_time) = DateTime::parse_from_rfc3339(end_time_str) {
                        event.end_time = end_time.with_timezone(&Local);
                    } else {
                        return Err("Invalid end_time format".to_string());
                    }
                }
                if let Some(Data::String(location)) = fields.get("location") {
                    event.location = location.clone();
                }
                if let Some(Data::Int(is_recurring)) = fields.get("is_recurring") {
                    event.is_recurring = *is_recurring != 0; // Assuming 0 is false, 1 is true
                }

                if let Some(data) = fields.get("recurrence") {
                    event.is_recurring = true;
                    match Recurrence::from_data(data) {
                        Ok(recurrence) => event.recurrence = Some(recurrence),
                        Err(_) => eprintln!("error parsing recurrence"),
                    }
                }

                if let Some(Data::List(attendees)) = fields.get("attendees") {
                    for anttendee_data in attendees {
                        if let Ok(attendee) = Attendee::from_data(anttendee_data) {
                            event.attendees.push(attendee);
                        }
                    }
                }

                if let Some(Data::List(notifications)) = fields.get("notification-settings") {
                    for notification_data in notifications {
                        if let Ok(notification) = Notification::from_data(notification_data) {
                            event.notification_settings.push(notification);
                        }
                    }
                }

                if let Some(Data::String(is_all_day)) = fields.get("is-all-day") {
                    event.is_all_day = is_all_day == "true";
                }

                if let Some(Data::List(categories)) = fields.get("categories") {
                    for category in categories {
                        if let Data::String(category) = category {
                            event.categories.push(category.clone());
                        }
                    }
                }

                if event.notification_settings.is_empty() {
                    event.notification_settings.push(Notification::default());
                }
                Ok(event)
            }
            _ => Err("Expected an Object variant".to_string()),
        }
    }

    pub fn is_time_to_notify(&self, now: DateTime<Local>) -> Vec<(usize, bool)> {
        let mut notifications = vec![];
        for (i, notification) in self.notification_settings.iter().enumerate() {
            if self.is_recurring {
                if self
                    .recurrence
                    .as_ref()
                    .unwrap()
                    .is_now(now + Duration::minutes(notification.notify_before))
                {
                    notifications.push((i, true));
                } else {
                    notifications.push((i, false));
                }
            } else if self.start_time - Duration::minutes(notification.notify_before) <= now
                && !notification.has_notified
            {
                notifications.push((i, true));
            } else {
                notifications.push((i, false));
            }
        }
        notifications
    }
}

// list of keywords for creating an event from data, with description as [[&str; 2]; num_of_keywords]
pub const EVENT_FIELDS: [[&str; 2]; 16] = [
    ["event_id", "ID of the event, currently autogenerated"],
    ["title", "Name of the event"],
    ["description", "More detailed Description of the event"],
    ["date", "Date of the event"],
    ["time", "Time of the event"],
    [
        "whole_day",
        "Flag to indicate if the event is an all-day event",
    ],
    ["duration", "Duration of the event"],
    ["start_time", "Start time of the event"],
    ["end_time", "End time of the event"],
    ["location", "Location of the event"],
    ["is_recurring", "Flag to indicate if the event is recurring"],
    ["recurrence", "Recurrence details for the event"],
    ["attendees", "List of attendees for the event"],
    [
        "notification_settings",
        "Notification settings for the event",
    ],
    [
        "is_all_day",
        "Flag to indicate if the event is an all-day event",
    ],
    ["categories", "Categories for the event"],
];

pub const RECURRENCE_FIELDS: [[&str; 2]; 10] = [
    [
        "frequency",
        "Frequency of recurrence (e.g., daily, weekly, monthly)",
    ],
    [
        "intervall",
        "Interval between occurrences (e.g., every 2 weeks)",
    ],
    ["start-time", "Start date for the recurrence"],
    ["end-time", "End date for the recurrence (optional)"],
    ["minute", "Minute of the hour for the event"],
    ["hour", "Hour of the day for the event"],
    ["day", "Day of the month for the event"],
    ["week-day", "Day of the week for the event"],
    ["month", "Month of the year for the event"],
    ["year", "Year for the event"],
];

pub const ATTENDEE_FIELDS: [[&str; 2]; 2] = [
    ["name", "Name of the attendee"],
    ["email", "Email of the attendee"],
];

pub const NOTIFICATION_FIELDS: [[&str; 2]; 2] = [
    [
        "remind-before",
        "Time in minutes before the event to send the notification",
    ],
    ["method", "Method of notification (e.g., email, SMS, push)"],
];
