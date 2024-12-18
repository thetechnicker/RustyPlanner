use std::io::{self, Write};

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
    }
}

