use std::io::{Read, Write};
use std::fs;

#[derive(Debug)]
pub struct Settings {
    pub(crate) paths_to_backup: Vec<String>,
    pub(crate) auto_start: bool,
    pub(crate) multi_thread: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self { 
            paths_to_backup: vec![home::home_dir().expect("Cannot find home directory for this user").to_str().unwrap().to_owned()], 
            auto_start: true,
            multi_thread: true,
        }
    }
}

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
    let file = fs::File::open(config_file_path);
    if file.is_ok() {
        let _ = file.unwrap().read_to_string(&mut file_content);
        let content_lines: Vec<String> = file_content.split("\n").map(|s| s.to_string()).collect();
        // config.auto_start = content_lines[num_path_lines + 1].trim().contains("true");

        let mut path_begin_line: Option<usize> = None;
        let mut path_end_line: Option<usize> = None;
        let mut auto_start_line: Option<usize> = None;
        let mut multi_thread_line: Option<usize> = None;
        for (index, elm) in content_lines.iter().enumerate() {
            if path_begin_line.is_none() && elm.contains("paths_to_backup") {
                path_begin_line = Some(index + 1);
            }
            if path_end_line.is_none() && elm.contains("}") {
                path_end_line = Some(index - 1);
            }
            if auto_start_line.is_none() && elm.contains("auto_start") {
                auto_start_line = Some(index);
            }
            if multi_thread_line.is_none() && elm.contains("multi_thread") {
                multi_thread_line = Some(index);
            }
        }
        if path_begin_line.is_none() || path_end_line.is_none() || auto_start_line.is_none() || multi_thread_line.is_none() {
            panic!("Invalid config in {}", config_file_path);
        }
        let path_lines = &content_lines[path_begin_line.unwrap()..=path_end_line.unwrap()];
        config.paths_to_backup = path_lines.iter().map(|p| p.trim().to_string()).collect();
        config.auto_start = content_lines[auto_start_line.unwrap()].contains("true");
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
    file.write(format!("auto_start={}", config.auto_start).as_bytes()).unwrap();
    file.write(format!("multi_thread={}", config.multi_thread).as_bytes()).unwrap();
}