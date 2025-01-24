use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Event {
    pub name: String,
    pub time: NaiveTime,
    pub date: NaiveDate,
    pub has_notified: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub alarm_time: Option<Duration>,
}

impl Event {
    fn new() -> Self {
        Self {
            name: "New Event".to_string(),
            time: Utc::now().time(),
            date: Utc::now().date_naive(),
            has_notified: false,
            description: None,
            location: None,
            alarm_time: None,
        }
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
