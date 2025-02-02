mod events;
mod utils;

use chrono::Duration;
use events::event::{event_from_cmd, Event};
use events::event_manager::{EventManager, EventManagerMode};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use utils::{
    clear_screen, duration_to_string, get_path, is_valid_date, is_valid_time, parse_duration,
};

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

fn parse_commands(command: &str, event_manager: &Arc<Mutex<EventManager>>) {
    match command {
        _ if command.starts_with("add") => {
            match event_from_cmd(command) {
                Some(event) => {
                    //println!("Event: {:?}", event);
                    let x = event_manager.lock().unwrap().add_event(event);
                    match event_manager.lock().unwrap().get_event(x as usize) {
                        Some(event) => println!("Event '{}' saved at index {}", event.name, x),
                        _ => eprintln!("error"),
                    };

                    //event_manager.lock().unwrap().save_events();
                }
                None => {
                    eprintln!("error")
                }
            }
        }
        _ if command.starts_with("remove") => {
            let x: &str = command.strip_prefix("remove ").unwrap_or("");
            match x.trim().parse::<usize>() {
                Ok(index) => {
                    let e = event_manager.lock().unwrap().remove_event(index);
                    match e {
                        Some(event) => {
                            println!("Event '{}' removed from index {}", event.name, index);
                        }
                        None => {
                            eprintln!("Error: Event not found at index {}", index);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Invalid index: {}", x);
                }
            }
        }
        _ if command.starts_with("edit") => {
            let x: &str = command.strip_prefix("edit ").unwrap_or("");
            match x.trim().parse::<usize>() {
                Ok(index) => {
                    match event_manager.lock().unwrap().get_event_mut(index) {
                        Some(event) => {
                            println!("Event '{}' edited at index {}", event.name, index);
                            edit_event(event);
                        }
                        _ => eprintln!("error"),
                    };
                    println!(
                        "Event '{:?}' edited at index {}",
                        event_manager.lock().unwrap().get_event(index),
                        index
                    );
                }
                Err(_) => {
                    eprintln!("Invalid index: {}", x);
                }
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
        "help" => {
            print_help();
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

fn edit_event(event: &mut Event) {
    // Ask the user for the new name
    event.name = ask_user("Enter the new name", &event.name);

    // Ask for the new time and validate it
    let new_time = ask_user(
        "Enter the new time",
        &event
            .time
            .as_ref()
            .map_or("".to_string(), |t| t.to_string()),
    );
    event.time = is_valid_time(&new_time).or_else(|| event.time.clone());

    // Ask for the new date and validate it
    let new_date = ask_user(
        "Enter the new date",
        &event
            .date
            .as_ref()
            .map_or("".to_string(), |d| d.to_string()),
    );
    event.date = is_valid_date(&new_date).or_else(|| event.date.clone());

    // Ask for the new alarm time and parse it
    let new_alarm_time = ask_user(
        "Enter the new alarm time",
        &duration_to_string(event.alarm_time.unwrap_or(Duration::zero())).as_str(),
    );
    event.alarm_time = parse_duration(&new_alarm_time).ok();

    // Ask for the new description
    event.description = Some(ask_user(
        "Enter the new description",
        event.description.as_ref().unwrap_or(&"".to_string()),
    ));

    // Ask for the new location
    event.location = Some(ask_user(
        "Enter the new location",
        event.location.as_ref().unwrap_or(&"".to_string()),
    ));
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
    println!("  add <name> <time> <date> [-d <description>] [-l <location>] [-a <time to notify before event>]  - Add a new event");
    println!("  save                                                                                            - Save events to file");
    println!("  remove <index>                                                                                  - Remove an event by index");
    println!("  edit <index>                                                                                    - Edit an event by index");
    println!("  cls                                                                                             - Clear the screen");
    println!("  list                                                                                            - List all events");
    println!("  clear                                                                                           - Clear all events");
    println!("  help                                                                                            - Show this help message");
    println!("  exit                                                                                            - Exit the application");
    println!();
    println!("Use the 'add' command to create a new event with optional parameters for description, location, and notification time.");
    println!("For more details on each command, refer to the documentation.");
}
