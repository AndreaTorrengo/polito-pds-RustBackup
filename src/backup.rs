use std::path::Path;
use std::{thread, time};
use std::sync::mpsc::{self, TryRecvError};

extern crate fs_extra;
use fs_extra::dir::*;
use fs_extra::error::*;

pub(crate) fn backup(paths_to_backup: Vec<String>, to: &'static str) -> Result<()> {
    let path_to = Path::new(to);
    let options = CopyOptions::new();
    let (tx, rx) = mpsc::channel();
    //options.buffer_size = 1;

    thread::spawn(move || {
        let mut path_from_vec: Vec<& Path> = vec![];
        paths_to_backup.iter().for_each(|path| { path_from_vec.push(Path::new(path)); });

        let handler = |process_info: TransitProcess| {
            tx.send(process_info).unwrap();
            thread::sleep(time::Duration::from_millis(500));
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        };
        for path in path_from_vec.iter() {
            match copy_with_progress(*path, &path_to, &options, handler) {
                Ok(_) => {},
                Err(e) => { println!("Cannot copy path {:?}.\nThrown error: {}", path, e) },
            }
        }
    });
    
    loop {
        match rx.try_recv() {
            Ok(process_info) => {
                println!("{} of {} bytes",
                         process_info.copied_bytes,
                         process_info.total_bytes);
            }
            Err(TryRecvError::Disconnected) => {
                println!("finished");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
    Ok(())

}