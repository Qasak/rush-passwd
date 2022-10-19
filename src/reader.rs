use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use anyhow::Result;

pub fn start_password_reader(file_path: PathBuf, send_password: Sender<String>, stop_signal: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::Builder::new()
        .name("password-reader".to_string())
        .spawn(move || {
            let file = File::open(file_path).expect("File should exist");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if stop_signal.load(Ordering::Relaxed) {
                    break;
                } else {
                    match send_password.send(line.unwrap()) {
                        Ok(_) => {}
                        Err(_) => break, // channel disconnected, stop thread
                    }
                }

            }
        })
        .unwrap()
}

