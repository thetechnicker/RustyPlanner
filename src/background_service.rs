// A simple background service that runs in a loop and prints a statement

use std::time::Duration;
use std::thread;

fn main() {
    loop {
        println!("Background service is running...");
        thread::sleep(Duration::from_secs(5));
    }
}
