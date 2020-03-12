#[allow(dead_code)]

use disco_emulator::Board;
use std::path::Path;
use std::process::{Child};
use std::io::{Read, Write};
use std::net::{TcpStream, Shutdown};

use crate::common::{spawn_openocd_server};

pub struct Online {
    tcl: TcpStream,
    tcp_buffer: Box<[u8]>,
    openocd: Child,
}

impl Online {
    pub fn new(elf_path: &Path) -> Result<Online, String> {
        if let Ok(mut tcl) = TcpStream::connect("127.0.0.1:6666") {
            write!(std::io::stdout(), "Shutting down previous openocd server...\n").unwrap();
            tcl.write("shutdown".as_bytes()).unwrap();
            tcl.write(&[0x1Au8]).unwrap();
            tcl.shutdown(Shutdown::Both).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let openocd = spawn_openocd_server(elf_path).unwrap();

        // HACK: Wait for openOCD to init, as we can't detect when it's ready.
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let tcl = match TcpStream::connect_timeout(&"127.0.0.1:6666".parse().unwrap(), std::time::Duration::from_millis(100)) {
            Ok(s) => s,
            Err(e) => {
                println!("Error connecting TCL TcpStream: {}\n", e);
                assert!(false);
                return Err(e.to_string());
            }
        };

        return Ok(Online {
            tcl,
            tcp_buffer: vec![0; 1024].into_boxed_slice(),
            openocd,
        });
    }

    pub fn exec_tcl(&mut self, command: &str) {
        self.tcl.write(format!("{}", command).as_bytes()).unwrap();
        self.tcl.write(&[0x1Au8]).unwrap();
    }

    pub fn read_tcl_line(&mut self) -> Result<Vec<u8>, String> {
        let mut input: Vec<u8> = Vec::new();
        loop {
            let size = match self.tcl.read(&mut self.tcp_buffer) {
                Ok(s) => s,
                Err(e) => {
                    return Err(format!("failed to read from tcp stream: {}", e));
                }
            };

            for &c in &self.tcp_buffer[0..size] {
                if c == 0x1A {
                    return Ok(input);
                } else {
                    input.push(c);
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.exec_tcl("step");
        self.read_tcl_line().unwrap();
    }

    pub fn get_registers(&mut self) -> [u32; 17] {
        self.exec_tcl("reg");
        let line = self.read_tcl_line().unwrap();

        let response = std::str::from_utf8(line.as_ref()).unwrap();

        if !response.starts_with("===== arm v7m registers") {
            panic!("unexpected registers response");
        }

        let mut split = response.split("\n");
        split.next(); // skip header

        let mut registers = [0; 17];
        let mut index = 0;
        while let Some(mut l) = split.next() {
            if l.starts_with("(17)") {
                break;
            }
            if l.ends_with(" (dirty)") {
                l = &l[..l.len() - 8];
            }

            let value = u32::from_str_radix(&l[l.len() - 8..], 16).unwrap();
            registers[index] = value;
            index += 1;
        }
        return registers;
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

        if board.cpu.read_xpsr() != registers[16] {
            return Err(format!("xPSR not matching: real=0x{:08X}, emulator=0x{:08X}", registers[16], board.cpu.read_xpsr()));
        }

        return Ok(());
    }

    pub fn close(&mut self) {
        self.exec_tcl("shutdown");
        self.tcl.shutdown(Shutdown::Both).unwrap();
        write!(std::io::stdout(), "waiting for openocd to exit...\n").unwrap();
        self.openocd.wait().unwrap();
        write!(std::io::stdout(), "closed\n").unwrap();
    }
}
