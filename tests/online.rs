#[allow(dead_code)]

use std::io::Write;
use disco_emulator::Board;

pub mod common;
use crate::common::get_default_linker;
use crate::common::compile_program;
use common::get_online_src_path;
use common::online::Online;


#[test]
fn test_openocd() {
    let src_path = get_online_src_path("program1").unwrap();
    let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path).unwrap();
    let mut online = Online::new(&elf_path).unwrap();

    write!(std::io::stdout(), "Running program").unwrap();
    for i in 0..100 {
        write!(std::io::stdout(), ".").unwrap();
        std::io::stdout().flush().unwrap();
        if let Err(e) = online.verify_state(&board) {
            println!("Step {} out of sync: {}", i, e);
            online.close();
            assert!(false);
        }
        board.step().unwrap();
        online.step();
    }
    write!(std::io::stdout(), "\n").unwrap();
    online.close();
}
