use std::env;
use std::path::Path;
use crossbeam_channel::{Receiver, Sender};

mod reader;
mod worker;



fn main() {
    let zip_path = env::args().nth(1).unwrap();
    let dictionary_path = "/opt/dict/passwd_10m.txt";
    let workers = 3;
    passwd_finder(&zip_path, dictionary_path, workers);
}

pub fn passwd_finder(zip_path: &str, passwd_list_path: &str, workers: usize) {
    let zip_file_path = Path::new(zip_path);
    let passwd_list_file_path = Path::new(passwd_list_path).to_path_buf();
    let (send_passwd, receive_passwd):(Sender<String>, Receiver<String>) = crossbeam_channel::bounded(workers * 10_000);
    let passwd_gen_handle = reader::start_password_reader(passwd_list_file_path, send_passwd);
    let mut worker_handles = Vec::with_capacity(workers);
    for i in 1..=workers {
        let h = worker::password_checker(i, zip_file_path, receive_passwd.clone());
        worker_handles.push(h);
    }

    for h in worker_handles {
        h.join().unwrap();
    }
    passwd_gen_handle.join().unwrap();
}