use chrono::{Duration, NaiveDate, NaiveTime};
use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher};
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, isize, usize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub name: String,
    pub time: NaiveTime,
    pub date: NaiveDate,
    pub has_notified: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub allarm_time: Option<Duration>,
}

#[derive(PartialEq, Eq)]
pub enum EventManagerMode {
    #[allow(dead_code)]
    Active, // manages events, has read/write access
    Passive, // handles notification, read only
}

pub struct EventManager {
    file_path: PathBuf,
    #[allow(dead_code)]
    auto_save: bool,
    events: Vec<Event>,
    #[allow(dead_code)]
    mode: EventManagerMode,
}

impl EventManager {
    pub fn new(
        file_path: PathBuf,
        auto_save: bool,
        mode: EventManagerMode,
    ) -> Arc<Mutex<EventManager>> {
        if EventManagerMode::Passive == mode {
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

        //if let EventManagerMode::Passive = event_manager.lock().unwrap().mode {
        println!("Monitoring file: {:?}", file_path);
        EventManager::monitor_file(event_manager.clone(), file_path);
        //}

        event_manager
    }

    #[allow(dead_code)]
    pub fn list_events(&self) {
        println!("Events:");
        for (index, event) in self.events.iter().enumerate() {
            println!("\t{index}: {event:?}");
        }
    }

    pub fn save_events(&self) {
        //if let EventManagerMode::Active = self.mode {
        // println!("saved Events");
        // Convert the vector of events to a JSON string
        let json_string = serde_json::to_string(&self.events).expect("Failed to convert to JSON");

        // Print the JSON string
        // println!("{}", json_string);
        if let Err(e) = fs::write(&self.file_path, json_string) {
            eprintln!("Failed to save file: {}", e);
        } else {
            println!("Events saved successfully.");
        }
        /*} else {
            println!("Cannot save events in Passive mode.");
        }*/
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

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        if EventManagerMode::Active == self.mode {
            self.events.clear();
            if self.auto_save {
                self.save_events();
            }
        } else {
            println!("Cannot clear events in Passive mode.");
        }
    }

    #[allow(dead_code)]
    pub fn get_event(&mut self, x: isize) -> Option<&Event> {
        self.events.get(x as usize)
    }

    #[allow(dead_code)]
    pub fn iter_events(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    #[allow(dead_code)]
    pub fn iter_events_mut(&mut self) -> impl Iterator<Item = &mut Event> {
        self.events.iter_mut()
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

    #[allow(dead_code)]
    pub fn add_event(&mut self, event: Event) -> isize {
        if EventManagerMode::Active == self.mode {
            self.events.push(event);
            if self.auto_save {
                self.save_events();
            }
            (self.events.len() - 1) as isize
        } else {
            return -1;
        }
    }

    #[allow(dead_code)]
    pub fn remove_event(&mut self, x: usize) -> Option<Event> {
        if x < self.events.len() {
            Some(self.events.remove(x))
        } else {
            None
        }
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
                    //println!("changed: {:?}", event);
                    event_manager.lock().unwrap().read_events_from_file();
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_manager() {
        use super::*;
        use std::env;
        use std::fs::File;
        use std::io::Write;

        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("test.json");

        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"[]").expect("Failed to write to file");

        let event_manager = EventManager::new(file_path.clone(), false, EventManagerMode::Active);

        let event = Event {
            name: String::from("Test Event"),
            time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            date: chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            has_notified: false,
            description: Some(String::from("Test Description")),
            location: Some(String::from("Test Location")),
            allarm_time: Some(chrono::Duration::minutes(10)),
        };

        let index = event_manager.lock().unwrap().add_event(event);

        assert_eq!(
            event_manager
                .lock()
                .unwrap()
                .get_event(index as isize)
                .unwrap()
                .name,
            "Test Event"
        );

        let mut event = event_manager
            .lock()
            .unwrap()
            .remove_event(index as usize)
            .unwrap();

        assert_eq!(event.name, "Test Event");

        event.name = String::from("New Test Event");

        event_manager.lock().unwrap().add_event(event);

        assert_eq!(
            event_manager
                .lock()
                .unwrap()
                .get_event(index as isize)
                .unwrap()
                .name,
            "New Test Event"
        );

        event_manager.lock().unwrap().clear();

        assert_eq!(
            event_manager.lock().unwrap().get_event(index as isize),
            None
        );

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_event() {
        use super::*;
        let event = Event {
            name: "Test Event".to_string(),
            time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            date: chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            has_notified: false,
            description: Some("Test Description".to_string()),
            location: Some("Test Location".to_string()),
            allarm_time: Some(chrono::Duration::minutes(10)),
        };

        assert_eq!(event.name, "Test Event");
        assert_eq!(
            event.time,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()
        );
        assert_eq!(
            event.date,
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()
        );
        assert_eq!(event.has_notified, false);
        assert_eq!(event.description, Some("Test Description".to_string()));
        assert_eq!(event.location, Some("Test Location".to_string()));
        assert_eq!(event.allarm_time, Some(chrono::Duration::minutes(10)));
    }
}
