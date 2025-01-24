use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher};
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, isize, usize};

use super::event::Event;

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
    pub fn get_event(&mut self, x: usize) -> Option<&Event> {
        self.events.get(x)
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

    #[allow(dead_code)]
    pub fn replace_event(&mut self, x: usize, event: Event) -> Option<Event> {
        let mut _event: Option<Event> = None;
        if x < self.events.len() {
            _event = Some(std::mem::replace(&mut self.events[x], event));
        } else {
            _event = None;
        }
        if self.auto_save {
            self.save_events();
        }
        _event
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
