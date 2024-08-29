

use std::{io::Write, sync::{Arc, Condvar, Mutex}, thread};
use std::process::Command;

use rdev::{/*display_size,*/ listen, Event, EventType, Key};
use sysinfo::System;

use crate::ui::open_gui;
use crate::utils::{get_value_condvar_tuple, set_and_wait_condvar_tuple};
use crate::settings::Settings;

fn log_cpu_usage(file: &mut std::fs::File, cpu_usage: f32) {
    // Log the CPU usage with a timestamp
    let log_entry = format!("CPU usage: {:.2}%\n", cpu_usage);
    file.write_all(log_entry.as_bytes()).expect("Failed to write to log file");
    //println!("{}", log_entry);
}

fn is_process_already_running(process_name: &str) -> bool {
    let mut p_name = process_name.to_string();
    #[cfg(target_os = "windows")]
    if !p_name.ends_with(".exe") { p_name.push_str(".exe"); }
    #[cfg(target_os = "windows")]
    let output = Command::new("tasklist")
        .arg("/FI")
        .arg(format!("IMAGENAME eq {}", p_name))
        .output()
        .expect("Failed to execute tasklist command");
    #[cfg(target_os = "linux")]
    let output = Command::new("pgrep")
        .arg(p_name)
        .output()
        .expect("Failed to execute pgrep command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let occurrencies_count = output_str.split("\n").filter(|s| s.contains(&p_name)).count();
    //println!("{:?}", occurrencies_count);
    occurrencies_count > 1
}

fn listen_for_keys(gui_opened: Arc<(Mutex<bool>, Condvar)>, backup_running: Arc<(Mutex<bool>, Condvar)>) {
    // Initialize state for tracking key combinations
    let mut ctrl_pressed = false;
    let mut shift_pressed = false;
    let mut alt_pressed = false;

    if let Err(error) = listen(move |event: Event| {
        // This callback will be periodically executed for the whole duration of the program
        match event.event_type {
            EventType::KeyPress(key) => match key {
                Key::ControlLeft | Key::ControlRight => {
                    ctrl_pressed = true;
                }
                Key::ShiftLeft | Key::ShiftRight => {
                    shift_pressed = true;
                }
                Key::Alt => {
                    alt_pressed = true;
                }
                Key::KeyG => {
                    if ctrl_pressed && shift_pressed && alt_pressed {
                        // Read condvar value (GUI must be closed for this command to work!)
                        if !get_value_condvar_tuple(gui_opened.clone()) {
                            // CTRL + SHIFT + ALT + G detected, notify main thread to reopen GUI
                            println!("CTRL + SHIFT + ALT + G detected!");
                            set_and_wait_condvar_tuple(gui_opened.clone(), Some(true), None);
                        } else {
                            println!("GUI is already opened");
                        }
                    }
                }
                Key::KeyB => {
                    if ctrl_pressed && shift_pressed && alt_pressed {
                        // Read condvar value (backup must not be running for this command to work!)
                        if !get_value_condvar_tuple(backup_running.clone()) && !get_value_condvar_tuple(gui_opened.clone()) {
                            // CTRL + SHIFT + ALT + B detected, notify main thread to reopen GUI and start backup
                            println!("CTRL + SHIFT + ALT + B detected!");

                            set_and_wait_condvar_tuple(backup_running.clone(), Some(true), None);
                            set_and_wait_condvar_tuple(gui_opened.clone(), Some(true), None); 
                            
                        } else {
                            println!("GUI is already opened or backup is already running.");
                        }
                    }
                }
                _ => {}
            },
            EventType::KeyRelease(key) => match key {
                Key::ControlLeft | Key::ControlRight => {
                    ctrl_pressed = false;
                }
                Key::ShiftLeft | Key::ShiftRight => {
                    shift_pressed = false;
                }
                Key::Alt => {
                    alt_pressed = false;
                }
                _ => {}
            },
            _ => {}
        }
    }) {
        println!("KeyListenerError: {:?}", error);
    }
}
/*
fn listen_for_mouse_drag(listening: Arc<Mutex<bool>>, width: u32, height: u32) {
    // Define the screen corners
    let corners = [
        (0, 0), // Top-left
        (width, 0), // Top-right (replace with your screen width)
        (width, height), // Bottom-right (replace with your screen width and height)
        (0, height), // Bottom-left (replace with your screen height)
    ];

    // Track the state of dragging
    let mut corner_index = 0;

    if let Err(error) = listen(move |event: Event| {
        if let EventType::MouseMove { x, y } = event.event_type {
            let pos = (x as u32, y as u32);

            // Check if the mouse is in the next corner
            if pos == corners[corner_index] {
                println!("Mouse reached corner: {:?}", pos);
                corner_index += 1;

                // If all corners have been visited
                if corner_index == corners.len() {
                    println!("All corners visited in sequence!");
                    corner_index = 0; // Reset to detect the next sequence
                }
            }
        }

        // Stop listening if necessary
        if !*listening.lock().unwrap() {
            return;
        }
    }) {
        println!("Error: {:?}", error);
    }
}
*/
pub fn start_daemon(settings: Settings) {
    let settings_mutex = Arc::new(Mutex::new(settings));
    if is_process_already_running("rust_backup") {
        println!("This process is already running. Cannot launch more than one instance.");
        return;
    }
    // Condvar for checking if the GUI is already opened, used by main thread and keyboard thread
    let gui_opened = Arc::new((Mutex::new(true), Condvar::new()));
    let gui_opened_clone = Arc::clone(&gui_opened);
    // Condvar for checking if the backup is running, used by GUI and keyboard thread
    let backup_running = Arc::new((Mutex::new(false), Condvar::new()));
    let backup_running_listener = Arc::clone(&backup_running);

    // Retrieve current monitor size
    /*let mut width=1920;
    let mut height=1080;
    if let Ok(size_tuple) = display_size() {
        width =  size_tuple.0;
        height = size_tuple.1;
        //println!("Current screen size: {}x{}", width, height);
    } else {
        println!("Failed to get primary monitor. Setting to default size {}x{}", width, height);
    }*/

    // Spawn a thread for logging CPU consumption
    thread::spawn(move || {
        let mut sys = System::new_all();
        // Open the log file in append mode
        let mut file = std::fs::File::create("./cpu_usage.log").expect("Failed to open log file");

        loop {
            // Refresh the CPU usage
            sys.refresh_all();
            // Get the PID of the current process
            let pid = sysinfo::get_current_pid().expect("Failed to get PID");
            if let Some(process) = sys.process(pid) {
                // Log CPU usage of the current process
                let cpu_usage = process.cpu_usage();
                //println!("CPU usage: {:.2}%", cpu_usage);
                log_cpu_usage(&mut file, cpu_usage);
            } else {
                println!("Process not found");
            }
            thread::sleep(std::time::Duration::from_secs(120));
        }
    });
    // Spawn a thread to listen for key events. Will exist till the end of the daemon program
    thread::spawn(move || {
        listen_for_keys(gui_opened_clone, backup_running_listener);
    });

    // Spawn a thread to listen for mouse events
    // thread::spawn(move || {
    //     listen_for_mouse_drag(listening_clone_mouse, width, height);
    // });
    
    loop {
        let gui_status = open_gui(backup_running.clone(), settings_mutex.clone(), get_value_condvar_tuple(backup_running.clone()));
        // When the GUI is closed, check the status (keep_alive)
        if gui_status.is_ok_and(|s| !s) {
            break;
        }
        // Wait until Keyboard thread request to open the GUI again
        set_and_wait_condvar_tuple(gui_opened.clone(), Some(false), Some(true));
    }

}