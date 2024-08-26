use std::{rc::Rc, sync::{Arc, Condvar, Mutex}};
slint::include_modules!();

use crate::utils::get_value_condvar_tuple;
use crate::settings::*;

/**
 * Open up the GUI: this is a blocking statement, unlocked when the UI is closed
 * - backup_running: used to prevent closing the GUI while backup is still running
 * returns a boolean (keep_alive) indicating if the main thread should still running or not after the window is closed
 */
pub(crate) fn open_gui(backup_running: Arc<(Mutex<bool>, Condvar)>, settings: &mut Settings) -> Result<bool, slint::PlatformError> {
    let ui = AppWindow::new().expect("Could not open GUI");
    let keep_alive = Rc::new(Mutex::new(settings.keep_daemon_alive));
    let keep_alive_clone = Rc::clone(&keep_alive);

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    // When user wants to close the whole daemon and GUI
    ui.on_request_exit({
        let ui_handle = ui.as_weak();
        move || {
            if get_value_condvar_tuple(backup_running.clone()) {
                // error popup
                println!("Cannot close GUI while backup is running");
            } else {
                let _ = ui_handle.unwrap().hide();
                *keep_alive_clone.lock().unwrap() = false;
            }
        }
    });
    ui.run()?;
    // Signal to the main (daemon) thread to continue execution or not
    let val = *keep_alive.lock().unwrap();
    Ok(val)
}