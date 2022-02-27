use std::{
    path::PathBuf,
    process,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::color::Color::*;

pub(crate) fn timeout_thread() -> (Sender<PathBuf>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let child = thread::spawn(move || {
        let mut prev = None;
        let mut duration_millis = 0;
        const INTERVAL: u64 = 50;
        loop {
            match rx.recv_timeout(Duration::from_millis(INTERVAL)) {
                Ok(path_buf) => {
                    prev = Some(path_buf);
                    duration_millis = 0;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(prev) = &prev {
                        duration_millis += INTERVAL;
                        if duration_millis == INTERVAL {
                            eprintln!("{}: Test case {prev:?} is taking >50ms", Yellow("Warning"));
                        } else if duration_millis == 5_000 {
                            eprintln!("{} after 5 secs", Red("Cancelled"));
                            process::exit(255);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    (tx, child)
}
