use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};
use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher};
use notify::{Event as NotifyEvent, RecursiveMode, Result, Watcher};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub timedate: NaiveDateTime,
    pub name: String,
}

pub enum EventManagerMode {
    Active,  // manages events, has read/write access
    Passive, // handles notification, read only
}

pub struct EventManager {
    file_path: PathBuf,
    auto_save: bool,
    events: Vec<Event>,
    mode: EventManagerMode,
}

impl EventManager {
    pub fn new(
        file_path: PathBuf,
        auto_save: bool,
        mode: EventManagerMode,
    ) -> Arc<Mutex<EventManager>> {
        if let EventManagerMode::Passive = mode {
            if !file_path.exists() {
                eprintln!("Error: File to monitor does not exist: {:?}", file_path);
                std::process::exit(1);
            }
        }

        let event_manager = Arc::new(Mutex::new(EventManager {
            file_path: file_path.clone(),
            auto_save,
            events: Vec::new(),
            mode,
        }));

        event_manager.lock().unwrap().read_events_from_file();

        if let EventManagerMode::Passive = event_manager.lock().unwrap().mode {
            println!("Monitoring file: {:?}", file_path);
            EventManager::monitor_file(event_manager.clone(), file_path);
        }

        event_manager
    }

    pub fn list_events(&self) {
        println!("Events:");
        for (index, event) in self.events.iter().enumerate() {
            println!("\t{index}: {event:?}");
        }
    }

    pub fn save_events(&self) {
        if let EventManagerMode::Active = self.mode {
            println!("saved Events");
            // Convert the vector of events to a JSON string
            let json_string =
                serde_json::to_string(&self.events).expect("Failed to convert to JSON");

            // Print the JSON string
            // println!("{}", json_string);
            if let Err(e) = fs::write(&self.file_path, json_string) {
                eprintln!("Failed to save file: {}", e);
            } else {
                println!("Events saved successfully.");
            }
        } else {
            println!("Cannot save events in Passive mode.");
        }
    }

    pub fn read_events_from_file(&mut self) {
        if self.file_path.exists() {
            // Read the file contents
            // println!("{}", &self.file_path.display());
            let data = fs::read_to_string(&self.file_path).expect("Unable to read file");

            if let Ok(Some(events)) = serde_json::from_str(&data) {
                self.events = events;
            }
        }
    }

    fn parse_datetime(&self, date_str: &str, time_str: &str) -> Option<NaiveDateTime> {
        // Get today's date if date_str is empty
        let date = if date_str.is_empty() {
            Some(Local::now().naive_utc().date()) // Get today's date in UTC
        } else {
            if date_str.contains('/') {
                NaiveDate::parse_from_str(date_str, "%d/%m/%Y")
                    .ok()
                    .or_else(|| NaiveDate::parse_from_str(date_str, "%d/%m").ok())
            } else if date_str.contains('-') {
                NaiveDate::parse_from_str(date_str, "%d-%m-%Y")
                    .ok()
                    .or_else(|| NaiveDate::parse_from_str(date_str, "%d-%m").ok())
            } else if date_str.contains('.') {
                NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
                    .ok()
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
                NaiveTime::parse_from_str(time_str, "%H:%M")
                    .ok()
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

    pub fn add_event_from_str(&mut self, add_str: &str) {
        if let EventManagerMode::Active = self.mode {
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
                time = _time.trim().to_string();
                name = name.replace(_time, "").trim().to_string();
            }
            if let Some(_date) = date_str {
                date = _date.trim().to_string();
                name = name.replace(_date, "").trim().to_string();
            }

            let datetime_opt = self.parse_datetime(&date, &time);

            if let Some(datetime) = datetime_opt {
                self.events.push(Event {
                    timedate: datetime,
                    name,
                });
                if self.auto_save {
                    self.save_events();
                }
            }
        } else {
            println!("Cannot add events in Passive mode.");
        }
    }

    pub fn clear(&mut self) {
        if let EventManagerMode::Active = self.mode {
            self.events.clear();
            if self.auto_save {
                self.save_events();
            }
        } else {
            println!("Cannot clear events in Passive mode.");
        }
    }

    pub fn monitor_file(event_manager: Arc<Mutex<EventManager>>, file_path: PathBuf) {
        std::thread::spawn(move || {
            futures::executor::block_on(async {
                if let Err(e) = async_watch(event_manager, file_path).await {
                    println!("error: {:?}", e)
                }
            });
        });
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<NotifyEvent>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch(event_manager: Arc<Mutex<EventManager>>, path: PathBuf) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    println!("changed: {:?}", event);
                    event_manager.lock().unwrap().read_events_from_file();
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
