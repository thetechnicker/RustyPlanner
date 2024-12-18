use std::io;
use std::env;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    println!("Command-line arguments:");
    for arg in args {
        println!("{}", arg);
    }

    // Read from standard input
    let mut input = String::new();
    println!("Please enter some input:");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("You entered: {}", input.trim());

}

