mod events;

use events::EventManager;
use std::io::{self, Write};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

fn main() {
    //let mut events: Vec<Event> = Vec::new();


    let data_file_path: Option<PathBuf>;

    if let Some(base_dirs) = BaseDirs::new() {
        let data_base_dir = base_dirs.data_dir();

        println!("Data Directory: {:?}", data_base_dir);

        let data_dir = data_base_dir.join("RustyPlanner");

        fs::create_dir_all(data_dir.clone()).expect("Failed to create data directory");

        data_file_path = Some(data_dir.join("dates.json"));

    } else {
        eprintln!("Could not find base directories.");
        data_file_path = None;
    }

    let mut event_manager: EventManager;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), true);
    } else {
        eprintln!("error cant create Config file");
        return;
    }

    event_manager.read_events_from_file();

    loop {
        let mut input = String::new();

        // Read from standard input
        print!("Please enter some input: ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();

        //println!("You entered: {}", trimmed);

        if trimmed.to_lowercase() == "exit" {
            break;
        } else if trimmed.to_lowercase() == "clear" {
            //event_manager.clear();
        } else if trimmed.to_lowercase() == "list" {
            event_manager.list_events();
        }

        if trimmed.to_lowercase().starts_with("add") {
            event_manager.add_event_from_str(trimmed);
            /*
               match event {
               Some(e) => {
               println!("event name: {}", e.name);
               println!("event datetime: {}", e.timedate);
               events.push(e);
               },
               None => println!("event couldnt be parsed!"),
               }
               */
            //save_events(&data_file_path, &events);
        }
    }
}

/*
   fn parse_add(add_str: &str) -> Option<Event> {
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

let datetime_opt = parse_datetime(&date, &time);

match datetime_opt {
Some(datetime) => Some(Event {timedate: datetime, name: name}),
_ => None,
}
}

fn parse_datetime(date_str: &str, time_str: &str) -> Option<NaiveDateTime> {
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
*/
