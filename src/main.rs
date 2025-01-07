mod events;

use events::EventManager;
use std::io::{self, Write};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;
use std::env;
//use notify_rust::Notification;



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

    let mut event_manager: EventManager;

    if let Some(dfp) = &data_file_path {
        event_manager = EventManager::new(dfp.clone(), true);
    } else {
        eprintln!("error cant create Config file");
        return;
    }

    event_manager.read_events_from_file();

    if args.len() > 1 {
         command_mode(&mut event_manager, &args[1..]);
    } else {
        loop_mode(&event_manager);
    }
}

fn loop_mode(event_manager: &mut EventManager){
    //let mut events: Vec<Event> = Vec::new();

    /*
       Notification::new()
       .summary("Firefox News")
       .body("This will almost look like a real firefox notification.")
       .icon("firefox")
       .show();
       */


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
        } else if trimmed.to_lowercase() == "clear" {
            event_manager.clear();
        } else if trimmed.to_lowercase() == "list" {
            event_manager.list_events();
        }

        if trimmed.to_lowercase().starts_with("add") {
            event_manager.add_event_from_str(trimmed);
        }
    }
}

fn command_mode(event_manager: &mut EventManager, commands: &[String]) {
    let command = commands.join(" ");
    
    if command.starts_with("add") {
        event_manager.add_event_from_str(&command);
    } else if command == "list" {
        event_manager.list_events();
    } else if command == "clear" {
        event_manager.clear();
    } else {
        eprintln!("Unknown command: {}", command);
    }
}
