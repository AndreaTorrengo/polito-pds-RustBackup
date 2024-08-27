use std::io::{Read, Write, BufReader};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Settings {
    pub(crate) paths_to_backup: Vec<String>,
    pub(crate) keep_daemon_alive: bool,
    pub(crate) multi_thread: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self { 
            paths_to_backup: vec![home::home_dir().expect("Cannot find home directory for this user").to_str().unwrap().to_owned()], 
            keep_daemon_alive: true,
            multi_thread: true,
        }
    }
}
pub const CONFIG_FILE_PATH: &str = "./settings.dat";

/**
 * Reads settings file. If it does not exists, create a new one.
 * File format example:
 * paths_to_backup={
 *      <path1>
 *      <...>
 *      <pathN>
 * }
 * auto_start=true
 */
pub(crate) fn read_settings(config_file_path: &str, config: &mut Settings) {
    let mut file_content = String::new();

    if Path::new(config_file_path).exists() {
        let file = fs::File::open(config_file_path);
        let _ = BufReader::new(file.unwrap()).read_to_string(&mut file_content);
        let content_lines: Vec<String> = file_content.split("\n").filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
        let mut path_begin_line: Option<usize> = None;
        let mut path_end_line: Option<usize> = None;
        let mut keep_daemon_alive_line: Option<usize> = None;
        let mut multi_thread_line: Option<usize> = None;

        for (index, elm) in content_lines.iter().enumerate() {
            if path_begin_line.is_none() && elm.contains("paths_to_backup") {
                path_begin_line = Some(index + 1);
            }
            if path_end_line.is_none() && elm.contains("}") {
                path_end_line = Some(index - 1);
            }
            if keep_daemon_alive_line.is_none() && elm.contains("keep_daemon_alive") {
                keep_daemon_alive_line = Some(index);
            }
            if multi_thread_line.is_none() && elm.contains("multi_thread") {
                multi_thread_line = Some(index);
            }
        }
        if path_begin_line.is_none() || path_end_line.is_none() || keep_daemon_alive_line.is_none() || multi_thread_line.is_none() {
            panic!("Invalid config in {}", config_file_path);
        }
        let path_lines = &content_lines[path_begin_line.unwrap()..=path_end_line.unwrap()];
        config.paths_to_backup = path_lines.iter().map(|p| p.trim().to_string()).collect();
        config.keep_daemon_alive = content_lines[keep_daemon_alive_line.unwrap()].contains("true");
        config.multi_thread = content_lines[multi_thread_line.unwrap()].contains("true");
    } else {
        *config = Settings::default();
        write_settings(config_file_path, config);
    }
}
/**
 * Completely overwrites settings file. If it does not exists, create a new one
 */
pub(crate) fn write_settings(config_file_path: &str, config: &Settings) {
    let file_opt = fs::File::create(config_file_path);
    if file_opt.is_err() { println!("Could not write on {}", config_file_path); }
    let mut file = file_opt.unwrap();

    file.write(b"paths_to_backup={\n\t").unwrap();
    file.write(config.paths_to_backup.join("\n\t").as_bytes()).unwrap();
    file.write(b"\n}\n").unwrap();
    file.write(format!("keep_daemon_alive={}\n", config.keep_daemon_alive).as_bytes()).unwrap();
    file.write(format!("multi_thread={}", config.multi_thread).as_bytes()).unwrap();
}