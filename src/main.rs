mod settings;
mod backup;
mod daemon;
mod ui;
mod utils;

use settings::*;
use backup::*;
use daemon::start_daemon;


fn main() {
    const DEFAULT_BACKUP_DIR: &str = "./backup/";
    let mut config = Settings::default();
    read_settings(CONFIG_FILE_PATH, &mut config);

    // start daemon process that listens to keyboard/mouse events. Consumes config object
    start_daemon(config);

    // match backup(&config, &DEFAULT_BACKUP_DIR) {
    //     Ok(()) => {},
    //     Err(e) => eprint!("Error when trying to backup: {}", e),
    // };
    //write_settings(CONFIG_FILE_PATH, &config);
    //println!("{:?}", config);
}
