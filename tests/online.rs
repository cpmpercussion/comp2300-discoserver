// #[allow(dead_code)]
//
// use disco_emulator::Board;
//
// pub mod common;
// use crate::common::get_default_linker;
// use crate::common::compile_program;
// use common::get_online_src_path;
// use common::online::Online;
//
//
//
// #[test]
// fn test_openocd() {
//     let port = 50_030;
//
//     let src_path = get_online_src_path("program1").unwrap();
//     let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
//     let mut board = Board::new();
//     board.load_elf_from_path(&elf_path).unwrap();
//     let mut online = Online::new(&elf_path, port);
//
//     // board.step().unwrap();
//     online.step();
//
//     for i in 0..100 {
//         println!("Iteration {}", i);
//         board.step().unwrap();
//         online.step();
//         if let Err(e) = online.verify_state(&board) {
//             online.close();
//             println!("Step {} out of sync: {}", i, e);
//             assert!(false);
//         }
//     }
//
//     online.close();
// }
