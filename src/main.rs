mod events;

use events::{EventManager, EventManagerMode};
use std::io::{self, Write};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;
use std::env;
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

fn service_start(){
    let _child = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("background_service")
        .arg(if cfg!(debug_assertions) { "" } else { "--release" })
        .spawn()
        .expect("Failed to start background service");
}

fn service_stop(){
    let build_type = if cfg!(debug_assertions) { "debug" } else { "release" };
    let service_name = format!("target/{}/background_service", build_type);
    let _output = Command::new("pkill")
        .arg("-f")
        .arg(service_name)
        .output()
        .expect("Failed to stop background service");
    println!("Service stopped, output: {:?}", _output);
}

fn service_restart(){
    service_stop();
    service_start();
}

fn loop_mode(event_manager: &Arc<Mutex<EventManager>>){
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
            event_manager.lock().unwrap().add_event_from_str(command);
        }
        _ if command.starts_with("save") => {
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

