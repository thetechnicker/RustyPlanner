use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug)]
struct Event {
    timedate: String, // You can use a more complex type for timedate if needed
    name: String,
}

fn main() {
    let mut input = String::new();
    loop {
        // Read from standard input
        print!("Please enter some input: ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();

        println!("You entered: {}", trimmed);

        if trimmed.to_lowercase().contains("exit") {
            return;
        }

        if let Some(event) = parse_event(input) {
            println!("{:?}", event);
        } else {
            println!("Failed to parse the event.");
        }
    }
}

fn parse_event(input: &str) -> Option<Event> {
    // Check if the input starts with "add "
    if !input.starts_with("add ") {
        return None;
    }

    // Remove the "add " prefix
    let trimmed = &input[4..];

    // Split the string by brackets
    let parts: Vec<&str> = trimmed.split('[').collect();

    if parts.len() < 3 {
        return None; // Not enough parts to parse
    }

    // Extract timedate and name
    let timedate = parts[1].trim_end_matches(']').trim().to_string();
    let name = parts[2].trim_end_matches(']').trim().to_string();

    Some(Event { timedate, name })
}
