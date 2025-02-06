mod events;
mod miscs;

use chrono::DateTime;
use chrono::Local;
use events::event::Attendee;
use events::event::Event;
use events::event::Notification;
use events::event::NotificationMethod;
use miscs::arg_parsing::parse_data;
// use arg_parsing::parse_kwargs;
use events::event_manager::{EventManager, EventManagerMode};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
// use utils::parse_stupid_recursive;
use miscs::utils::{clear_screen, get_path};

fn main() {
    let args: Vec<String> = env::args().collect();

    let data_file_path = get_path();

    let event_manager: Arc<Mutex<EventManager>>;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), false, EventManagerMode::Active);
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
}

fn service_start() {
    // Check if the binary is built locally or installed globally/for user
    #[cfg(debug_assertions)]
    {
        println!("Running local build");
        let mut _child = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("RustyPlanner_daemon")
            //.stdout(Stdio::null()) // Redirect standard output to null
            //.stderr(Stdio::null()) // Redirect standard error to null
            .spawn()
            .expect("Failed to start background service");
        _child.wait().expect("Failed to wait on child process");
        println!("Service started");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("Running installed version");
        let _child = Command::new("RustyPlanner_daemon")
            // .stdout(Stdio::null()) // Redirect standard output to null
            // .stderr(Stdio::null()) // Redirect standard error to null
            .spawn()
            .expect("Failed to start background service");
    }
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
        _ if command.starts_with("stupid") => {
            let input = command.strip_prefix("stupid").unwrap_or(command).trim();
            let data = parse_data(input, 0);
            data.print(0);
        }
        _ if command.starts_with("add") => {
            let input = command.strip_prefix("add").unwrap_or("").trim();
            match input {
                _ if input.starts_with("event") => {
                    let string = input.strip_prefix("event").unwrap_or(command).trim();
                    add_event_loop(string, event_manager);
                }
                _ if input.starts_with("notification") => {
                    let input = input.strip_prefix("notification").unwrap_or(command).trim();
                    add_notification_loop(input, event_manager);
                }
                _ if input.starts_with("attendance") => {
                    let input = input.strip_prefix("attendance").unwrap_or(command).trim();
                    add_attendance_loop(input, event_manager);
                }
                _ => print_add_help(),
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
                "" => print_help(), // Default help message
                _ => print_help(),  // Fallback for unrecognized commands
            }
        }
        "save" => {
            event_manager.lock().unwrap().save_events();
        }
        "list" => {
            event_manager.lock().unwrap().list_events();
        }
        "clear" => {
            event_manager.lock().unwrap().clear();
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

fn add_attendance_loop(_input: &str, _event_manager: &Mutex<EventManager>) {
    todo!()
}

fn add_notification_loop(_input: &str, _event_manager: &Mutex<EventManager>) {
    todo!()
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

fn print_help() {
    println!("Available commands:");
    println!("  add            - Add a new event");
    println!("  save           - Save events to file");
    println!("  remove <index> - Remove an event by index");
    println!("  edit <index>   - Edit an event by index");
    println!("  cls            - Clear the screen");
    println!("  list           - List all events");
    println!("  clear          - Clear all events");
    println!("  help           - Show this help message");
    println!("  exit           - Exit the application");
    println!();
    println!("Use the 'add' command to create a new event with optional parameters for description, location, and notification time.");
    println!("For more details on each command, refer to the documentation.");
}

fn print_add_help() {
    let help_message = r#"
Usage: add [event|notification|attendance] [OPTIONS]

event:
    event_id =      [EVENT_ID]
    title =         [TITLE]
    description =   [DESCRIPTION]
    start_time =    [START_TIME] (format: RFC3339)
    end_time =      [END_TIME] (format: RFC3339)
    location =      [LOCATION]
    is_recurring =  [true|false]
    recurrence =    [RECURRING_DETAILS] (if applicable)

notification:
    event_id =      [EVENT_ID]
    time_before =   [TIME_BEFORE] (e.g., "1 hour before")

attendance:
    event_id =      [EVENT_ID]
    attendees =     [ATTENDEE_LIST] (comma-separated list of attendee names)

"#;

    println!("{}", help_message);
}

fn print_save_help() {
    println!("  save           - Save events to file");
    println!("                  Usage: save <filename>");
    println!("                  Description: Saves all current events to the specified file.");
}

fn print_remove_help() {
    println!("  remove <index> - Remove an event by index");
    println!("                  Usage: remove <index>");
    println!("                  Description: Removes the event at the specified index from the list of events.");
}

fn print_edit_help() {
    println!("  edit <index>   - Edit an event by index");
    println!("                  Usage: edit <index> [options]");
    println!("                  Description: Edits the event at the specified index. Options can include");
    println!("                  mode, name, date, time, description, location, and alarm time.");
}

fn print_cls_help() {
    println!("  cls            - Clear the screen");
    println!("                  Description: Clears the console screen for better visibility.");
}

fn print_list_help() {
    println!("  list           - List all events");
    println!("                  Description: Displays all current events in the calendar.");
}

fn print_clear_help() {
    println!("  clear          - Clear all events");
    println!("                  Description: Removes all events from the calendar.");
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
    let new_start_time_str = ask_user(
        "Enter new start time (RFC3339 format)",
        &event.start_time.to_rfc3339(),
    );
    if let Ok(new_start_time) = DateTime::parse_from_rfc3339(&new_start_time_str) {
        event.update_start_time(new_start_time.with_timezone(&Local));
    } else {
        println!("Invalid start time format. Keeping the original value.");
    }

    // Update end time
    let new_end_time_str = ask_user(
        "Enter new end time (RFC3339 format)",
        &event.end_time.to_rfc3339(),
    );
    if let Ok(new_end_time) = DateTime::parse_from_rfc3339(&new_end_time_str) {
        event.update_end_time(new_end_time.with_timezone(&Local));
    } else {
        println!("Invalid end time format. Keeping the original value.");
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
