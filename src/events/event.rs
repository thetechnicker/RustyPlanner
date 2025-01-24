use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Event {
    pub name: String,
    pub time: NaiveTime,
    pub date: NaiveDate,
    pub has_notified: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub allarm_time: Option<Duration>,
}
