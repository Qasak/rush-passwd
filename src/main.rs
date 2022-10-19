use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::{Receiver, Sender};
use indicatif;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

mod reader;
mod worker;



fn main() {
    let zip_path = env::args().nth(1).unwrap();
    let dict_path = "/opt/dict/passwd_10m.txt";
    let workers = 8;

    match passwd_finder(&zip_path, dict_path, workers) {
        Some(f) => println!("passwd found:{}", f),
        None => println!("passwd not found"),
    }
}

pub fn passwd_finder(zip_path: &str, dict_path: &str, workers: usize) -> Option<String> {
    let zip_file_path = Path::new(zip_path);
    let input_path = Path::new(dict_path).to_path_buf();
    let progress_bar = ProgressBar::new(0);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);
    let (send_input, receive_input):(Sender<String>, Receiver<String>) = crossbeam_channel::bounded(workers * 10_000);
    let file = BufReader::new(File::open(input_path.clone()).expect("Unable to open file"));
    let cnt = file.lines().count();
    progress_bar.set_length(cnt as u64);

    let (send_found, receive_found): (Sender<String>, Receiver<String>) = crossbeam_channel::bounded(1);
    let stop_worker_signal = Arc::new(AtomicBool::new(false));
    let stop_master_signal = Arc::new(AtomicBool::new(false));


    let passwd_gen_handle = reader::start_password_reader(
        input_path,
        send_input,
        stop_master_signal.clone()
    );
    let mut worker_handles = Vec::with_capacity(workers);
    for i in 1..=workers {
        let h = worker::password_checker(
            i, zip_file_path,
            receive_input.clone(),
            stop_worker_signal.clone(),
            send_found.clone(),
            progress_bar.clone(),
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
            progress_bar.finish_and_clear();
            Some(f)
        }
        Err(_) => None
    }
    // passwd_gen_handle.join().unwrap();
}