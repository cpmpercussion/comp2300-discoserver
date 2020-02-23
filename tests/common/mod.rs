use disco_emulator::Board;

use std::path::{Path, PathBuf};

use std::process::Command;

fn get_tests_path() -> Result<PathBuf, String> {
    let exec_path = match std::env::args().nth(0) {
        Some(p) => p,
        None => {
            return Err("Failed to find tests path".to_string());
        }
    };
    let mut path = PathBuf::new();
    path.push(exec_path);
    path.pop();
    path.pop();
    path.pop();
    path.pop();
    path.push("tests");

    println!("tests: {:?}", path);
    return Ok(path);
}

fn compile_program(src_path: &Path, linker_path: &Path) -> Result<PathBuf, String> {
    let mut elf_path = PathBuf::from(&src_path);
    elf_path.push("firmware.elf");

    let mut asm_child = Command::new("arm-none-eabi-as")
                                .arg("-mthumb")
                                .arg("-mcpu=cortex-m4")
                                .arg("-o")
                                .arg("main.o")
                                .arg("main.S")
                                .current_dir(&src_path)
                                .spawn()
                                .expect("failed to execute assembler; is `arm-none-eabi-as` on your PATH?");

    let ecode = asm_child.wait().expect("failed to wait on assembler");
    if !ecode.success() {
        return Err("failed to assemble source".to_string());
    }

    let mut ld_child = Command::new("arm-none-eabi-ld")
                                .arg("-T")
                                .arg(&linker_path)
                                .arg("-nostartfiles")
                                .arg("-o")
                                .arg(&elf_path)
                                .arg("main.o")
                                .current_dir(&src_path)
                                .spawn()
                                .expect("failed to execute linker; is `arm-none-eabi-ld` on your PATH?");
    let ecode = ld_child.wait().expect("failed to wait on linker");
    if !ecode.success() {
        return Err("failed to link source".to_string());
    }
    return Ok(elf_path);
}

pub fn load_program(name: &str) -> Result<Board, String> {
    let mut path = get_tests_path()?;
    path.push("fixtures");

    let mut linker_path = PathBuf::from(&path);
    linker_path.push("common");
    linker_path.push("linker.ld");
    println!("linker: {:?}", linker_path);

    let mut src_path = PathBuf::from(&path);
    src_path.push("instructions");
    src_path.push(name);
    println!("src: {:?}", src_path);

    let elf_path = compile_program(&src_path, &linker_path)?;
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path)?;

    return Ok(board);
}

pub fn load_and_step(name: &str, steps: usize) -> Result<Board, String> {
    let mut board = load_program(name)?;

    for _ in 0..steps {
        match board.step() {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to step board");
                assert!(false);
                return Err(e);
            }
        }
    }

    return Ok(board);
}

pub fn load_and_wait(name: &str, wait_reg: u32, wait_signal: u32) -> Result<Board, String> {
    let mut board = load_program(name)?;

    let mut i = 0;
    while board.read_reg(wait_reg) != wait_signal {
        match board.step() {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to step board");
                assert!(false);
                return Err(e);
            }
        }
        i += 1;
        if i > 100_000 {
            return Err("Board setup exceeded default iterations".to_string());
        }
    }

    return Ok(board);
}
