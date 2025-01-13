mod events;

use chrono::{Date, NaiveDate, NaiveTime};
use directories::BaseDirs;
use events::Event;
use events::{EventManager, EventManagerMode};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = env::args().collect();

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
        let _child = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("RustyPlanner_background_service")
            .spawn()
            .expect("Failed to start background service");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("Running installed version");
        let _child = Command::new("RustyPlanner_background_service")
            .spawn()
            .expect("Failed to start background service");
    }
}

fn service_stop() {
    #[cfg(debug_assertions)]
    {
        let service_name = "target/debug/RustyPlanner_background_service";
        let _output = Command::new("pkill")
            .arg("-f")
            .arg(service_name)
            .output()
            .expect("Failed to stop background service");
        println!("Service stopped, output: {:?}", _output);
    }
    #[cfg(not(debug_assertions))]
    {
        let service_name = "RustyPlanner_background_service";
        let _output = Command::new("pkill")
            .arg("-f")
            .arg(service_name)
            .output()
            .expect("Failed to stop background service");
        println!("Service stopped, output: {:?}", _output);
    }
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
            parse_add(command);
            // let x = event_manager.lock().unwrap().add_event_from_str(command);
            // match event_manager.lock().unwrap().get_event(x) {
            //     Some(event) => {
            //         println!("Event '{}' saved at index {}", event.name, x);
            //     }
            //     None => {
            //         eprintln!("Error: Event not found at index {}", x);
            //     }
            // }
        }
        _ if command.starts_with("save") => {
            event_manager.lock().unwrap().save_events();
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
        "list" => {
            event_manager.lock().unwrap().list_events();
        }
        "clear" => {
            event_manager.lock().unwrap().clear();
        }
        "help" => {
            print_help();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help(); // Suggest help for valid commands
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  add <event details> - Add a new event");
    println!("  list                - List all events");
    println!("  clear               - Clear all events");
    println!("  help                - Show this help message");
    println!("  exit                - Exit the application");
}

fn parse_add(input: &str) -> Option<Event> {
    let command = input.strip_prefix("add ").unwrap_or("");
    let parts: Vec<&str> = command.split_whitespace().collect();

    let mut name: String = String::from("");
    let mut time: Option<NaiveTime> = None;
    let mut date: Option<NaiveDate> = None;

    let mut is_name = false;
    for part in parts {
        if is_name {
            name += part;
            name += " ";
        }
        if let Some(_date) = is_valid_date(part) {
            date = Some(_date);
            is_name = true;
        }
        if let Some(_time) = is_valid_time(part) {
            time = Some(_time);
            is_name = true;
        }
    }

    if date.is_none() || time.is_none() {
        eprintln!("Error: Date and time must be provided.");
        return None;
    }
    if is_name {
        eprintln!("Error: Name not Defined");
        return None;
    }

    name = name.trim().to_owned();

    let event = Event {
        name,
        time: time.unwrap(),
        date: date.unwrap(),
        has_notified: false,
        allarm_time: None,
        description: None,
        location: None,
    };
    Some(event)
}

fn is_valid_date(date_str: &str) -> Option<NaiveDate> {
    let formats = ["%Y-%m-%d", "%d-%m-%Y", "%d.&m.&Y", "%m/%d/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(date);
        }
    }
    None
}

fn is_valid_time(time_str: &str) -> Option<NaiveTime> {
    let formats = ["%H:%M:%S", "%H:%M", "%I:%M %p"];
    for format in &formats {
        if let Ok(time) = NaiveTime::parse_from_str(time_str, format) {
            return Some(time);
        }
    }
    None
}
