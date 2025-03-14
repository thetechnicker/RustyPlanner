use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher};
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::miscs::arg_parsing::parse_data;

use super::event::Event;

pub enum SearchType {
    Title,
    Description,
    Location,
    Category,
    Attendee,
    Date,
    FullText,
}

impl SearchType {
    pub fn from(s: &str) -> SearchType {
        match s {
            "title" => SearchType::Title,
            "description" => SearchType::Description,
            "location" => SearchType::Location,
            "category" => SearchType::Category,
            "attendee" => SearchType::Attendee,
            "date" => SearchType::Date,
            "fulltext" => SearchType::FullText,
            _ => SearchType::Title,
        }
    }
}

#[derive(PartialEq, Eq)]
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
        if EventManagerMode::Passive == mode && !file_path.exists() {
            eprintln!("Error: File to monitor does not exist: {:?}", file_path);
            std::process::exit(1);
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

    pub fn monitor_file(event_manager: Arc<Mutex<EventManager>>, file_path: PathBuf) {
        std::thread::spawn(move || {
            futures::executor::block_on(async {
                if let Err(e) = async_watch(event_manager, file_path).await {
                    println!("error: {:?}", e)
                }
            });
        });
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

    pub fn save_events(&self) {
        let json_string = serde_json::to_string(&self.events).expect("Failed to convert to JSON");

        if let Err(e) = fs::write(&self.file_path, json_string) {
            eprintln!("Failed to save file: {}", e);
        } else {
            println!("Events saved successfully.");
        }
    }

    pub fn list_events(&self) {
        println!("Events:");
        for (index, event) in self.events.iter().enumerate() {
            println!("{}: {}", index + 1, event);
        }
    }

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

    pub fn get_event(&mut self, x: usize) -> Option<&Event> {
        Some(&self.events[x])
    }

    pub fn get_event_mut(&mut self, x: usize) -> Option<&mut Event> {
        Some(&mut self.events[x])
    }

    pub fn iter_events_mut(&mut self) -> impl Iterator<Item = &mut Event> {
        self.events.iter_mut()
    }

    pub fn add_event(&mut self, mut event: Event) -> isize {
        if EventManagerMode::Active == self.mode {
            if event.event_id.is_empty() {
                event.event_id = format!("#{}", self.events.len() + 1);
            }
            self.events.push(event);
            if self.auto_save {
                self.save_events();
            }
            (self.events.len() - 1) as isize
        } else {
            -1
        }
    }

    pub fn remove_event(&mut self, x: usize) -> Option<Event> {
        if x < self.events.len() {
            Some(self.events.remove(x))
        } else {
            None
        }
    }

    pub fn add_event_from_str(&mut self, string: &str) -> isize {
        let data = parse_data(string, 0);
        data.print(0);
        let event = Event::from_data(data);
        match event {
            Ok(e) => {
                // println!("{}", e);
                self.add_event(e)
            }
            Err(_e) => {
                // eprintln!("{}", e);
                -1
            }
        }
    }

    #[allow(dead_code)]
    pub fn sort_events_by(&mut self, sort_by: SearchType) -> Vec<Event> {
        let mut result: Vec<Event> = self.events.clone();
        match sort_by {
            SearchType::Title => result.sort_by(|a, b| a.title.cmp(&b.title)),
            SearchType::Date => result.sort_by(|a, b| a.start_time.cmp(&b.start_time)),
            SearchType::Location => result.sort_by(|a, b| a.location.cmp(&b.location)),
            _ => todo!(),
        }
        result
    }

    pub fn search_event(&self, search_string: &str, search_type: SearchType) -> Vec<&Event> {
        let mut result: Vec<&Event> = Vec::new();
        for event in self.events.iter() {
            match search_type {
                SearchType::Title => {
                    if event.title.contains(search_string) {
                        result.push(event);
                    }
                }
                SearchType::Date => {
                    if event
                        .start_time
                        .format("%Y-%m-&d %H:%M")
                        .to_string()
                        .contains(search_string)
                    {
                        result.push(event);
                    }
                }
                SearchType::FullText => {
                    if serde_json::to_string(event)
                        .expect("Failed to convert to JSON")
                        .contains(search_string)
                    {
                        result.push(event);
                    }
                }
                _ => todo!(),
            }
        }
        result
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
                //if event_manager.lock().unwrap().mode == EventManagerMode::Passive
                //    && !event.kind.is_access()
                //{
                //    println!("{:?}", event.kind);
                //}
                if event.kind.is_modify() {
                    event_manager.lock().unwrap().read_events_from_file();
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
