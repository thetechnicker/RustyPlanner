mod events;

use std::time::Duration;
use std::thread;
use events::{EventManager, EventManagerMode};
use std::path::PathBuf;
use directories::BaseDirs;
use std::fs;

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

    let mut event_manager: EventManager;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), true, EventManagerMode::Passive);
    } else {
        eprintln!("error cant create Config file");
        return;
    }

    // event_manager.read_events_from_file();

    loop {
        println!("Background service is running...");
        event_manager.list_events();
        thread::sleep(Duration::from_secs(5));
    }
}
