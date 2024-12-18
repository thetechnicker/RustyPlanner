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

        if trimmed.to_lowercase().starts_with("add") {
            if let Some(event) = parse_add(trimmed) {
                println!("{:?}", event);
            } else {
                println!("Failed to parse the event.");
            }
        }
    }
}

fn parse_add(add_str: &str) -> Option<Event> {
    let words = split_string_at_spaces(add_str);
    for word in words {
        println!("{}", word);
    }
    None
}

fn split_string_at_spaces(input: &str) -> Vec<String> {
    input
        .split_whitespace() // Split the string at whitespace
        .map(|s| s.to_string()) // Convert each &str to String
        .collect() // Collect the results into a Vec<String>
}
