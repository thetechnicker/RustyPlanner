use chrono::{DateTime, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};

use crate::miscs::{
    arg_parsing::Data,
    utils::{date_from_str, parse_duration, time_from_str},
};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub notify_before: i64, // Time in minutes before the event to send the notification
    pub method: NotificationMethod, // Method of notification (e.g., email, SMS, push)
    pub has_notified: bool,
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
    pub fn from_data(data: Data) -> Result<Self, String> {
        match data {
            Data::Object(data_object) => {
                let mut notification = Self::default();
                if let Some(Data::String(duration_str)) = data_object.get("k") {
                    notification.notify_before = duration_str.parse::<i64>().unwrap_or(10);
                }
                Ok(notification)
            }
            _ => Err("Data must be an Object".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceFrequency {
    pub fn from_str(string: &str) -> Self {
        match string.to_lowercase().as_str() {
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
    pub interval: i32,                  // Interval between occurrences (e.g., every 2 weeks)
    pub days_of_week: Vec<Weekday>, // Days of the week for weekly events (e.g., ["Monday", "Wednesday"])
    pub start_date: DateTime<Local>, // Start date for the recurrence
    pub end_date: Option<DateTime<Local>>, // End date for the recurrence (optional)
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
                    days_of_week: Vec::new(),
                    start_date: Local::now(),
                    end_date: None,
                };
                if let Some(Data::String(frequency)) = _data.get("frequency") {
                    recurrence.frequency = RecurrenceFrequency::from_str(frequency);
                }
                //if recurrence.frequency == RecurrenceFrequency::Weekly
                // not sure if i want this only for weekly or always
                {
                    if let Some(Data::String(day_name)) = _data.get("day") {
                        recurrence
                            .days_of_week
                            .push(parse_weekday_default(day_name));
                    } else if let Some(Data::List(days)) = _data.get("days") {
                        for day in days {
                            if let Data::String(day_name) = day {
                                recurrence
                                    .days_of_week
                                    .push(parse_weekday_default(day_name))
                            }
                        }
                    }
                }

                if let Some(Data::Int(intervall)) = _data.get("intervall") {
                    recurrence.interval = *intervall as i32;
                }

                if let Some(Data::String(start_time)) = _data.get("start-time") {
                    let start_time_naive =
                        date_from_str(start_time).and_time(time_from_str("00:00"));
                    recurrence.start_date = DateTime::from_naive_utc_and_offset(
                        start_time_naive,
                        *Local::now().offset(),
                    );
                }

                Ok(recurrence)
            }
            _ => Err("Data must be Type Object".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub attendee_id: String, // Unique identifier for the attendee
    pub name: String,        // Name of the attendee
    pub email: String,       // Email of the attendee
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

                if event.notification_settings.is_empty() {
                    event.notification_settings.push(Notification::default());
                }
                Ok(event)
            }
            _ => Err("Expected an Object variant".to_string()),
        }
    }
}
