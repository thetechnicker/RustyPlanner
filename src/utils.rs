use chrono::{Duration, NaiveDate, NaiveTime};
#[cfg(not(test))]
use directories::BaseDirs;
use regex::Regex;
use std::collections::HashMap;
#[cfg(not(test))]
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[cfg(test)]
pub fn get_path() -> std::option::Option<PathBuf> {
    let tmp_dir = std::env::temp_dir();
    Some(tmp_dir.join("dates.json"))
}

#[cfg(not(test))]
pub fn get_path() -> std::option::Option<PathBuf> {
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

    return data_file_path;
}

#[allow(dead_code)]
pub fn duration_to_string(duration: &Duration) -> String {
    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let _seconds = seconds % 60;

    format!("{}h{}m", hours, minutes)
}

#[allow(dead_code)]
pub fn date_from_str(date_str: &str) -> Option<NaiveDate> {
    let formats = ["%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%m/%d/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(date);
        }
    }
    None
}

#[allow(dead_code)]
pub fn time_from_str(time_str: &str) -> Option<NaiveTime> {
    let formats = ["%H:%M:%S", "%H:%M", "%I:%M %p"];
    for format in &formats {
        if let Ok(time) = NaiveTime::parse_from_str(time_str, format) {
            return Some(time);
        }
    }
    None
}

#[allow(dead_code)]
pub fn clear_screen() {
    // ANSI escape code to clear the screen
    print!("{}[2J", 27 as char);
    // Move the cursor to the top left corner
    print!("{}[H", 27 as char);
    // Flush the output to ensure it is displayed
    io::stdout().flush().unwrap();
}

#[allow(dead_code)]
pub fn parse_duration(s: &str) -> Result<Duration, String> {
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

pub fn parse_args(input: &str) -> Result<(Vec<String>, HashMap<String, String>), String> {
    let args: Vec<&str> = input.split(',').collect();
    let mut positional_args = Vec::new();
    let mut keyword_args = HashMap::new();
    let mut found_keyword = false; // Flag to track if a keyword argument has been found

    for arg in args {
        let arg = arg.trim(); // Remove any leading/trailing whitespace
        if arg.contains('=') {
            // If we find a keyword argument, set the flag
            found_keyword = true;
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid keyword argument: {}", arg));
            }
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            keyword_args.insert(key, value);
        } else {
            if found_keyword {
                return Err("Positional arguments cannot follow keyword arguments.".to_string());
            }
            positional_args.push(arg.to_string());
        }
    }

    Ok((positional_args, keyword_args))
}
