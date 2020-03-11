#[allow(dead_code)]

use std::io::Write;
use disco_emulator::Board;

pub mod common;
use crate::common::get_default_linker;
use crate::common::compile_program;
use common::get_online_src_path;
use common::online::Online;

use std::net::{TcpStream, TcpListener, Shutdown};


#[test]
fn test_openocd() {
    let src_path = get_online_src_path("program1").unwrap();
    let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path).unwrap();
    let mut online = Online::new(&elf_path);

    let mut stream = match TcpStream::connect("127.0.0.1:6666") {
        Ok(s) => {
            write!(std::io::stdout(), "connected tcl\n");
            s
        },
        Err(e) => {
            write!(std::io::stdout(), "err 2: {}\n", e);
            return;
        }
    };

    stream.write("shutdown".as_bytes());

    online.close();

    stream.shutdown(std::net::Shutdown::Both);


    // let listener = match TcpListener::bind(format!("127.0.0.1:6666")) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         write!(std::io::stdout(), "err 1: {}", e);
    //         return;
    //     }
    // };
    //
    // match listener.accept() {
    //     Ok((socket, _addr)) => {
    //         write!(std::io::stdout(), "connected tcl");
    //     }
    //     Err(e) => {
    //         write!(std::io::stdout(), "error accepting connection: {:?}", e);
    //         return;
    //     }
    // }
    return;




    // board.step().unwrap();
    write!(std::io::stdout(), "stepping online\n");
    online.step();

    for i in 0..100 {
        println!("Iteration {}", i);
        board.step().unwrap();
        online.step();
        if let Err(e) = online.verify_state(&board) {
            online.close();
            println!("Step {} out of sync: {}", i, e);
            assert!(false);
        }
    }

    online.close();
}
