mod events;
mod notification;
mod utils;

use events::{EventManager, EventManagerMode};
// use notify_rust::Notification;
use daemonize::Daemonize;
use notification::send_notification;
use std::fs;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use users::{get_current_gid, get_current_uid};
use utils::get_path;

use signal_hook::flag;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> Result<(), Error> {
    let stdout = File::create("/tmp/RustyPlannerDaemon.out").unwrap();
    let stderr = File::create("/tmp/RustyPlannerDaemon.err").unwrap();

    let user = get_current_uid();
    let group = get_current_gid();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/RustyPlannerDaemon.pid") // Every method except `new` and `start`
        .chown_pid_file(true)
        .working_directory("/tmp") // for default behaviour.
        .user(user) // Group name
        .group(group) // Group name
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            println!("Success, daemonized");
            main_loop()
        }
        Err(e) => {
            eprintln!("Error, {}", e);
            Err(Error::new(
                std::io::ErrorKind::Other,
                "Error, can't daemonize",
            ))
        }
    }
}

pub fn main_loop() -> Result<(), Error> {
    let term = Arc::new(AtomicBool::new(false));

    flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    let data_file_path = get_path();

    let event_manager: Arc<Mutex<EventManager>>;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), false, EventManagerMode::Passive);
    } else {
        eprintln!("Can't open Event File");
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "Can't open Event File",
        ));
    }

    // event_manager.lock().unwrap().read_events_from_file();

    while !term.load(Ordering::Relaxed) {
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
                has_to_save = true;
            }
        }
        if has_to_save {
            event_manager.lock().unwrap().save_events();
        }
        thread::sleep(Duration::from_millis(250));
    }

    println!("Received SIGTERM kill signal. Exiting...");

    fs::remove_file("/tmp/RustyPlannerDaemon.pid")?;

    Ok(())
}
