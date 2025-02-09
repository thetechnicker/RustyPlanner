use crate::events::event::{ATTENDEE_FIELDS, EVENT_FIELDS, RECURRENCE_FIELDS};

pub fn print_help() {
    println!("Available commands:");
    println!("  add    - Add a new event");
    println!("  save   - Save events to file");
    println!("  remove - Remove an event by index");
    println!("  edit   - Edit an event by index");
    println!("  cls    - Clear the screen");
    println!("  list   - List all events");
    println!("  clear  - Clear all events");
    println!("  help   - Show this help message");
    println!("  exit   - Exit the application");
    println!();
    //println!("Use the 'add' command to create a new event with optional parameters for description, location, and notification time.");
    println!(
        "For more details on each command run `help [Command Name]`, refer to the documentation."
    );
}

pub fn print_add_help() {
    let mut help_message = "Usage: add event [OPTIONS]\n\n".to_string();
    help_message += "event:\n";
    for attribute in EVENT_FIELDS.iter() {
        let part_a = format!("\t{}:", attribute[0]);
        let part_b = format!("\t[{}]", attribute[1]);
        help_message += &format!("{:<20}{}\n", part_a, part_b);
        if attribute[0] == "recurrence" {
            help_message += "\t\tRecurrence Attributes:\n";
            for recurrence_attribute in RECURRENCE_FIELDS.iter() {
                let part_a = format!("\t{}:", recurrence_attribute[0]);
                let part_b = format!("\t[{}]", recurrence_attribute[1]);
                help_message += &format!("\t\t{:<20}{}\n", part_a, part_b);
            }
        }
        if attribute[0] == "attendees" {
            help_message += "\tAttendee Attributes:\n";
            for attendee_attributes in ATTENDEE_FIELDS.iter() {
                let part_a = format!("\t{}:", attendee_attributes[0]);
                let part_b = format!("\t[{}]", attendee_attributes[1]);
                help_message += &format!("\t\t{:<20}{}\n", part_a, part_b);
            }
        }
    }
    println!("{}", help_message);
}

pub fn print_save_help() {
    println!("  save           - Save events to file");
    println!("                  Usage: save <filename>");
    println!("                  Description: Saves all current events to the specified file.");
}

pub fn print_remove_help() {
    println!("  remove <index> - Remove an event by index");
    println!("                  Usage: remove <index>");
    println!("                  Description: Removes the event at the specified index from the list of events.");
}

pub fn print_edit_help() {
    println!("  edit <index>   - Edit an event by index");
    println!("                  Usage: edit <index> [options]");
    println!("                  Description: Edits the event at the specified index. Options can include");
    println!("                  mode, name, date, time, description, location, and alarm time.");
}

pub fn print_cls_help() {
    println!("  cls            - Clear the screen");
    println!("                  Description: Clears the console screen for better visibility.");
}

pub fn print_list_help() {
    println!("  list           - List all events");
    println!("                  Description: Displays all current events in the calendar.");
}

pub fn print_clear_help() {
    println!("  clear          - Clear all events");
    println!("                  Description: Removes all events from the calendar.");
}
