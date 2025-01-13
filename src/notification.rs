use notify_rust::Notification;
use std::process::Command;

pub fn send_notification(title: &str, message: &str) {
    if is_wsl() {
        println!("Sending WSL notification");
        send_wsl_notification(title, message);
    } else {
        println!("Sending generic notification");
        send_generic_notification(title, message);
    }
}

fn is_wsl() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/version") {
            println!("Contents: {}", contents);
            return contents.contains("WSL");
        }
    }
    false
}

fn send_generic_notification(title: &str, message: &str) {
    Notification::new()
        .summary(title)
        .body(message)
        .show()
        .unwrap();
}

fn send_wsl_notification(title: &str, message: &str) {
    let powershell_command = format!(
        "powershell.exe -Command \"Import-Module BurntToast; New-BurntToastNotification -Text '{}', '{}'\"",
        title, message
    );

    Command::new("sh")
        .arg("-c")
        .arg(powershell_command)
        .output()
        .expect("Failed to send WSL notification");
}
