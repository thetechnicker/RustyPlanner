use crate::utils::{is_valid_date, is_valid_time, parse_duration};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum EventType {
    REPEATING,
    SINGLETIME,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
enum ParseMode {
    Desc,
    Loc,
    AlarmTime,
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Event {
    pub name: String,
    pub time: NaiveTime,
    pub date: NaiveDate,
    pub event_type: EventType,
    pub has_notified: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub alarm_time: Option<Duration>,
}

impl Event {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            name: "New Event".to_string(),
            time: Utc::now().time(),
            date: Utc::now().date_naive(),
            has_notified: false,
            alarm_time: None,
            description: None,
            location: None,
            event_type: EventType::SINGLETIME,
        }
    }

    #[allow(dead_code)]
    pub fn new(
        name: String,
        time: NaiveTime,
        date: NaiveDate,
        has_notified: bool,
        alarm_time: Option<Duration>,
        description: Option<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            name,
            time,
            date,
            has_notified,
            description,
            location,
            alarm_time,
            event_type: EventType::SINGLETIME,
        }
    }

    #[allow(dead_code)]
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[allow(dead_code)]
pub fn event_from_cmd(input: &str) -> Option<Event> {
    let command = input.strip_prefix("add ").unwrap_or("");
    let parts: Vec<&str> = command.split_whitespace().collect();

    let mut name: String = String::from("");
    let mut time: Option<NaiveTime> = None;
    let mut date: Option<NaiveDate> = None;
    let mut location: String = String::from("");
    let mut description: String = String::from("");
    let mut alarm_time: Option<Duration> = None;

    let mut is_name = true;
    let mut mode = ParseMode::None;
    for part in parts {
        if date.is_none() {
            if let Some(_date) = is_valid_date(part) {
                date = Some(_date);
                is_name = false;
                continue;
            }
        }
        if time.is_none() {
            if let Some(_time) = is_valid_time(part) {
                time = Some(_time);
                is_name = false;
                continue;
            }
        }
        if is_name {
            name += part;
            name += " ";
        } else {
            match part {
                "-d" => {
                    mode = ParseMode::Desc;
                    continue;
                }
                "-l" => {
                    mode = ParseMode::Loc;
                    continue;
                }
                "-a" => {
                    mode = ParseMode::AlarmTime;
                    continue;
                }
                _ => {
                    //mode=ParseMode::None;
                }
            }

            match mode {
                ParseMode::Desc => {
                    description += part;
                    description += " ";
                }
                ParseMode::Loc => {
                    location += part;
                    location += " ";
                }
                ParseMode::AlarmTime => {
                    if alarm_time.is_none() {
                        alarm_time = Some(parse_duration(part).expect("Failed Parsing"));
                    }
                }
                ParseMode::None => {
                    //println!("idk where to put {}", part);
                }
            }
        }
    }

    if date.is_none() {
        eprintln!("Error: Date must be provided.");
        return None;
    }
    if time.is_none() {
        eprintln!("Error: Time must be provided.");
        return None;
    }
    if is_name {
        eprintln!("Error: Name not Defined");
        return None;
    }

    name = name.trim().to_owned();

    let event = Event::new(
        name,
        time.unwrap(),
        date.unwrap(),
        false,
        alarm_time,
        Some(description.trim().to_owned()),
        Some(location.trim().to_owned()),
    );
    Some(event)
}
