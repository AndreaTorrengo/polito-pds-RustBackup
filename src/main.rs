mod settings;
mod backup;
mod daemon;
mod ui;
mod utils;

use settings::*;
use daemon::start_daemon;


fn main() {
    let mut config = Settings::default();
    read_settings(CONFIG_FILE_PATH, &mut config);

    // start daemon process that listens to keyboard/mouse events. Consumes config object
    start_daemon(config);
}
