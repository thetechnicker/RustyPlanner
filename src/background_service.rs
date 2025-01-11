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
        println!("Background service is running...");
        // event_manager.lock().unwrap().list_events();
        for (index, event) in event_manager.lock().unwrap().iter_events().enumerate() {
            println!("\t{index}: {event:?}");
            // is it time to notify the user?
            if event.timedate <= chrono::Local::now().naive_local() {
                println!("Time to notify the user!");
                // Notification::new()
                //     .summary(&event.name)
                //     .body("")
                //     .show()
                //     .expect("Failed to show notification");
                send_notification(&event.name, "");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
