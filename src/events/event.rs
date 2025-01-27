use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum EventType {
    REPEATING,
    SINGLETIME,
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
