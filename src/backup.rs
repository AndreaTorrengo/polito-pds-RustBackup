use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

extern crate fs_extra;
use fs_extra::dir::*;
use fs_extra::error::*;
use crate::settings::*;

pub(crate) fn backup(conf: &Settings, to: &'static str) -> Result<()> {
    let config: Arc<Mutex<&Settings>> = Arc::new(Mutex::new(conf));
    let path_to = Path::new(to);
    create_all(&path_to, true)?;    // Create the destination directory, if it does not already exist

    let opt = CopyOptions::new();
    //opt.buffer_size = 1;
    let (tx, rx): (Sender<TransitProcess>, Receiver<TransitProcess>) = mpsc::channel();
    let config_clone = config.clone();
    let config_mutex = config_clone.lock().unwrap();
    let mut path_from_vec: Vec<& Path> = vec![];
    config_mutex.paths_to_backup.iter().for_each(|path| { path_from_vec.push(Path::new(path)); });

    let ref_path_from_vec = &path_from_vec;
    let ref_opt = &opt;

    thread::scope(|s| {
        // Receiver thread loop
        thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(process_info) => {
                        println!("{} of {} bytes",
                                process_info.copied_bytes,
                                process_info.total_bytes);
                    }
                    Err(TryRecvError::Disconnected) => {
                        println!("Backup completed!");
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }
            }
        });

        let mut nthread: usize = 1;
        let mut index_path: usize = 0;
        let n_paths = path_from_vec.len();

        if config_clone.lock().unwrap().multi_thread {
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
                        thread::sleep(time::Duration::from_millis(500));
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    };
                    
                    match copy_with_progress(ref_path_from_vec[index_path], &path_to, ref_opt, handler) {
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