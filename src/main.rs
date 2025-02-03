mod events;
mod utils;

use events::event::Event;
use events::event_manager::{EventManager, EventManagerMode};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use utils::{clear_screen, get_path, parse_args};

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

#[allow(unused_mut)]
fn parse_commands(command: &str, event_manager: &Arc<Mutex<EventManager>>) {
    match command {
        _ if command.starts_with("add") => {
            let mut input = command.strip_prefix("add ").unwrap_or(command);
            match parse_args(input) {
                Ok((positional, keywords)) => {
                    println!("Positional Arguments: {:?}", positional);
                    println!("Keyword Arguments: {:?}", keywords);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            // let mut event = Event::default().set_title(input.to_string());
            // println!("{}", event)
        }
        _ if command.starts_with("add_old") => {
            // let mut event = Event::from_str(command);
            let mut event = Event::default();
            println!("{}", event);
            let mut attempts = 0; // Counter for invalid attempts

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
                        // Keep the event
                        event_manager.lock().unwrap().add_event(event);
                        break; // Exit the loop
                    }
                    "2" => {
                        // Discard the event
                        println!("Event has been discarded.");
                        break; // Exit the loop
                    }
                    "3" => {
                        // Edit the event
                        // edit_event(&mut event);
                        println!("Event has been edited. Here is the updated event:");
                        println!("{}", event);
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
        // _ if command.starts_with("remove") => {
        //     let x: &str = command.strip_prefix("remove ").unwrap_or("");
        //     match x.trim().parse::<usize>() {
        //         Ok(index) => {
        //             let e = event_manager.lock().unwrap().remove_event(index);
        //             match e {
        //                 Some(event) => {
        //                     println!("Event '{}' removed from index {}", event.name, index);
        //                 }
        //                 None => {
        //                     eprintln!("Error: Event not found at index {}", index);
        //                 }
        //             }
        //         }
        //         Err(_) => {
        //             eprintln!("Invalid index: {}", x);
        //         }
        //     }
        // }
        // _ if command.starts_with("edit") => {
        //     let x: &str = command.strip_prefix("edit ").unwrap_or("");
        //     match x.trim().parse::<usize>() {
        //         Ok(index) => {
        //             match event_manager.lock().unwrap().get_event_mut(index) {
        //                 Some(event) => {
        //                     println!("Event '{}' edited at index {}", event.name, index);
        //                     edit_event(event);
        //                 }
        //                 _ => eprintln!("error"),
        //             };
        //             println!(
        //                 "Event '{:?}' edited at index {}",
        //                 event_manager.lock().unwrap().get_event(index),
        //                 index
        //             );
        //         }
        //         Err(_) => {
        //             eprintln!("Invalid index: {}", x);
        //         }
        //     }
        // }
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

// fn edit_event(event: &mut Event) {
//     // Ask the user for the new name
//     event.name = ask_user("Enter the new name", &event.name);

//     // Ask for the new time and validate it
//     let new_time = ask_user(
//         "Enter the new time",
//         &event
//             .time
//             .as_ref()
//             .map_or("".to_string(), |t| t.to_string()),
//     );
//     event.time = time_from_str(&new_time).or_else(|| event.time.clone());

//     // Ask for the new date and validate it
//     let new_date = ask_user(
//         "Enter the new date",
//         &event
//             .date
//             .as_ref()
//             .map_or("".to_string(), |d| d.to_string()),
//     );
//     event.date = date_from_str(&new_date).or_else(|| event.date.clone());

//     // Ask for the new alarm time and parse it
//     let new_alarm_time = ask_user(
//         "Enter the new alarm time",
//         &duration_to_string(&event.alarm_time.unwrap_or(Duration::zero())).as_str(),
//     );
//     event.alarm_time = parse_duration(&new_alarm_time).ok();

//     // Ask for the new description
//     event.description = Some(ask_user(
//         "Enter the new description",
//         event.description.as_ref().unwrap_or(&"".to_string()),
//     ));

//     // Ask for the new location
//     event.location = Some(ask_user(
//         "Enter the new location",
//         event.location.as_ref().unwrap_or(&"".to_string()),
//     ));
// }

#[allow(unused_mut)]
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
Usage: event_manager add [OPTIONS]

Add a new event to the calendar.

Options:
    mode: <MODE>,           Specify the event type. Valid options are:
                            one-time, recurring.

    name: <NAME>,           The name of the event. Default is "New Event".

    date: <DATE>,           The date of the event in YYYY-MM-DD format.
                            Required for one-time events.

    weekday: <WEEKDAY>,     The day of the week for recurring events.
                            Valid options are: Monday, Tuesday, ..., Sunday.

    time: <TIME>,           The time of the event in HH:MM format.
                            Required for both one-time and recurring events.

    description: <DESC>,    A brief description of the event.

    location: <LOCATION>,    The location where the event will take place.

    alarm time: <DURATION>, Set an alarm for the event. Specify duration in
                            hours, minutes, or seconds (e.g., "10m", "1h30m").

Examples:
    event_manager add mode: one-time, name: "Doctor Appointment", date: "2023-10-15", time: "14:30", description: "Annual check-up", location: "Clinic", alarm time: "30m"
    
    event_manager add mode: recurring, name: "Weekly Meeting", weekday: "Monday", time: "09:00", description: "Team sync-up", location: "Office", alarm time: "10m"

Note: If no mode is specified, the default is one-time. If no date is provided for a one-time event, the event will not be created. For recurring events, a weekday must be specified.
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
