#[allow(dead_code)]

use std::io::Write;
use disco_emulator::Board;

pub mod common;
use crate::common::get_default_linker;
use crate::common::compile_program;
use common::get_online_src_path;
use common::online::Online;

fn run_program(name: &str) {
    let src_path = get_online_src_path(name).unwrap();
    let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path).unwrap();
    let mut online = Online::new(&elf_path).unwrap();
    write!(std::io::stdout(), "Running {}", name).unwrap();


    let mut i = 0;
    loop {
        write!(std::io::stdout(), ".").unwrap();
        std::io::stdout().flush().unwrap();
        if let Err(e) = online.verify_state(&board) {
            println!("Step {} out of sync: {}", i, e);
            online.close();
            assert!(false);
        }

        if board.read_lr() == 0x444F4E45 {
            break;
        }

        board.step().unwrap();
        online.step();
        i += 1;
    }
    write!(std::io::stdout(), "\n").unwrap();
    online.close();
}


#[test]
fn test_online() {
    // These tests use the physical board, so we can only
    // run one at a time.

    let programs = [
        "offline_mirror",
    ];

    for program in programs.iter() {
        run_program(program);
    }
}
