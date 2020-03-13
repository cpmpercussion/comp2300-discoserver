#[allow(dead_code)]

use std::process::ExitStatus;
use disco_emulator::Board;

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Child};

pub mod online;

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
    return Ok(path);
}

pub fn compile_program(src_path: &Path, linker_path: &Path) -> Result<PathBuf, String> {
    let mut elf_path = PathBuf::from(&src_path);
    elf_path.push("firmware.elf");

    let mut asm_child = Command::new("arm-none-eabi-as")
                                .arg("-g")
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

pub fn get_default_linker() -> Result<PathBuf, String> {
    let mut path = get_tests_path()?;
    path.push("fixtures");
    path.push("common");
    path.push("linker.ld");
    return Ok(path);
}

pub fn load_program(name: &str) -> Result<Board, String> {
    let mut path = get_tests_path()?;
    path.push("fixtures");

    let mut linker_path = PathBuf::from(&path);
    linker_path.push("common");
    linker_path.push("linker.ld");
    println!("linker: {:?}", linker_path);

    let mut src_path = PathBuf::from(&path);
    src_path.push("offline");
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

fn get_openocd_config_path() -> Result<PathBuf, String> {
    let mut tests = get_tests_path()?;
    tests.push("fixtures");
    tests.push("common");
    tests.push("board_config.cfg");
    return Ok(tests);
}

pub fn get_online_src_path(folder: &str) -> Result<PathBuf, String> {
    let mut tests = get_tests_path()?;
    tests.push("fixtures");
    tests.push("online");
    tests.push(folder);
    return Ok(tests);
}

pub fn spawn_openocd_server(elf_path: &Path) -> Result<Child, String> {
    let openocd = Command::new("openocd")
                        .arg("-f")
                        .arg(get_openocd_config_path()?)
                        .arg("-c")
                        .arg(format!("init; program {}; reset init", elf_path.to_str().unwrap()))
                        // .stdin(std::process::Stdio::piped())
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn().expect("failed to spawn openocd; is it on your PATH?");

    return Ok(openocd);
    // TODO: Get stderr initially, but redirect to null after initialisation.
    //       Otherwise buffer fills up because we don't read from it and we get
    //       "mystery" hang.

    // let mut child_err = BufReader::new(openocd.stderr.as_mut().unwrap());
    // let mut line = String::new();
    // for _ in 0..30 {
    //     line.clear();
    //     child_err.read_line(&mut line).unwrap();
    //     write!(std::io::stdout(), "> {}", line);
    //
    //     if line == "Info : Listening on port 4444 for telnet connections\n" {
    //         // return Err(format!("Jmm"));
    //         return Ok(openocd);
    //     }
    // }
    //
    // openocd.kill().unwrap();
    // return Err("failed to spawn openocd properly".to_string());
}

pub fn upload_via_openocd(elf_path: &Path) -> Result<ExitStatus, String> {
    return Ok(Command::new("openocd")
                        .arg("-f")
                        .arg(get_openocd_config_path()?)
                        .arg("-c")
                        .arg(format!("program {} verify reset exit", elf_path.to_str().unwrap()))
                        // .stdin(std::process::Stdio::piped())
                        // .stdout(std::process::Stdio::piped())
                        // .stderr(std::process::Stdio::piped())
                        .spawn().expect("failed to spawn openocd; is it on your PATH?")
                        .wait().unwrap());
}

pub fn spawn_gdb(elf_path: &Path, port: usize) -> Result<Child, String> {
    let mut gdb = Command::new("arm-none-eabi-gdb")
                        .arg("--nx")
                        .arg("--quiet")
                        .arg("--interpreter=mi2")
                        .arg(elf_path)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn().expect("failed to spawn arm-none-eabi-gdb; is it on your PATH?");

    let gdb_stdin = gdb.stdin.as_mut().unwrap();
    gdb_stdin.write(format!("-target-select remote localhost:{}\n", port).as_bytes()).unwrap();

    let mut child_err = BufReader::new(gdb.stdout.as_mut().unwrap());
    let mut line = String::new();
    for _ in 0..5 {
        line.clear();
        child_err.read_line(&mut line).unwrap();
        println!(">> {}", line);
    }

    return Ok(gdb);
}
