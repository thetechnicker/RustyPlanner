# RustyPlanner

RustyPlanner is a command-line tool for managing your appointments and events, serving as a personal appointment calendar. It allows you to add, list, and clear events stored in a JSON file.

## Features

- Add new events
- Remove event by Index
- List all events
- Clear all events
- Interactive and command-line modes

## Usage

### Interactive Mode

Run the application without any arguments to enter the interactive mode:

```sh
cargo run
```

### Command-Line Mode

You can also use the application directly from the command line with the following commands:

- **Add an event**: `cargo run add <event name> <time> [<date>]`
- **List all events**: `cargo run list`
- **Clear all events**: `cargo run clear`
- **Show help**: `cargo run help`

### Available Commands

- `add <event name> <time> <date> [-d <description>] [-l <location>] [-a <time before event to notify>]` - Add a new event
- `remove <index>` - remove event
- `edit <index>` - edit event
- `cls` - Clear the screen
- `list` - List all events
- `clear` - Clear all events
- `help` - Show this help message
- `exit` - Exit the application (interactive mode only)

## Dependencies

RustyPlanner depends on the following crates:

- `chrono`
- `directories`
- `notify-rust`
- `regex`
- `serde` and `serde_json`
- `notify`
- `futures`
- `daemonize`
- `users`
- `signal-hook`

## Installation

To install RustyPlanner, you need to have Rust installed. You can then build and run the application using Cargo:

```sh
git clone https://github.com/thetechnicker/RustyPlanner.git
cd RustyPlanner
cargo build
cargo run
```

## Contributing

If you would like to contribute to RustyPlanner, please fork the repository and submit a pull request.

## License

This project is licensed under the MIT License.
