#[allow(dead_code)]

use disco_emulator::Board;
use std::path::Path;
use std::process::{Child};
use std::io::{BufRead, BufReader, Write};

use crate::common::{spawn_gdb, spawn_openocd_server};

pub struct Online {
    gdb: Child,
    openocd: Child,
}

impl Online {
    pub fn new(elf_path: &Path, port: usize) -> Online {
        let openocd = spawn_openocd_server(port).unwrap();
        let gdb = spawn_gdb(&elf_path, port).unwrap();

        let mut online = Online {
            openocd,
            gdb,
        };

        // upload_via_openocd(&elf_path).unwrap();

        println!("reseting monitor");
        online.exec_gdb("interpreter console \"monitor reset halt\"");
        println!("reset monitor");

        online.exec_gdb("-target-download");
        online.read_until("^done").unwrap();

        return online;
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
            println!(">>> {}", line);
            if line.starts_with(start) {
                return Ok(line);
            } else if line.starts_with("^error") {
                return Err(line);
            }
        }
        return Err("too many lines".to_string());
    }

    pub fn step(&mut self) {
        self.exec_gdb("-exec-step-instruction 1");
        self.read_until("*stopped").unwrap();
    }

    pub fn get_registers(&mut self) -> [u32; 16] {
        self.exec_gdb("-data-list-register-values x [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]");
        let response = self.read_until("^done").unwrap();

        if !response.starts_with("^done,register-values=") {
            panic!("unexpected registers response");
        }

        let values = &response[24..response.len() - 3];

        let mut split = values.split("},{");
        let mut values = [0u32; 16];


        for i in 0..=15 {
            let reg_str = split.next().unwrap();
            let val_str = &reg_str[format!("number=\"{}\",value=\"0x", i).len().. reg_str.len() - 1];
            let val = u32::from_str_radix(val_str, 16).unwrap();
            values[i] = val;
        }

        return values;
    }

    pub fn verify_state(&mut self, board: &Board) -> Result<(), String> {
        let registers = self.get_registers();

        for i in 0..=14u32 {
            if board.read_reg(i) != registers[i as usize] {
                return Err(format!("Register {} not matching: real=0x{:08X}, emulator=0x{:08X}", i, registers[i as usize], board.read_reg(i)));
            }
        }

        if board.cpu.read_instruction_pc() != registers[15] {
            return Err(format!("PC not matching: real=0x{:08X}, emulator=0x{:08X}", registers[15], board.cpu.read_instruction_pc()));
        }

        return Ok(());
    }

    pub fn close(&mut self) {
        write!(self.gdb.stdin.as_mut().unwrap(), "-gdb-exit\n").unwrap();

        println!("waiting for gdb to exit...");
        self.gdb.wait().unwrap();

        if let Err(e) = self.openocd.kill() {
            eprintln!("Failed to kill openocd: {}", e);
        }

        println!("done");
    }
}
//
// impl Drop for Online {
//     fn drop(&mut self) {
//         if let Err(e) = self.gdb.kill() {
//             eprintln!("Failed to kill gdb: {}", e);
//         }
//         if let Err(e) = self.openocd.kill() {
//             eprintln!("Failed to kill openocd: {}", e);
//         }
//     }
// }
