use std::cmp::Ordering;

use crate::utils::{is_valid_date, is_valid_time, parse_duration};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum EventType {
    REPEATING,
    SINGLETIME,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RepeatingWeekDay {
    pub weekday: Weekday,
    pub time: NaiveTime,
}

impl PartialEq<NaiveDate> for RepeatingWeekDay {
    fn eq(&self, other: &NaiveDate) -> bool {
        self.weekday != other.weekday()
    }

    fn ne(&self, other: &NaiveDate) -> bool {
        !self.eq(other)
    }
}

impl PartialEq<NaiveDateTime> for RepeatingWeekDay {
    fn eq(&self, other: &NaiveDateTime) -> bool {
        self.weekday == other.weekday() && self.time == other.time()
    }

    fn ne(&self, other: &NaiveDateTime) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd<NaiveDateTime> for RepeatingWeekDay {
    fn partial_cmp(&self, other: &NaiveDateTime) -> Option<std::cmp::Ordering> {
        let weekday_ordering = self
            .weekday
            .clone()
            .num_days_from_monday()
            .cmp(&other.weekday().num_days_from_monday());
        match weekday_ordering {
            Ordering::Equal => {
                // If weekdays are equal, compare the times
                self.time.clone().partial_cmp(&other.time())
            }
            ordering => Some(ordering),
        }
    }

    fn lt(&self, other: &NaiveDateTime) -> bool {
        std::matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Less))
    }

    fn le(&self, other: &NaiveDateTime) -> bool {
        std::matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    }

    fn gt(&self, other: &NaiveDateTime) -> bool {
        std::matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Greater))
    }

    fn ge(&self, other: &NaiveDateTime) -> bool {
        std::matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
    }
}

// pub struct  RepeatingDay {
//     pub
// }

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
    pub time: Option<NaiveTime>,
    pub date: Option<NaiveDate>,
    pub repeating_day: Option<RepeatingWeekDay>,
    pub event_type: EventType,
    pub has_notified: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub alarm_time: Option<Duration>,
}

// impl PartialEq<NaiveDateTime> for Event {
//     fn eq(&self, other: &NaiveDateTime) -> bool {
//         self.time == other.time() && self.date == other.date()
//     }

//     fn ne(&self, other: &NaiveDateTime) -> bool {
//         !self.eq(other)
//     }
// }

#[allow(dead_code)]
fn next_weekday(start_date: NaiveDate, target_weekday: Weekday) -> NaiveDate {
    let mut days_ahead = target_weekday.num_days_from_monday() as i64
        - start_date.weekday().num_days_from_monday() as i64;
    if days_ahead <= 0 {
        days_ahead += 7; // Move to the next week if the target weekday is today or in the past
    }
    start_date + chrono::Duration::days(days_ahead)
}

impl Event {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            name: "New Event".to_string(),
            time: Some(Utc::now().time()),
            date: Some(Utc::now().date_naive()),
            repeating_day: None,
            has_notified: false,
            alarm_time: None,
            description: None,
            location: None,
            event_type: EventType::SINGLETIME,
        }
    }

    #[allow(dead_code)]
    pub fn new_single_time_event(
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
            time: Some(time),
            date: Some(date),
            repeating_day: None,
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

    #[allow(dead_code)]
    pub fn is_alarm(&mut self, now: NaiveDateTime) -> bool {
        match self.event_type {
            EventType::SINGLETIME => {
                if let (Some(date), Some(time)) = (self.date, self.time) {
                    date.and_time(time) <= now + self.alarm_time.unwrap_or(Duration::minutes(0))
                } else {
                    unreachable!("this should never happen");
                }
            }
            EventType::REPEATING => {
                if let Some(ref repeating_day) = self.repeating_day {
                    repeating_day <= &(now + self.alarm_time.unwrap_or(Duration::minutes(0)))
                } else {
                    unreachable!("this should never happen");
                }
            } // _ => unreachable!("what did u doo"),
        }
    }

    #[allow(dead_code)]
    pub fn get_event_datetime(&mut self) -> NaiveDateTime {
        match self.event_type {
            EventType::SINGLETIME => {
                if let (Some(date), Some(time)) = (self.date, self.time) {
                    date.and_time(time)
                } else {
                    unreachable!("this should never happen");
                }
            }
            EventType::REPEATING => {
                if let Some(ref repeating_day) = self.repeating_day {
                    next_weekday(Utc::now().naive_local().date(), repeating_day.weekday)
                        .and_time(repeating_day.time)
                } else {
                    unreachable!("this should never happen");
                }
            } // _ => unreachable!("will not be needed"),
        }
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

    let event = Event::new_single_time_event(
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
