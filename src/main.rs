// mod background_service;

//use background_service::service_main;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use rusty_planner_lib::events::event::parse_weekday;
use rusty_planner_lib::events::event::Attendee;
use rusty_planner_lib::events::event::Notification;
use rusty_planner_lib::events::event::NotificationMethod;
use rusty_planner_lib::events::event::Recurrence;
use rusty_planner_lib::events::event::RecurrenceFrequency;
use rusty_planner_lib::events::event::CATEGORIES;
use rusty_planner_lib::events::{
    event::{load_categories, save_categories, Event},
    event_manager::{EventManager, EventManagerMode},
};
use rusty_planner_lib::miscs::help::*;
use rusty_planner_lib::miscs::utils;
use rusty_planner_lib::miscs::utils::datetime_from_str;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

fn main() {
    let _args: Vec<String> = env::args().collect();

    let path = utils::get_path();

    let event_manager: Arc<Mutex<EventManager>>;
    let data_file_path: PathBuf;
    let category_file_path: PathBuf;

    if let Some(fp) = &path {
        data_file_path = fp.clone().join("dates.json");
        category_file_path = fp.clone().join("categories.txt");

        event_manager = EventManager::new(data_file_path.clone(), false, EventManagerMode::Active);
        load_categories(&category_file_path);
    } else {
        eprintln!("error cant create Config file");
        return;
    }

    event_manager.lock().unwrap().read_events_from_file();
    loop {
        let command = get_io(">");
        match command {
            _ if command.starts_with("add") => {
                let event = event_from_io();
                event_manager.lock().unwrap().add_event(event);
            }
            _ if command.starts_with("help") => {
                let command_help = command.strip_prefix("help ").unwrap_or("");
                match command_help {
                    // "add" => print_add_help(), // Assuming print_help() provides help for the "add" command
                    // "save" => print_save_help(),
                    // "remove" => print_remove_help(),
                    // "edit" => print_edit_help(),
                    // "cls" => print_cls_help(),
                    // "list" => print_list_help(),
                    // "clear" => print_clear_help(),
                    // "search" => print_search_help(),
                    // "" => print_help(), // Default help message
                    _ => print_help(), // Fallback for unrecognized commands
                }
            }
            _ if command.starts_with("list") => {
                let input = command.strip_prefix("list").unwrap_or("").trim();
                match input {
                    _ if input.starts_with("event") => {
                        let index = command
                            .strip_prefix("event")
                            .unwrap_or("")
                            .trim()
                            .parse::<usize>();
                        if let Ok(index) = index {
                            if index > 0 {
                                if let Some(event) =
                                    event_manager.lock().unwrap().get_event(index - 1)
                                {
                                    println!("{}", event);
                                } else {
                                    eprintln!("No event found at index {}", index);
                                }
                            } else {
                                eprintln!("Invalid index: {}", index);
                            }
                        } else {
                            event_manager.lock().unwrap().list_events();
                        }
                    }
                    _ if input.starts_with("categories") => {
                        println!("Categories:");
                        for category in CATEGORIES.lock().unwrap().iter() {
                            println!("\t{}", category);
                        }
                    }
                    _ => print_list_help(),
                }
            }
            _ if command.starts_with("exit") => break,
            _ => println!("No such command: '{}'", command),
        }
    }

    save_categories(&category_file_path);
    event_manager.lock().unwrap().save_events();
}

fn get_io(lable: &str) -> String {
    let mut input = String::new();
    print!("{}: ", lable);
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    // print!("\n");
    input.trim().replace("\n", "")
}

///
fn get_datetime(lable: &str) -> DateTime<Local> {
    loop {
        let input = get_io(lable);
        if input == "" {
            return Local::now();
        }
        let x = datetime_from_str(&input);
        if let Ok(x) = x {
            let x = match Local::now().offset().from_local_datetime(&x) {
                chrono::offset::LocalResult::Single(x) => Some(x.with_timezone(&Local)),
                chrono::offset::LocalResult::Ambiguous(_, _)
                | chrono::offset::LocalResult::None => None,
            };
            if let Some(x) = x {
                return x;
            }
        }
        println!("Error: Invalid input")
    }
}

fn event_from_io() -> Event {
    let mut event = Event::default();

    // Basic properties
    event = event.set_title(get_io("Title"));
    event = event.set_description(get_io("Description"));
    event = event.set_start_time(get_datetime("Start Time (e.g., 2025-10-01 15:00)"));
    event = event.set_end_time(get_datetime("End Time (e.g., 2025-10-01 16:00)"));
    event = event.set_location(get_io("Location"));

    // Is all day event?
    event.is_all_day = get_io("Is this an all-day event? (y/n)") == "y";

    // Categories (comma separated)
    let categories_input = get_io("Categories (comma separated, e.g., Work, Personal)");
    event.categories = categories_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Is recurring?
    event = event.set_is_recurring(get_io("Is the event recurring? (y/n)") == "y");

    // If recurring, get recurrence details
    if event.is_recurring {
        println!("Enter recurrence details:");
        let frequency_str =
            get_io("Frequency (hourly, daily, weekly, monthly, yearly)").to_lowercase();
        let frequency =
            RecurrenceFrequency::from_str(&frequency_str).unwrap_or(RecurrenceFrequency::Daily);
        let interval_str = get_io("Interval (e.g., 1 for every day/week/etc.)");
        let interval = interval_str.parse::<i64>().unwrap_or(1);
        let start_date = event.start_time;
        let end_date = {
            let input = get_io("Recurrence end date (YYYY-MM-DD) or leave blank for none");
            if input.is_empty() {
                None
            } else {
                match chrono::NaiveDateTime::parse_from_str(
                    &(input + " 23:59:59"),
                    "%Y-%m-%d %H:%M:%S",
                ) {
                    Ok(naive) => {
                        match Local.from_local_datetime(&naive).single() {
                            Some(dt) => Some(dt),
                            None => None, // ambiguous or invalid local time
                        }
                    }
                    Err(_) => None,
                }
            }
        };

        // Optional specific timing
        let minute = match get_io("Specify minute (0-59) or leave blank") {
            m if m.is_empty() => None,
            m => m.parse::<u32>().ok(),
        };
        let hour = match get_io("Specify hour (0-23) or leave blank") {
            h if h.is_empty() => None,
            h => h.parse::<u32>().ok(),
        };
        let day = match get_io("Specify day of month (1-31) or leave blank") {
            d if d.is_empty() => None,
            d => d.parse::<u32>().ok(),
        };
        let week_day = {
            let wd = get_io("Specify week day (Monday, Tue, etc.) or leave blank");
            if wd.is_empty() {
                None
            } else {
                parse_weekday(&wd)
            }
        };
        let month = match get_io("Specify month (1-12) or leave blank") {
            m if m.is_empty() => None,
            m => m.parse::<u32>().ok(),
        };
        let year = match get_io("Specify year (e.g., 2025) or leave blank") {
            y if y.is_empty() => None,
            y => y.parse::<u32>().ok(),
        };

        let recurrence = Recurrence {
            frequency,
            interval,
            start_date,
            end_date,
            minute,
            hour,
            day,
            week_day,
            month,
            year,
        };

        event = event.set_recurrence(Some(recurrence));
    } else {
        event = event.set_recurrence(None);
    }

    // Attendees
    let mut attendees: Vec<Attendee> = Vec::new();
    println!("Add attendees (enter empty name to stop):");
    loop {
        let name = get_io("Attendee name");
        if name.is_empty() {
            break;
        }
        let email = get_io("Attendee email");
        if email.is_empty() {
            println!("Email cannot be empty, skipping attendee.");
            continue;
        }
        attendees.push(Attendee {
            attendee_id: "None".to_string(),
            name,
            email,
        });
    }
    event = event.set_attendees(attendees);

    // Notification settings
    let mut notifications: Vec<Notification> = Vec::new();
    println!("Add notifications (enter empty method to stop):");
    loop {
        let method_str = get_io("Notification method (email, sms, push)");
        if method_str.is_empty() {
            break;
        }
        let notify_before_str = get_io("Notify before (minutes)");
        let notify_before = notify_before_str.parse::<i64>().unwrap_or(10);

        let method = match method_str.to_lowercase().as_str() {
            "email" => NotificationMethod::Email,
            "sms" => NotificationMethod::Sms,
            "push" => NotificationMethod::Push,
            _ => NotificationMethod::Push,
        };

        notifications.push(Notification {
            notify_before,
            method,
            has_notified: false,
        });
    }
    if notifications.is_empty() {
        notifications.push(Notification::default());
    }
    event = event.set_notification_settings(notifications);

    event
}
