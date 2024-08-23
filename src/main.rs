mod settings;
mod backup;

use settings::*;
use backup::*;


fn main() {
    const CONFIG_FILE_PATH: &str = "./settings.dat";
    const DEFAULT_BACKUP_DIR: &str = "./backup/";
    let mut config = Settings::default();
    read_settings(CONFIG_FILE_PATH, &mut config);

    match backup(&config, &DEFAULT_BACKUP_DIR) {
        Ok(()) => {},
        Err(e) => eprint!("Error when trying to backup: {}", e),
    };
    //write_settings(CONFIG_FILE_PATH, &config);
    println!("{:?}", config);
}
