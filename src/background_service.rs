mod events;
mod miscs;

use daemonize::Daemonize;
use events::event_manager::{EventManager, EventManagerMode};
use miscs::utils::get_path;
use std::fs;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uzers::{get_current_gid, get_current_uid};

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

#[allow(unused_mut)]
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

            // if event.is_alarm(time) && !event.has_notified {
            //     println!("Time to notify the user!");
            //     // let event_datetime = event.get_event_datetime();
            //     let message = format!("{}", event);
            //     send_notification(&event.name, &message);
            //     event.has_notified = true;
            //     has_to_save = true;
            // }
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
