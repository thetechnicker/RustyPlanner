mod background_service;

use background_service::service_main;
use chrono::DateTime;
use chrono::Local;
use rusty_planner_lib::events::{
    event::{
        load_categories, save_categories, Attendee, Event, Notification, NotificationMethod,
        CATEGORIES,
    },
    event_manager::{EventManager, EventManagerMode, SearchType},
};
use rusty_planner_lib::miscs::{
    help::{
        print_add_help, print_clear_help, print_cls_help, print_edit_help, print_help,
        print_list_help, print_remove_help, print_save_help, print_search_help,
    },
    utils::{clear_screen, date_from_str, get_path, time_from_str},
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = get_path();

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

    if args.len() > 1 {
        if args[1] == "service" {
            if args.len() > 2 {
                match args[2].as_str() {
                    "start" => {
                        service_start();
                    }
                    "stop" => {
                        service_stop();
                    }
                    "restart" => {
                        service_restart();
                    }
                    _ => {
                        eprintln!("Unknown service command: {}", args[2]);
                    }
                }
            } else {
                eprintln!("Service command required");
            }
        } else {
            command_mode(&event_manager, &args[1..]);
        }
    } else {
        event_manager.lock().unwrap().list_events();
        loop_mode(&event_manager);
    }
    save_categories(&category_file_path);
}

fn service_start() {
    service_main().expect("Failed to start background service");
}

fn service_stop() {
    let pid = fs::read_to_string("/tmp/RustyPlannerDaemon.pid").expect("Failed to read PID file");
    let _output = Command::new("kill")
        .arg(pid.trim())
        .output()
        .expect("Failed to stop background service");
    println!("Service stopped, output: {:?}", _output);
    let err_msg = format!("error trying to kill background service. pid: {}", pid);
    fs::remove_file("/tmp/RustyPlannerDaemon.pid").expect(&err_msg);
}

fn service_restart() {
    service_stop();
    service_start();
}

fn loop_mode(event_manager: &Arc<Mutex<EventManager>>) {
    loop {
        let mut input = String::new();

        // Read from standard input
        print!("Please enter some input: ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();

        //println!("You entered: {}", trimmed);

        if trimmed.to_lowercase() == "exit" {
            break;
        } else {
            parse_commands(trimmed, event_manager);
        }
    }
}

fn command_mode(event_manager: &Arc<Mutex<EventManager>>, commands: &[String]) {
    let command = commands.join(" ");

    parse_commands(&command, event_manager);
}

// #[allow(unused_mut)]
fn parse_commands(command: &str, event_manager: &Arc<Mutex<EventManager>>) {
    match command {
        _ if command.starts_with("add") => {
            let input = command.strip_prefix("add").unwrap_or("").trim();
            match input {
                _ if input.starts_with("event") => {
                    let string = input.strip_prefix("event").unwrap_or(command).trim();
                    add_event_loop(string, event_manager);
                }
                _ if input.starts_with("category") => {
                    let category = input.strip_prefix("category").unwrap_or("").trim();
                    if !category.is_empty() {
                        CATEGORIES.lock().unwrap().push(category.to_string());
                        println!("Category added: {}", category);
                    } else {
                        eprintln!("Invalid category name");
                    }
                }
                _ => print_add_help(),
            }
        }
        _ if command.starts_with("edit") => {
            let index = command.strip_prefix("edit").unwrap_or("").trim();
            let index = index.parse::<usize>().unwrap_or(0);
            if index > 0 {
                update_event(
                    event_manager
                        .lock()
                        .unwrap()
                        .get_event_mut(index - 1)
                        .unwrap(),
                );
                event_manager.lock().unwrap().save_events();
            } else {
                eprintln!("Invalid index: {}", index);
            }
        }
        _ if command.starts_with("remove") => {
            let index = command.strip_prefix("remove").unwrap_or("").trim();
            let index = index.parse::<usize>().unwrap_or(0);
            if index > 0 {
                event_manager.lock().unwrap().remove_event(index - 1);
                event_manager.lock().unwrap().save_events();
            } else {
                eprintln!("Invalid index: {}", index);
            }
        }
        _ if command.starts_with("help") => {
            let command_help = command.strip_prefix("help ").unwrap_or("");
            match command_help {
                "add" => print_add_help(), // Assuming print_help() provides help for the "add" command
                "save" => print_save_help(),
                "remove" => print_remove_help(),
                "edit" => print_edit_help(),
                "cls" => print_cls_help(),
                "list" => print_list_help(),
                "clear" => print_clear_help(),
                "search" => print_search_help(),
                "" => print_help(), // Default help message
                _ => print_help(),  // Fallback for unrecognized commands
            }
        }
        _ if command.starts_with("search") => {
            let (search_type_str, search_query) = command
                .strip_prefix("search")
                .unwrap_or("")
                .trim()
                .split_once(" ")
                .unwrap_or(("", ""));

            for (i, event) in event_manager
                .lock()
                .unwrap()
                .search_event(search_query, SearchType::from(search_type_str))
                .iter()
                .enumerate()
            {
                println!("{}, {}", i + 1, event);
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
                            if let Some(event) = event_manager.lock().unwrap().get_event(index - 1)
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
        "save" => {
            event_manager.lock().unwrap().save_events();
        }
        "clear" => {
            event_manager.lock().unwrap().clear();
            event_manager.lock().unwrap().save_events();
        }
        "cls" => {
            clear_screen();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help(); // Suggest help for valid commands
        }
    }
}

fn add_event_loop(input: &str, event_manager: &Arc<Mutex<EventManager>>) {
    let index = event_manager.lock().unwrap().add_event_from_str(input);
    if index < 0 {
        eprintln!("Error when trying to add the Event");
        return;
    }
    println!(
        "{}",
        event_manager
            .lock()
            .unwrap()
            .get_event(index as usize)
            .unwrap()
    );
    let mut attempts = 0;
    loop {
        println!("What would you like to do with the event?");
        println!("1. Keep");
        println!("2. Discard");
        println!("3. Edit");

        let mut choice = String::new();
        print!("Enter your choice (1/2/3): ");
        io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading input
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                event_manager.lock().unwrap().save_events();
                break; // Exit the loop
            }
            "2" => {
                // Discard the event
                event_manager.lock().unwrap().remove_event(index as usize);
                println!("Event has been discarded.");
                break; // Exit the loop
            }
            "3" => {
                update_event(
                    event_manager
                        .lock()
                        .unwrap()
                        .get_event_mut(index as usize)
                        .unwrap(),
                );
                attempts = 0; // Reset attempts after a successful edit
            }
            _ => {
                attempts += 1; // Increment the invalid attempts counter
                println!("Invalid choice. Please enter 1, 2, or 3.");
                if attempts >= 10 {
                    println!("Too many invalid attempts. The event will be discarded.");
                    break; // Exit the loop after 10 invalid attempts
                }
            }
        }
    }
}

fn ask_user(prompt: &str, default: &str) -> String {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn update_event(event: &mut Event) {
    // Update title
    let new_title = ask_user("Enter new title", &event.title);
    event.update_title(new_title);

    // Update description
    let new_description = ask_user("Enter new description", &event.description);
    event.update_description(new_description);

    // Update location
    let new_location = ask_user("Enter new location", &event.location);
    event.update_location(new_location);

    // Update start time
    {
        let time = time_from_str(&ask_user(
            "Enter new start time",
            &event.start_time.time().format("%H:%M").to_string(),
        ));
        let date = date_from_str(&ask_user(
            "Enter new start date",
            &event.start_time.date_naive().format("%Y-%m-%d").to_string(),
        ));

        event.start_time =
            DateTime::from_naive_utc_and_offset(date.and_time(time), *Local::now().offset());
    }

    // Update end time
    {
        let time = time_from_str(&ask_user(
            "Enter new end time",
            &event.start_time.time().format("%H:%M").to_string(),
        ));
        let date = date_from_str(&ask_user(
            "Enter new end date",
            &event.start_time.date_naive().format("%Y-%m-%d").to_string(),
        ));

        event.end_time =
            DateTime::from_naive_utc_and_offset(date.and_time(time), *Local::now().offset());
    }

    // Update is_recurring
    let new_is_recurring_str = ask_user(
        "Is the event recurring? (true/false)",
        &event.is_recurring.to_string(),
    );
    if let Ok(new_is_recurring) = new_is_recurring_str.parse::<bool>() {
        event.update_is_recurring(new_is_recurring);
    } else {
        println!("Invalid input for is_recurring. Keeping the original value.");
    }

    // Update attendees
    loop {
        println!("Current attendees:");
        for attendee in &event.attendees {
            println!("Attendee: {}", attendee);
        }
        let action = ask_user(
            "Do you want to add, remove, edit an attendee, or done? (add/remove/edit/done)",
            "done",
        );
        match action.as_str() {
            "add" => {
                let attendee_id = ask_user("Enter attendee ID", "");
                let name = ask_user("Enter attendee name", "");
                let email = ask_user("Enter attendee email", "");
                let new_attendee = Attendee {
                    attendee_id,
                    name,
                    email,
                };
                event.add_attendee(new_attendee);
            }
            "remove" => {
                let index_str = ask_user("Enter index of attendee to remove", "");
                if let Ok(index) = index_str.parse::<usize>() {
                    if event.remove_attendee(index).is_none() {
                        println!("No attendee found at index {}", index);
                    }
                } else {
                    println!("Invalid index. Please enter a number.");
                }
            }
            "edit" => {
                let index_str = ask_user("Enter index of attendee to edit", "");
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < event.attendees.len() {
                        let attendee_id =
                            ask_user("Enter new attendee ID", &event.attendees[index].attendee_id);
                        let name =
                            ask_user("Enter new attendee name", &event.attendees[index].name);
                        let email =
                            ask_user("Enter new attendee email", &event.attendees[index].email);
                        event.attendees[index] = Attendee {
                            attendee_id,
                            name,
                            email,
                        };
                    } else {
                        println!("No attendee found at index {}", index);
                    }
                } else {
                    println!("Invalid index. Please enter a number.");
                }
            }
            "done" => break,
            _ => println!("Invalid action. Please enter add, remove, edit, or done."),
        }
    }

    // Update notification settings
    loop {
        println!("Current notifications:");
        for notification in &event.notification_settings {
            println!("Notification: {}", notification);
        }
        let action = ask_user(
            "Do you want to add, remove, edit a notification, or done? (add/remove/edit/done)",
            "done",
        );
        match action.as_str() {
            "add" => {
                let notify_before_str = ask_user("Enter notify before (in minutes)", "10");
                let method_str = ask_user("Enter notification method (Email/SMS/Push)", "Email");
                let method = match method_str.to_lowercase().as_str() {
                    "email" => NotificationMethod::Email,
                    "sms" => NotificationMethod::Sms,
                    "push" => NotificationMethod::Push,
                    _ => {
                        println!("Invalid method. Defaulting to Email.");
                        NotificationMethod::Email
                    }
                };
                let notify_before = notify_before_str.parse::<i64>().unwrap_or(10); // Default to 10 minutes if parsing fails
                let new_notification = Notification {
                    notify_before,
                    method,
                    has_notified: false,
                };
                event.add_notification(new_notification);
            }
            "remove" => {
                let index_str = ask_user("Enter index of notification to remove", "");
                if let Ok(index) = index_str.parse::<usize>() {
                    if event.remove_notification(index).is_none() {
                        println!("No notification found at index {}", index);
                    }
                } else {
                    println!("Invalid index. Please enter a number.");
                }
            }
            "edit" => {
                let index_str = ask_user("Enter index of notification to edit", "");
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < event.notification_settings.len() {
                        let notify_before_str = ask_user(
                            "Enter new notify before (in minutes)",
                            &event.notification_settings[index].notify_before.to_string(),
                        );
                        let method_str = ask_user(
                            "Enter new notification method (Email/SMS/Push)",
                            match event.notification_settings[index].method {
                                NotificationMethod::Email => "Email",
                                NotificationMethod::Sms => "SMS",
                                NotificationMethod::Push => "Push",
                            },
                        );

                        let method = match method_str.to_lowercase().as_str() {
                            "email" => NotificationMethod::Email,
                            "sms" => NotificationMethod::Sms,
                            "push" => NotificationMethod::Push,
                            _ => {
                                println!("Invalid method. Keeping the original value.");
                                event.notification_settings[index].method.clone()
                            }
                        };

                        let notify_before = notify_before_str
                            .parse::<i64>()
                            .unwrap_or(event.notification_settings[index].notify_before); // Keep original if parsing fails
                        event.notification_settings[index] = Notification {
                            notify_before,
                            method,
                            has_notified: false,
                        };
                    } else {
                        println!("No notification found at index {}", index);
                    }
                } else {
                    println!("Invalid index. Please enter a number.");
                }
            }
            "done" => break,
            _ => println!("Invalid action. Please enter add, remove, edit, or done."),
        }
    }
}
