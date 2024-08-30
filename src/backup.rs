use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

extern crate fs_extra;
use fs_extra::{copy_items_with_progress, TransitProcess};
use fs_extra::dir::{CopyOptions, create_all};
use fs_extra::error::*;
use slint::invoke_from_event_loop;
use crate::settings::*;
use crate::ui::AppWindow;

pub(crate) fn backup(config: Arc<Mutex<Settings>>, ui_handle: slint::Weak<AppWindow>) -> Result<()> {
    // Create the destination directory, if it does not already exist
    let settings = config.lock().unwrap();
    let path_to = Path::new(settings.default_backup_path.as_str());
    if !path_to.exists() {
        create_all(&path_to, true)?;
    }

    let mut opt = CopyOptions::new();
    //opt.copy_inside = true;
    opt.overwrite = true;
    //opt.buffer_size = 1;
    let (tx, rx): (Sender<TransitProcess>, Receiver<TransitProcess>) = mpsc::channel();
    let mut path_from_vec: Vec<_> = vec![];
    let mut dir_file_log = settings.default_backup_path.clone();
    //settings.paths_to_backup.iter().for_each(|path| { path_from_vec.push(path.as_str()); });
    settings.paths_to_backup.iter().for_each(|path| { path_from_vec.push(Path::new(path)); });
    let multithread_enabled = settings.multi_thread;
    let npaths = settings.paths_to_backup.len();

    let ref_path_from_vec = &path_from_vec;
    let ref_opt = &opt;
    
    thread::scope(|s| {
        struct FileStat {
            dir: String,
            file: String,
            status: f32,
        }
        // Receiver thread loop
        thread::spawn(move || {
            let files_status_mutex: Arc<Mutex<Vec<FileStat>>> = Arc::new(Mutex::new(Vec::new()));
            let backup_log_mutex = Arc::new(Mutex::new(String::from("Beginning backup\n------------------------")));

            loop {
                let files_status_m1 = files_status_mutex.clone();
                let backup_log_m1 = backup_log_mutex.clone();
                let mut file_status_1 = files_status_m1.lock().unwrap();
                let mut backup_log_1 = backup_log_m1.lock().unwrap();

                let backup_log_m2 = backup_log_mutex.clone();
                let ui_handle_clone = ui_handle.clone();

                match rx.try_recv() {
                    Ok(process_info) => {
                        let elm = file_status_1.iter_mut().find(|x| x.dir == process_info.dir_name && x.file == process_info.file_name);
                        
                        if elm.is_none() {
                            backup_log_1.push_str(format!("\nBacking up {}/{}   (size: {:.2} KB)...", process_info.dir_name, process_info.file_name, process_info.total_bytes as f32 / 1024.0).as_str());
                            file_status_1.push(FileStat{
                                dir: process_info.dir_name, 
                                file: process_info.file_name, 
                                status: process_info.copied_bytes as f32 / process_info.total_bytes as f32
                            });
                            invoke_from_event_loop(move || {
                                let bk = backup_log_m2.lock().unwrap();
                                if let Some(ui_weak) = ui_handle_clone.upgrade() {
                                    ui_weak.set_backup_log(slint::SharedString::from(bk.as_str()));
                                }
                            }).unwrap();
                        } else {
                            elm.unwrap().status = process_info.copied_bytes as f32 / process_info.total_bytes as f32;
                            let status_sum = file_status_1.iter().map(|x| x.status / npaths as f32).sum();
                            invoke_from_event_loop(move || {
                                if let Some(ui_weak) = ui_handle_clone.upgrade() {
                                    ui_weak.set_backup_perc(status_sum);
                                }
                            }).unwrap();
                        }
                        // println!("{} of {} bytes",
                        //     process_info.copied_bytes,
                        //     process_info.total_bytes);
                    }
                    Err(TryRecvError::Disconnected) => {
                        backup_log_1.push_str("\n\n ====>  Backup completed successfully  <====");
                        let bk = backup_log_m2.clone();
                        invoke_from_event_loop(move || {
                            if let Some(ui_weak) = ui_handle_clone.upgrade() {
                                ui_weak.set_backup_log(slint::SharedString::from(bk.lock().unwrap().as_str()));
                            }
                        }).unwrap();
                        println!("Backup completed!");
                        // Save backup log
                        if !dir_file_log.ends_with("/") && !dir_file_log.ends_with("\\") {
                            dir_file_log.push('/');
                        }
                        let mut file = std::fs::File::create(format!("{}backup.log", dir_file_log)).expect("Failed to create backup log file");
                        let _ = file.write_all(backup_log_1.as_bytes());
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }
            }
        });

        let mut nthread: usize = 1;
        let mut index_path: usize = 0;
        let n_paths = path_from_vec.len();

        if multithread_enabled {
            nthread = std::thread::available_parallelism().unwrap().get();   // get number of core for current CPU
        }
        
        while index_path < n_paths {
            let mut index_core = 0;
            let mut thread_handle = vec![];
            while index_path < n_paths && index_core < nthread {
                let tx_clone = tx.clone();
                thread_handle.push(s.spawn(move || {
                    
                    let handler = |process_info: TransitProcess| {
                        let tx_clone_thread = tx_clone.clone();
                        tx_clone_thread.send(process_info).unwrap();
                        thread::sleep(time::Duration::from_micros(500));
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    };
                    
                    match copy_items_with_progress(&[ref_path_from_vec[index_path]], &path_to, ref_opt, handler) {
                        Ok(_) => {},
                        Err(e) => { println!("Cannot copy path {:?}.\nThrown error: {}", ref_path_from_vec[index_path], e) },
                    }
                }));
                index_core += 1;
                index_path += 1;
            }
            thread_handle.into_iter().for_each(|handle| handle.join().unwrap());
        }
    });
    
    Ok(())

}