use serde::{Deserialize, Serialize};
use chrono::{NaiveTime, NaiveDate, NaiveDateTime, Local};
use std::fs;
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub timedate: NaiveDateTime,
    pub name: String,
}

pub struct EventManager {
    file_path: PathBuf,
    auto_save: bool,
    events: Vec<Event>,
}

impl EventManager {
    pub fn new(file_path: PathBuf, auto_save: bool)-> EventManager {
        let mut event_manager = EventManager {
            file_path,
            auto_save,
            events: Vec::new(),
        };
        event_manager.read_events_from_file();
        event_manager.list_events();
        event_manager
    }

    pub fn list_events(&self) {
        println!("Events:");
        for (index, event) in self.events.iter().enumerate() {
            println!("\t{index}: {event:?}");
        }
    }

    pub fn save_events(&self) {
        println!("saved Events");
        // Convert the vector of events to a JSON string
        let json_string = serde_json::to_string(&self.events).expect("Failed to convert to JSON");

        // Print the JSON string
        // println!("{}", json_string);
        if let Err(e) = fs::write(&self.file_path, json_string) {
            eprintln!("Failed to save file: {}", e);
        } else {
            println!("Events saved successfully.");
        }
    }

    pub fn read_events_from_file(&mut self) {
        // Read the file contents
        let data = fs::read_to_string(&self.file_path)
            .expect("Unable to read file");

        if let Ok(Some(events)) = serde_json::from_str(&data) {
            self.events=events;
        }
    }

    fn parse_datetime(&self, date_str: &str, time_str: &str) -> Option<NaiveDateTime> {
        // Get today's date if date_str is empty
        let date = if date_str.is_empty() {
            Some(Local::now().naive_utc().date()) // Get today's date in UTC
        } else {
            if date_str.contains('/') {
                NaiveDate::parse_from_str(date_str, "%d/%m/%Y").ok()
                    .or_else(|| NaiveDate::parse_from_str(date_str, "%d/%m").ok())
            } else if date_str.contains('-') {
                NaiveDate::parse_from_str(date_str, "%d-%m-%Y").ok()
                    .or_else(|| NaiveDate::parse_from_str(date_str, "%d-%m").ok())
            } else if date_str.contains('.') {
                NaiveDate::parse_from_str(date_str, "%d.%m.%Y").ok()
                    .or_else(|| NaiveDate::parse_from_str(date_str, "%d.%m").ok())
            } else {
                unreachable!("This should not be valid {}", date_str)
            }
        };

        // Parse the time
        let time = if time_str.contains(':') {
            if time_str.to_lowercase().contains("am") || time_str.to_lowercase().contains("pm") {
                NaiveTime::parse_from_str(time_str, "%I:%M %p").ok()
            } else {
                NaiveTime::parse_from_str(time_str, "%H:%M").ok()
                    .or_else(|| NaiveTime::parse_from_str(time_str, "%H:%M:%S").ok())
            }
        } else {
            None
        };

        // Combine date and time
        match (date, time) {
            (Some(d), Some(t)) => Some(NaiveDateTime::new(d, t)),
            _ => None,
        }
    }

    pub fn add_event_from_str(&mut self, add_str: &str){
        // Define regex patterns for time and date
        let time_pattern = Regex::new(r"(?i)\b(1[0-2]|0?[1-9]):([0-5][0-9]) ?([AP]M)?|([01]?[0-9]|2[0-3])(:[0-5][0-9]){0,2}\b").unwrap();
        let date_pattern = Regex::new(r"(?i)\b(\d{2}\.\d{2}(\.\d{4})?|\d{2}/\d{2}/\d{4}|\d{4}-\d{2}-\d{2}|[A-Za-z]+ \d{1,2}, \d{4})\b").unwrap();

        // Extract the part after "add "
        let entry = add_str.strip_prefix("add ").unwrap_or(add_str);

        // Extract time
        let time_match = time_pattern.find(entry);
        let time_str = time_match.map(|m| m.as_str());

        // Extract date
        let date_match = date_pattern.find(entry);
        let date_str = date_match.map(|m| m.as_str());


        // Extract name
        let mut name = entry.to_string();
        let mut time = String::new();
        let mut date = String::new();
        if let Some(_time) = time_str {
            time=_time.trim().to_string();
            name = name.replace(_time, "").trim().to_string();
        }
        if let Some(_date) = date_str {
            date=_date.trim().to_string();
            name = name.replace(_date, "").trim().to_string();
        }

        let datetime_opt = self.parse_datetime(&date, &time);

        if let Some(datetime) = datetime_opt {
            self.events.push(Event {timedate: datetime, name});
            if self.auto_save {
                self.save_events();
            }
        }
    }
}

/*
   pub fn save_events(data_file_path: &Option<PathBuf>, events: &Vec<Event>) {
   println!("saved Events");
// Convert the vector of events to a JSON string
let json_string = serde_json::to_string(&events).expect("Failed to convert to JSON");

// Print the JSON string
println!("{}", json_string);
if let Some(dfp) = data_file_path {
if let Err(e) = fs::write(dfp, json_string) {
eprintln!("Failed to save file: {}", e);
} else {
println!("Events saved successfully.");
}
} else {
println!("Didn't save file due to missing data directory.");
}
}


pub fn read_events_from_file(file_path: &PathBuf) -> Result<Vec<Event>> {
// Read the file contents
let data = fs::read_to_string(file_path)
.expect("Unable to read file");

// Deserialize the JSON string into a Vec<Event>
let events: Vec<Event> = serde_json::from_str(&data)?;
Ok(events)
}
*/
