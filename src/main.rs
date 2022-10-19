use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::{Receiver, Sender};

mod reader;
mod worker;



fn main() {
    let zip_path = env::args().nth(1).unwrap();
    let dictionary_path = "/opt/dict/passwd_10m.txt";
    let workers = 8;

    match passwd_finder(&zip_path, dictionary_path, workers) {
        Some(f) => println!("passwd found:{}", f),
        None => println!("passwd not found"),
    }
}

pub fn passwd_finder(zip_path: &str, passwd_list_path: &str, workers: usize) -> Option<String> {
    let zip_file_path = Path::new(zip_path);
    let passwd_list_file_path = Path::new(passwd_list_path).to_path_buf();
    let (send_passwd, receive_passwd):(Sender<String>, Receiver<String>) = crossbeam_channel::bounded(workers * 10_000);

    let (send_found, receive_found): (Sender<String>, Receiver<String>) = crossbeam_channel::bounded(1);
    let stop_worker_signal = Arc::new(AtomicBool::new(false));
    let stop_master_signal = Arc::new(AtomicBool::new(false));


    let passwd_gen_handle = reader::start_password_reader(
        passwd_list_file_path,
        send_passwd,
        stop_master_signal.clone()
    );
    let mut worker_handles = Vec::with_capacity(workers);
    for i in 1..=workers {
        let h = worker::password_checker(
            i, zip_file_path,
            receive_passwd.clone(),
            stop_worker_signal.clone(),
            send_found.clone(),
        );
        worker_handles.push(h);
    }

    // for h in worker_handles {
    //     h.join().unwrap();
    // }
    drop(send_found);
    match receive_found.recv() {
        Ok(f) =>{
            stop_master_signal.store(true, Ordering::Relaxed);
            passwd_gen_handle.join().unwrap();

            stop_worker_signal.store(true, Ordering::Relaxed);
            for h in worker_handles {
                h.join().unwrap();
            }
            Some(f)
        }
        Err(_) => None
    }
    // passwd_gen_handle.join().unwrap();
}