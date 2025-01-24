use std::{fs, process::Command, thread, time::Duration};

// NEEDS TO BE REDONE IDK HOW TO TEST THIS

// #[test]
#[allow(dead_code)]
fn test_service_commands() {
    let _ = Command::new("target/debug/RustyPlanner")
        .arg("service")
        .arg("start")
        .spawn()
        .expect("Failed to start RustyPlanner Daemon");

    thread::sleep(Duration::from_secs(5));

    let pid = fs::read_to_string("/tmp/RustyPlannerDaemon.pid")
        .expect("Failed to read pid file")
        .trim()
        .parse::<i32>()
        .expect("Failed to parse pid");

    if let Ok(_) = Command::new("kill").arg("-0").arg(pid.to_string()).output() {
        assert!(true);
        println!("Process with PID {} exists.", pid);

        // Step 3: Optionally verify the process (e.g., check the command line)
        let process_info = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("comm=") // Get the command name
            .output()
            .unwrap();

        let command_name = String::from_utf8_lossy(&process_info.stdout)
            .trim()
            .to_string();

        assert_eq!(
            command_name, "target/debug/RustyPlanner_daemon",
            "not the process we are loocing for {}",
            command_name
        )
    } else {
        assert!(false, "service not running");
    }

    let _ = Command::new("target/debug/RustyPlanner")
        .arg("service")
        .arg("stop")
        .output()
        .expect("Failed to stop RustyPlanner Daemon");
}
