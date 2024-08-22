mod settings;
mod backup;
use settings::*;
use backup::*;


fn main() {
    const CONFIG_FILE_PATH: &str = "./settings.dat";
    const DEFAULT_BACKUP_DIR: &str = "./backup/";
    //let mut config = config_mutex.lock().unwrap();
    let mut config = Settings::default();
    read_settings(CONFIG_FILE_PATH, &mut config);
    let paths_to_backup = config.paths_to_backup.clone();

    //match backup(&config.paths_to_backup.iter().map(|p| p.as_str()).collect(), &DEFAULT_BACKUP_DIR) {
    match backup(paths_to_backup, &DEFAULT_BACKUP_DIR) {
        Ok(()) => println!("backup complete"),
        Err(e) => eprint!("Error when trying to backup: {}", e),
    };
    //write_settings(CONFIG_FILE_PATH, &config);
    println!("{:?}", config);
}
