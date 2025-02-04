mod events;
mod miscs;

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
        event_manager = EventManager::new(dfp.clone(), true, EventManagerMode::Active);
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
            parse_commands(&trimmed, event_manager);
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
            let input = command.strip_prefix("stupid ").unwrap_or(command);
            let data = parse_data(input, 0);
            data.print(0);
        }
        _ if command.starts_with("add") => {
            let input = command.strip_prefix("add ").unwrap_or(command);
            match input {
                _ if input.starts_with("event") => {
                    let input = command.strip_prefix("event ").unwrap_or(command);
                    add_event_loop(input, event_manager);
                }
                _ if input.starts_with("notification") => {
                    let input = command.strip_prefix("notification ").unwrap_or(command);
                    add_notification_loop(input, event_manager);
                }
                _ if input.starts_with("attendance") => {
                    let input = command.strip_prefix("attendance ").unwrap_or(command);
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
    let index = event_manager.lock().unwrap().add_event_from_str(&input);
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
                break; // Exit the loop
            }
            "2" => {
                // Discard the event
                println!("Event has been discarded.");
                break; // Exit the loop
            }
            "3" => {
                // attempts = 0; // Reset attempts after a successful edit
                todo!();
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

#[allow(dead_code)]
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
