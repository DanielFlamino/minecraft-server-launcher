use chrono::prelude::*;
use chrono::Duration;
use std::fs::{self, File};
use std::io::{self, stdout, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

fn main() {
    println!("MINECRAFT SMART SERVER LAUNCHING THINGY\n");

    let mut one_hour_reminder_sent = false;
    let mut thirty_minutes_reminder_sent = false;
    let mut fifteen_minutes_reminder_sent = false;
    let mut five_minutes_reminder_sent = false;
    let mut one_minute_reminder_sent = false;
    let start_time = Local::now();
    let mut scheduled_time = start_time;

    // Get scheduled time
    loop {
        println!("\nInsert time for scheduled server shutdown");

        // Read hours for scheduled shutdown
        print!("Hours > ");
        let _ = stdout().flush();
        let mut scheduled_hours = String::new();
        let scheduled_hours = match io::stdin().read_line(&mut scheduled_hours) {
            Ok(_) => {
                let scheduled_hours = match scheduled_hours.trim().parse::<u32>() {
                    Ok(scheduled_hours) => {
                        if scheduled_hours > 23 {
                            println!("[WARN] Hours must be between 0 and 23");
                            continue;
                        };
                        scheduled_hours
                    }
                    Err(error) => {
                        println!("[WARN] Failed to parse number: {}", error);
                        continue;
                    }
                };
                scheduled_hours
            }
            Err(error) => {
                println!("[WARN] Failed to read input: {}", error);
                continue;
            }
        };

        // Read minutes for scheduled shutdown
        print!("Minutes > ");
        let _ = stdout().flush();
        let mut scheduled_minutes = String::new();
        let scheduled_minutes = match io::stdin().read_line(&mut scheduled_minutes) {
            Ok(_) => {
                let scheduled_minutes = match scheduled_minutes.trim().parse::<u32>() {
                    Ok(scheduled_minutes) => {
                        if scheduled_minutes > 59 {
                            println!("[WARN] Minutes must be between 0 and 59");
                            continue;
                        };
                        scheduled_minutes
                    }
                    Err(error) => {
                        println!("[WARN] Failed to parse number: {}", error);
                        continue;
                    }
                };
                scheduled_minutes
            }
            Err(error) => {
                println!("[WARN] Failed to read input: {}", error);
                continue;
            }
        };

        // Convert into datetime
        scheduled_time = scheduled_time
            .with_hour(scheduled_hours)
            .expect("")
            .with_minute(scheduled_minutes)
            .expect("")
            .with_second(0)
            .expect("")
            .with_nanosecond(0)
            .expect("");
        if scheduled_time <= start_time {
            scheduled_time = scheduled_time + Duration::days(1);
        }
        break;
    }
    println!("[INFO] Shutdown scheduled for {}", scheduled_time);

    // Check for server lock
    let server_lock_path = Path::new("./server.lock");
    match File::open(&server_lock_path) {
        Ok(mut file) => {
            // Server lock exists
            println!("[WARN] Found server.lock file!");
            println!("[WARN] Server is currently being run by another user or shutdown did not clear the lock file.");

            // Check lock file contents
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => println!("[WARN] server.lock contents: '{}'", contents),
                Err(error) => println!("[ERROR] Failed to read server.lock contents: '{}", error),
            }
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                // Server lock does not exist

                // Acquire server lock
                lock_server(server_lock_path);

                // Launch server process
                let mut process = match Command::new("java")
                    .args(&["-Xmx2048M", "-Xms1024M", "-jar", "server.jar"])
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    Ok(process) => process,
                    Err(error) => panic!("Running process error: {}", error),
                };

                loop {
                    // Check if server process has exited
                    match process.try_wait() {
                        Ok(Some(status)) => {
                            // Server process has already exited
                            println!("[INFO] Server process has already exited! ({})", status);
                            unlock_server(server_lock_path); //Release server lock
                            break;
                        }
                        Ok(None) => {
                            // Server process is still running

                            // Check current time
                            let now = Local::now();
                            if scheduled_time < now {
                                // Time's Up!
                                let message = "Time's Up!";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                // Wait a bit
                                thread::sleep(std::time::Duration::from_secs(5));
                                // Save server
                                write_to_child_process(
                                    process.stdin.as_mut().unwrap(),
                                    "save-all".to_string(),
                                );
                                // Wait a bit more
                                thread::sleep(std::time::Duration::from_secs(5));
                                // Stop server
                                write_to_child_process(
                                    process.stdin.as_mut().unwrap(),
                                    "stop".to_string(),
                                );
                                // Wait for server process to exit
                                match process.wait() {Ok(status) => println!("[INFO] Server process exited ({})", status),Err(error) => println!("[WARN] Error attempting to wait for server process to exit: {} ", error)};
                                // Release server lock
                                unlock_server(server_lock_path);
                                break;
                            } else if !one_minute_reminder_sent
                                && (scheduled_time - now) < Duration::minutes(1)
                            {
                                // One minute to go
                                let message = "Server closing in one minute!";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                one_minute_reminder_sent = true;
                            } else if !one_minute_reminder_sent
                                && !five_minutes_reminder_sent
                                && (scheduled_time - now) < Duration::minutes(5)
                            {
                                // Five minutes to go
                                let message = "Server closing in five minutes!";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                five_minutes_reminder_sent = true;
                            } else if !one_minute_reminder_sent
                                && !five_minutes_reminder_sent
                                && !fifteen_minutes_reminder_sent
                                && (scheduled_time - now) < Duration::minutes(15)
                            {
                                // Fifteen minutes to go
                                let message = "Server closing in fifteen minutes.";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                fifteen_minutes_reminder_sent = true;
                            } else if !one_minute_reminder_sent
                                && !five_minutes_reminder_sent
                                && !fifteen_minutes_reminder_sent
                                && !thirty_minutes_reminder_sent
                                && (scheduled_time - now) < Duration::minutes(30)
                            {
                                // Thirty minutes to go
                                let message = "Server closing in thirty minutes.";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                thirty_minutes_reminder_sent = true;
                            } else if !one_minute_reminder_sent
                                && !five_minutes_reminder_sent
                                && !fifteen_minutes_reminder_sent
                                && !thirty_minutes_reminder_sent
                                && !one_hour_reminder_sent
                                && (scheduled_time - now) < Duration::hours(1)
                            {
                                // One hour to go
                                let message = "Server closing in one hour.";
                                println!("[INFO] {}", message);
                                say_shutdown_reminder(
                                    process.stdin.as_mut().unwrap(),
                                    message,
                                    scheduled_time,
                                );
                                one_hour_reminder_sent = true;
                            };
                        }
                        Err(error) => println!(
                            "[WARN] Error attempting to wait for server process: {}",
                            error
                        ),
                    }

                    thread::sleep(std::time::Duration::from_secs(1)); // Sleep a bit before next check
                }
            }
            _ => panic!(error),
        },
    };
}

// Acquire server lock
fn lock_server(path: &Path) {
    // Create lock file
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(error) => panic!("Failed to create server.lock file: {}", error),
    };

    // Check whoami
    let whoami = Command::new("whoami")
        .output()
        .expect("Unable to call `whoami`");
    let whoami = String::from_utf8_lossy(&whoami.stdout);

    // Write whoami to lock file
    match file.write_all(whoami.trim().as_bytes()) {
        Ok(_) => println!("[INFO] server.lock file created"),
        Err(error) => panic!("Couldn't write to server.lock file: {}", error),
    }
}

// Release server lock
fn unlock_server(path: &Path) {
    match fs::remove_file(&path) {
        Ok(_) => println!("[INFO] server.lock file deleted"),
        Err(error) => panic!("Failed to delete server.lock file: {}", error),
    };
}

// Send shutdown reminder
fn say_shutdown_reminder(
    child_stdin: &mut std::process::ChildStdin,
    message: &str,
    timestamp: DateTime<Local>,
) {
    write_to_child_process(
        child_stdin,
        format!("tellraw @a {{\"text\":\"{}\",\"color\":\"#FBA800\",\"hoverEvent\":{{\"action\":\"show_text\",\"contents\":{{\"text\":\"Scheduled shutdown time: {}\"}}}}}}", message, timestamp)
    );
}

// Write input text to child process stdin
fn write_to_child_process(child_stdin: &mut std::process::ChildStdin, input: String) {
    let input = input.to_owned() + "\n";
    match child_stdin.write_all(input.as_bytes()) {
        Ok(result) => result,
        Err(_) => (),
    };
}
