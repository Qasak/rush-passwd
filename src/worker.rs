use std::{fs, thread};
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use indicatif::ProgressBar;

pub fn password_checker(
    index: usize,
    file_path: &Path,
    receive_password: Receiver<String>,
    stop_signal: Arc<AtomicBool>,
    passwd_sender: Sender<String>,
    progress_bar: ProgressBar,
) -> JoinHandle<()> {
    let file = fs::File::open(file_path).expect("File should exist");
    thread::Builder::new()
        .name(format!("worker-{}", index))
        .spawn(move || {
            println!("worker-{} start", index);
            let mut archive = zip::ZipArchive::new(file).expect("zip should valid");
            while !stop_signal.load(Ordering::Relaxed) {
                match receive_password.recv() {
                    Err(_) => break,
                    Ok(passwd) => {
                        let res = archive.by_index_decrypt(0, passwd.as_bytes());
                        match res {
                            Err(e) => panic!("ZipError {:?}", e),
                            Ok(Err(_)) => (),
                            Ok(Ok(mut zip)) => {
                                // Validate password by reading the zip file to make sure it is not merely a hash collision.
                                let mut buffer = Vec::with_capacity(zip.size() as usize);
                                match zip.read_to_end(&mut buffer) {
                                    Ok(_) => {
                                        passwd_sender
                                            .send(passwd)
                                            .expect("Send found passwd shouldn't fail");
                                    },
                                    // password collision - continue
                                    Err(_) => ()
                                }
                            }
                        }
                    },
                }
                progress_bar.inc(1);
            }
        })
        .unwrap()
}