mod events;

use chrono::{Duration, NaiveDate, NaiveTime};
use directories::BaseDirs;
use events::event::Event;
use events::event_manager::{EventManager, EventManagerMode};
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
            match parse_add(command) {
                Some(event) => {
                    //println!("Event: {:?}", event);
                    let x = event_manager.lock().unwrap().add_event(event);
                    match event_manager.lock().unwrap().get_event(x as usize) {
                        Some(event) => {
                            println!("Event '{}' saved at index {}", event.name, x);
                        }
                        None => {
                            eprintln!("Error: Event not found at index {}", x);
                        }
                    }
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
                    let event = event_manager
                        .lock()
                        .unwrap()
                        .get_event(index)
                        .expect("Error")
                        // .clone();
                        .to_owned();
                    println!("Event '{}' edited at index {}", event.name, index);
                    event_manager
                        .lock()
                        .unwrap()
                        .replace_event(index, edit_event(&event));

                    println!(
                        "Event '{:?}' edited at index {}",
                        event_manager
                            .lock()
                            .unwrap()
                            .get_event(index)
                            .expect("error"),
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

fn edit_event(event: &Event) -> Event {
    // Ask the user for the new name
    let new_name = ask_user("Enter the new name", &event.name);
    let new_time = ask_user("Enter the new time", &event.time.to_string());
    let new_date = ask_user("Enter the new date", &event.date.to_string());
    let new_alarm_time = ask_user(
        "Enter the new alarm time",
        duration_to_string(event.allarm_time.unwrap_or(Duration::zero()).to_owned()).as_str(),
    );
    let new_description = ask_user(
        "Enter the new description",
        event.description.as_ref().unwrap_or(&"".to_string()),
    );
    let new_location = ask_user(
        "Enter the new location",
        event.location.as_ref().unwrap_or(&"".to_string()),
    );

    Event {
        name: new_name,
        time: is_valid_time(&new_time).unwrap_or(event.time),
        date: is_valid_date(&new_date).unwrap_or(event.date),
        has_notified: false,
        allarm_time: Some(parse_duration(&new_alarm_time).expect("Failed Parsing")),
        description: Some(new_description),
        location: Some(new_location),
    }
}

fn duration_to_string(duration: Duration) -> String {
    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let _seconds = seconds % 60;

    format!("{}h{}m", hours, minutes)
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

enum ParseMode {
    Desc,
    Loc,
    AlarmTime,
    None,
}

fn parse_add(input: &str) -> Option<Event> {
    let command = input.strip_prefix("add ").unwrap_or("");
    let parts: Vec<&str> = command.split_whitespace().collect();

    let mut name: String = String::from("");
    let mut time: Option<NaiveTime> = None;
    let mut date: Option<NaiveDate> = None;
    let mut location: String = String::from("");
    let mut description: String = String::from("");
    let mut allarm_time: Option<Duration> = None;

    let mut is_name = true;
    let mut mode = ParseMode::None;
    for part in parts {
        if date.is_none() {
            if let Some(_date) = is_valid_date(part) {
                date = Some(_date);
                is_name = false;
                continue;
            }
        }
        if time.is_none() {
            if let Some(_time) = is_valid_time(part) {
                time = Some(_time);
                is_name = false;
                continue;
            }
        }
        if is_name {
            name += part;
            name += " ";
        } else {
            match part {
                "-d" => {
                    mode = ParseMode::Desc;
                    continue;
                }
                "-l" => {
                    mode = ParseMode::Loc;
                    continue;
                }
                "-a" => {
                    mode = ParseMode::AlarmTime;
                    continue;
                }
                _ => {
                    //mode=ParseMode::None;
                }
            }

            match mode {
                ParseMode::Desc => {
                    description += part;
                    description += " ";
                }
                ParseMode::Loc => {
                    location += part;
                    location += " ";
                }
                ParseMode::AlarmTime => {
                    if allarm_time.is_none() {
                        allarm_time = Some(parse_duration(part).expect("Failed Parsing"));
                    }
                }
                ParseMode::None => {
                    //println!("idk where to put {}", part);
                }
            }
        }
    }

    if date.is_none() {
        eprintln!("Error: Date must be provided.");
        return None;
    }
    if time.is_none() {
        eprintln!("Error: Time must be provided.");
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
        allarm_time,
        description: Some(description.trim().to_owned()),
        location: Some(location.trim().to_owned()),
    };
    Some(event)
}

fn is_valid_date(date_str: &str) -> Option<NaiveDate> {
    let formats = ["%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%m/%d/%Y"];
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

fn clear_screen() {
    // ANSI escape code to clear the screen
    print!("{}[2J", 27 as char);
    // Move the cursor to the top left corner
    print!("{}[H", 27 as char);
    // Flush the output to ensure it is displayed
    io::stdout().flush().unwrap();
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let trimmed = s.trim();
    println!("{}", trimmed);

    // Regular expression to match hours and minutes
    let re =
        Regex::new(r"(?:(\d+)h)?(?:(\d+)m)?").map_err(|_| "Failed to compile regex".to_string())?;

    // Capture groups for hours and minutes
    let caps = re.captures(trimmed).ok_or("Invalid format".to_string())?;

    //println!("Captured groups: {:?}", caps);

    // Parse hours and minutes
    let hours = caps
        .get(1)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);
    let minutes = caps
        .get(2)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);

    // Create a Duration from the parsed values
    Ok(Duration::hours(hours) + Duration::minutes(minutes))
}
