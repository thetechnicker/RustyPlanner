use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
#[cfg(not(test))]
use directories::BaseDirs;
use regex::Regex;
#[cfg(not(test))]
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const DATE_FORMATS: [&str; 4] = ["%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%m/%d/%Y"];
const TIME_FORMATS: [&str; 3] = ["%H:%M:%S", "%H:%M", "%I:%M %p"];

#[cfg(test)]
pub fn get_path() -> std::option::Option<PathBuf> {
    let tmp_dir = std::env::temp_dir();
    Some(tmp_dir)
}

#[cfg(not(test))]
pub fn get_path() -> std::option::Option<PathBuf> {
    let data_file_path: Option<PathBuf>;

    if let Some(base_dirs) = BaseDirs::new() {
        let data_base_dir = base_dirs.data_dir();

        println!("Data Directory: {:?}", data_base_dir);

        let data_dir = data_base_dir.join("RustyPlanner");

        fs::create_dir_all(data_dir.clone()).expect("Failed to create data directory");

        data_file_path = Some(data_dir);
    } else {
        eprintln!("Could not find base directories.");
        data_file_path = None;
    }

    data_file_path
}

#[allow(dead_code)]
pub fn duration_to_string(duration: &Duration) -> String {
    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let _seconds = seconds % 60;

    format!("{}h{}m", hours, minutes)
}

#[derive(Debug)]
pub enum AAAA {
    Date(Result<NaiveDate, chrono::ParseError>),
    Time(Result<NaiveTime, chrono::ParseError>),
    DateTime(Result<NaiveDateTime, chrono::ParseError>),
}

// TODO:
pub fn datetime_from_str(datetime_str: &str) -> Result<NaiveDateTime, Vec<(AAAA, String)>> {
    let mut format_vec: Vec<String> = Vec::new();
    for df in DATE_FORMATS {
        for tf in TIME_FORMATS {
            format_vec.push(format!("{} {}", df, tf));
            format_vec.push(format!("{} {}", tf, df));
        }
    }
    let mut err: Vec<(AAAA, String)> = Vec::new();
    for fmt in format_vec {
        let x: Result<NaiveDateTime, chrono::ParseError> =
            NaiveDateTime::parse_from_str(datetime_str, &fmt);
        if let Ok(dt) = x {
            return Ok(dt);
        }
        err.push((AAAA::DateTime(x), String::from(fmt)));
    }
    for fmt in DATE_FORMATS {
        let x = NaiveDate::parse_from_str(datetime_str, &fmt);
        if let Ok(dt) = x {
            return Ok(dt.and_time(Local::now().naive_utc().time()));
        }
        err.push((AAAA::Date(x), String::from(fmt)));
    }
    for fmt in TIME_FORMATS {
        let x = NaiveTime::parse_from_str(datetime_str, &fmt);
        if let Ok(dt) = x {
            return Ok(Local::now().naive_utc().date().and_time(dt));
        }
        err.push((AAAA::Time(x), String::from(fmt)));
    }
    Err(err)
}

pub fn date_from_str(date_str: &str) -> NaiveDate {
    for format in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return date;
        }
    }
    Local::now().naive_utc().date()
}

pub fn time_from_str(time_str: &str) -> NaiveTime {
    for format in DATE_FORMATS {
        if let Ok(time) = NaiveTime::parse_from_str(time_str, format) {
            println!("{}", time.format("%H:%M:%S"));
            return time - *Local::now().offset();
        }
    }
    Local::now().naive_utc().time()
}

pub fn clear_screen() {
    // ANSI escape code to clear the screen
    print!("{}[2J", 27 as char);
    // Move the cursor to the top left corner
    print!("{}[H", 27 as char);
    // Flush the output to ensure it is displayed
    io::stdout().flush().unwrap();
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let trimmed = s.trim();
    // println!("{}", trimmed);

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
