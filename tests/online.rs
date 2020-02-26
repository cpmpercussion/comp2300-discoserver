#[allow(dead_code)]

use disco_emulator::Board;

mod common;
use crate::common::get_default_linker;
use crate::common::compile_program;
use common::get_online_src_path;
use common::online::Online;



#[test]
fn test_openocd() {
    let port = 50_030;

    let src_path = get_online_src_path("program1").unwrap();
    let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path).unwrap();
    let mut online = Online::new(&elf_path, port);

    online.exec_gdb("-target-download");
    // online.read_until("(gdb)");



    online.exec_gdb("-exec-step");
    // online.read_until("(gdb)");

    online.exec_gdb("-data-list-register-values x [0, 1, 15]");
    online.read_until("^done");
    online.read_until("^done");

    // for _ in 0..1 {
    //     online.step();
    // }

    println!("all good");

    online.close();
    assert!(false);
}
