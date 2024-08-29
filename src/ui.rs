use std::{rc::Rc, sync::{Arc, Condvar, Mutex}, thread};
use native_dialog::FileDialog;
use slint::{invoke_from_event_loop, ComponentHandle};
slint::include_modules!();

use crate::utils::{get_value_condvar_tuple, set_and_wait_condvar_tuple};
use crate::settings::*;
use crate::backup::*;

fn slint_serialize_settings(settings: &Settings) -> AppSettings {
    AppSettings{
        keep_alive: settings.keep_daemon_alive, 
        multithread: settings.multi_thread, 
        paths_to_backup: slint::SharedString::from(settings.paths_to_backup.join("\n")),
        default_backup_path: slint::SharedString::from(&settings.default_backup_path),
    }
}

/**
 * Open up the GUI: this is a blocking statement, unlocked when the UI is closed
 * - backup_running: used to prevent closing the GUI while backup is still running
 * returns a boolean (keep_alive) indicating if the main thread should still running or not after the window is closed
 */
pub(crate) fn open_gui(bkrn_daemon: Arc<(Mutex<bool>, Condvar)>, settings_mutex: Arc<Mutex<Settings>>, backupnow: bool) -> Result<bool, slint::PlatformError> {
    let ui = AppWindow::new().expect("Could not open GUI");
    let ui_weak = ui.as_weak();
    let settings_clone_save = Arc::clone(&settings_mutex);
    let settings = settings_mutex.lock().unwrap();
    let keep_alive = Rc::new(Mutex::new(settings.keep_daemon_alive));
    let keep_alive_clone_req_exit = Rc::clone(&keep_alive);

    let backup_running: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));

    let backup_running_clone_thread = backup_running.clone();
    let backup_running_clone_backup = backup_running.clone();

    // Pass Settings struct to settings page
    let sett = slint_serialize_settings(&settings);
    ui.set_app_settings(sett);
    drop(settings);

    {
        let ui_handle = ui_weak.clone();
        let _ = invoke_from_event_loop(
            move || {
            if backupnow {
                ui_handle.unwrap().invoke_request_backup();
            }
        });
    }
    // Save settings event
    ui.on_request_save_settings(move |app_settings: AppSettings| {
        let mut deser = settings_clone_save.lock().unwrap();
        deser.keep_daemon_alive = app_settings.keep_alive;
        deser.multi_thread = app_settings.multithread;
        deser.paths_to_backup = app_settings.paths_to_backup.split("\n").filter(|s| !s.is_empty()).map(|x| x.to_string()).collect();
        deser.default_backup_path = app_settings.default_backup_path.to_string();
        write_settings(CONFIG_FILE_PATH, &deser);
        // println!("{:?}", app_settings);
    });
    // Open Directory/File event
    ui.on_request_path_dialog(move |open_directory_path| {
        let path_obj = FileDialog::new().set_location("~");
        let path_opt;
        if open_directory_path {
            path_opt = path_obj.set_title("Open directory").show_open_single_dir().unwrap();
        } else {
            path_opt = path_obj.set_title("Open file").show_open_single_file().unwrap();
        }
        
        let path = match path_opt {
            Some(path) => slint::SharedString::from(path.to_str().unwrap()),
            None => return slint::SharedString::from(""),
        };
        path
    });
    ui.on_request_backup(move || {
        set_and_wait_condvar_tuple(backup_running_clone_backup.clone(), Some(true), None);
    });
    // When user wants to close the whole daemon and GUI
    ui.on_request_exit({
        let ui_handle = ui.as_weak();
        let bkrn = backup_running.clone();
        move || {
            if get_value_condvar_tuple(bkrn.clone()) {
                // error popup
                println!("Cannot close GUI while backup is running");
            } else {
                let _ = ui_handle.unwrap().hide();
                *keep_alive_clone_req_exit.lock().unwrap() = false;
            }
        }
    });
    
    
    thread::spawn(move || {
        let settings_clone_backup = Arc::clone(&settings_mutex);
        let ui_handle = ui_weak.clone();
        loop {
            set_and_wait_condvar_tuple(backup_running_clone_thread.clone(), None, Some(true));

            // Show backup page
            let ui_handle_clone = ui_handle.clone();
            invoke_from_event_loop(move || {
                if let Some(ui_weak) = ui_handle_clone.upgrade() {
                    ui_weak.set_backup_completed(false);
                    ui_weak.set_backup_perc(0.0);
                    ui_weak.set_selected_page(2);
                }
            }).unwrap();
            match backup(settings_clone_backup.clone(), ui_handle.clone()) {
                Ok(()) => {},
                Err(e) => eprint!("Error while trying to backup: {:?}", e.kind),
            };
            set_and_wait_condvar_tuple(backup_running_clone_thread.clone(), Some(false), None);
            set_and_wait_condvar_tuple(bkrn_daemon.clone(), Some(false), None);

            let ui_handle_clone = ui_handle.clone();
            invoke_from_event_loop(move || {
                if let Some(ui_weak) = ui_handle_clone.upgrade() {
                    ui_weak.set_backup_completed(true);
                    ui_weak.set_backup_perc(1.0);
                }
            }).unwrap();
        }
    });
    
    

    ui.run()?;
    // Signal to the main (daemon) thread to continue execution or not
    let val = *keep_alive.lock().unwrap();
    Ok(val)
}