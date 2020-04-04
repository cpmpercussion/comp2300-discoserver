#![allow(dead_code)]

mod server;
use server::{GdbServer, get_elf_file_path_from_argv};

use disco_emulator::{self, Board};

use std::sync::mpsc::sync_channel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    if get_version_from_argv() {
        println!("disco-emulator v{}", disco_emulator::get_version());
        println!("disco-server v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some((start, count)) = get_audio_samples() {
        let elf_path = get_elf_file_path_from_argv().expect("Path to elf file required");
        let mut board = Board::new();
        board.load_elf_from_path(&elf_path).expect("Failed to load from ELF file");

        let (tx, rx) = sync_channel(1);
        let rx = Arc::new(Mutex::new(rx));

        let end = Arc::new(AtomicBool::new(false));
        let end_t = Arc::clone(&end);

        let samples = Arc::new(Mutex::new(Vec::new()));
        let samples_t = Arc::clone(&samples);

        let handle = thread::spawn(move || {
            let end = end_t;
            let rx = rx.lock().unwrap();
            let mut samples = samples_t.lock().unwrap();

            for _ in 0..start {
                let _ = rx.recv().unwrap();
            }

            for _ in 0..count {
                samples.push(rx.recv().unwrap());
            }

            end.store(true, Ordering::Relaxed);
        });

        board.audio_handler.set_observer(tx);

        while !end.load(Ordering::Relaxed) {
            board.step().unwrap();
        }

        handle.join().unwrap();

        let samples: &Vec<i16> = &samples.lock().unwrap();
        println!("===start-samples===");
        for sample in samples {
            println!("{}", sample);
        }
        println!("===end-samples===");
        return;
    }

    println!("started emulator server");
    GdbServer::start_server();
}

fn get_version_from_argv() -> bool {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "-v" || arg == "--version" {
            return true;
        }
    }
    return false;
}

fn get_audio_samples() -> Option<(usize, usize)> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "--samples" {
            let start_err = "Could not read start sample";
            let count_err = "Could not read sample count";
            let start = usize::from_str_radix(&args.next().expect(&start_err), 10).expect(&start_err);
            let count = usize::from_str_radix(&args.next().expect(&count_err), 10).expect(&count_err);
            return Some((start, count));
        }
    }
    return None;
}
