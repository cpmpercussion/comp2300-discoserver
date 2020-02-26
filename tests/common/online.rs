#[allow(dead_code)]

use crate::common::upload_via_openocd;
use disco_emulator::Board;
use std::path::Path;
use crate::common::spawn_gdb;
use crate::common::spawn_openocd_server;
use std::process::{Child};
use std::io::{BufRead, BufReader, Write};


pub struct Online {
    gdb: Child,
    openocd: Child,
}

impl Online {
    pub fn new(elf_path: &Path, port: usize) -> Online {
        // if !upload_via_openocd(elf_path).unwrap().success() {
        //     panic!("Failed to upload to board");
        // };

        let openocd = spawn_openocd_server(port).unwrap();
        let gdb = spawn_gdb(&elf_path, port).unwrap();

        return Online {
            openocd,
            gdb,
        };
    }

    pub fn exec_gdb(&mut self, command: &str) {
        let gdb_stdin = self.gdb.stdin.as_mut().unwrap();
        gdb_stdin.write(format!("{}\n", command).as_bytes()).unwrap();
    }

    pub fn read_gdb_line(&mut self) -> String {
        let mut gdb_stdout = BufReader::new(self.gdb.stdout.as_mut().unwrap());
        let mut line = String::new();
        gdb_stdout.read_line(&mut line).unwrap();
        return line;
    }

    pub fn read_until(&mut self, start: &str) -> Result<String, String> {
        for _ in 0..15 {
            let line =  self.read_gdb_line();
            println!(">> {}", line);
            if line.starts_with(start) {
                return Ok(line);
            } else if line.starts_with("^error") {
                return Err(line);
            }
        }
        return Err("too many lines".to_string());
    }

    pub fn step(&mut self) {
        self.exec_gdb("-exec-step-instruction");
        self.read_until("^running").unwrap();
    }

    pub fn get_registers(&mut self) -> [u32; 16] {
        return [0u32; 16];
    }

    pub fn verify_state(&mut self, board: &Board) -> Result<(), String> {
        let registers = self.get_registers();

        for i in 0..=14u32 {
            if board.read_reg(i) != registers[i as usize] {
                return Err(format!("Register {} not matching: real={:08X}, emulator={:08X}", i, registers[i as usize], board.read_reg(i)));
            }
        }

        if board.cpu.read_instruction_pc() != registers[15] {
            return Err(format!("PC not matching: real={:08X}, emulator={:08X}", registers[15], board.cpu.read_instruction_pc()));
        }

        return Ok(());
    }

    pub fn close(&mut self) {
        write!(self.gdb.stdin.as_mut().unwrap(), "-gdb-exit\n").unwrap();

        println!("waiting for gdb to exit...");
        self.gdb.wait().unwrap();
        println!("done");
    }
}

impl Drop for Online {
    fn drop(&mut self) {
        if let Err(e) = self.gdb.kill() {
            eprintln!("Failed to kill gdb: {}", e);
        }
        if let Err(e) = self.openocd.kill() {
            eprintln!("Failed to kill openocd: {}", e);
        }
    }
}
