mod events;
mod notification;

use directories::BaseDirs;
use events::{EventManager, EventManagerMode};
// use notify_rust::Notification;
use notification::send_notification;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
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

    let event_manager: Arc<Mutex<EventManager>>;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), true, EventManagerMode::Passive);
    } else {
        eprintln!("error cant create Config file");
        return;
    }

    // event_manager.lock().unwrap().read_events_from_file();

    loop {
        let time = chrono::Local::now().naive_local();
        println!(
            "{}: Background service is running...",
            time.format("%Y-%m-%d %H:%M:%S")
        );
        // event_manager.lock().unwrap().list_events();
        let mut has_to_save = false;
        for (index, event) in event_manager.lock().unwrap().iter_events_mut().enumerate() {
            println!("\t{index}: {event:?}");
            // is it time to notify the user?
            let mut event_datetime = event.date.and_time(event.time);
            if let Some(alarm_time) = event.allarm_time {
                event_datetime -= alarm_time;
            }
            if event_datetime <= chrono::Local::now().naive_local() && event.has_notified == false {
                println!("Time to notify the user!");
                let message = format!(
                    "Event: {}\nDescription: {}\nLocation {}\nDate: {}\nTime: {}",
                    event.name,
                    event.description.as_ref().unwrap_or(&String::from("")),
                    event.location.as_ref().unwrap_or(&String::from("")),
                    event.date,
                    event.time
                );
                send_notification(&event.name, &message);
                event.has_notified = true;
                has_to_save=true;
            }
        }
        if has_to_save {
            event_manager.lock().unwrap().save_events();
        }
        thread::sleep(Duration::from_millis(250));
    }
}
