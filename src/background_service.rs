mod events;
mod miscs;

use chrono::Duration;
use daemonize::Daemonize;
use events::event::NotificationMethod;
use events::event_manager::{EventManager, EventManagerMode};
use miscs::notification::send_notification;
use miscs::utils::get_path;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;
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
        let now = chrono::Local::now();
        println!(
            "{}: Background service is running...",
            now.format("%Y-%m-%d %H:%M:%S")
        );
        // event_manager.lock().unwrap().list_events();
        let mut has_to_save = false;
        for (index, event) in event_manager.lock().unwrap().iter_events_mut().enumerate() {
            println!("\t{index}: {event:?}");
            if !event.is_recurring {
                for notification in event.notification_settings.iter_mut() {
                    if event.start_time - Duration::minutes(notification.notify_before) <= now
                        && !notification.has_notified
                    {
                        match notification.method {
                            NotificationMethod::Push => {
                                send_notification(&event.title, &event.description)
                            }
                            NotificationMethod::Email => todo!(),
                            NotificationMethod::Sms => todo!(),
                        }
                        notification.has_notified = true;
                        has_to_save = true;
                    }
                }
            }
        }
        if has_to_save {
            println!("Saving events...");
            event_manager.lock().unwrap().save_events();
        }
        thread::sleep(StdDuration::from_millis(500)); // old values: 250ms
    }

    println!("Received SIGTERM kill signal. Exiting...");

    //fs::remove_file("/tmp/RustyPlannerDaemon.pid")?;

    Ok(())
}
