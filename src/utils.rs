use std::sync::{Arc, Condvar, Mutex};
/**
 * This function handle a tuple made up of (Mutex, Condvar)
 * - gui_opened: tuple
 * - wake_value: condvar waits until the mutex content reach this value. If None, condvar does not wait
 * - set_value: condvar set this value to the mutex, then calls notify_all(). If None, condvar does not set
 */
pub(crate) fn set_and_wait_condvar_tuple(cvar_tuple: Arc<(Mutex<bool>, Condvar)>, set_value: Option<bool>, wake_value: Option<bool>) {
    let (lock, cvar) = &*cvar_tuple;
    let mut opened = lock.lock().expect("Could not acquire lock");
    // Set
    if set_value.is_some() {
        *opened = set_value.unwrap();
        cvar.notify_all();
    }
    // Wait
    if wake_value.is_some() {
        // Keep waiting until the KeyListener thread sets opened = wake_value
        while !*opened {
            opened = cvar.wait(opened).expect("Could not wait for CondVar");
        }
    }
}
/**
 * This function handle a tuple made up of (Mutex, Condvar)
 * Return a bool indicating if the condvar is satisfied or not
 */
pub(crate) fn get_value_condvar_tuple(cvar_tuple: Arc<(Mutex<bool>, Condvar)>) -> bool {
    let (lock, _) = &*cvar_tuple;
    *lock.lock().expect("Could not acquire lock")
}
