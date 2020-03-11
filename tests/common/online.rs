#[allow(dead_code)]

use disco_emulator::Board;
use std::path::Path;
use std::process::{Child};
use std::io::{BufRead, BufReader, Write};

use crate::common::{spawn_telnet, spawn_openocd_server};

pub struct Online {
    telnet: Child,
    openocd: Child,
}

impl Online {
    pub fn new(elf_path: &Path) -> Online {
        let openocd = spawn_openocd_server(elf_path).unwrap();
        let telnet = spawn_telnet().unwrap();
        return Online {
            openocd,
            telnet,
        };
    }

    pub fn exec_telnet(&mut self, command: &str) {
        let telnet_stdin = self.telnet.stdin.as_mut().unwrap();
        telnet_stdin.write(format!("{}\n", command).as_bytes()).unwrap();
    }

    pub fn read_telnet_line(&mut self) -> String {
        let mut telnet_stdout = BufReader::new(self.telnet.stdout.as_mut().unwrap());
        let mut line = String::new();
        telnet_stdout.read_line(&mut line).unwrap();
        return line;
    }

    pub fn read_until(&mut self, start: &str) -> Result<String, String> {
        for _ in 0..30 {
            let line =  self.read_telnet_line();
            write!(std::io::stdout(), ">>> {}", line);
            if line.starts_with(start) {
                return Ok(line);
            } else if line.starts_with("^error") {
                return Err(line);
            }
        }
        return Err("too many lines".to_string());
    }

    pub fn step(&mut self) {
        self.exec_telnet("step");
        self.exec_telnet("reg");
        self.read_until("halted:").unwrap();
    }

    pub fn get_registers(&mut self) -> [u32; 16] {
        self.exec_telnet("reg");
        let _response = self.read_until("\n").unwrap();

        // if !response.starts_with("^done,register-values=") {
        //     panic!("unexpected registers response");
        // }
        //
        // let values = &response[24..response.len() - 3];
        //
        // let mut split = values.split("},{");
        // let mut values = [0u32; 16];
        //
        //
        // for i in 0..=15 {
        //     let reg_str = split.next().unwrap();
        //     let val_str = &reg_str[format!("number=\"{}\",value=\"0x", i).len().. reg_str.len() - 1];
        //     let val = u32::from_str_radix(val_str, 16).unwrap();
        //     values[i] = val;
        // }

        return [0; 16];
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
        // write!(self.telnet.stdin.as_mut().unwrap(), "shutdown\n").unwrap();
        // write!(self.telnet.stdin.as_mut().unwrap(), "exit\n").unwrap();

        // write!(std::io::stdout(), "waiting for telnet to exit...");
        // self.telnet.wait().unwrap();
        //
        // write!(std::io::stdout(), "waiting for openocd to exit...");
        // self.openocd.wait().unwrap();

        println!("done");
    }
}
