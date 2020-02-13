use crate::server::packet::{hex_to_word, word_to_hex, build_reply};
use std::collections::HashSet;
use std::env;
use std::ffi::{OsString};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, Read};
use std::net::{TcpStream, TcpListener, Shutdown};
use std::path::{PathBuf, Path};
use std::vec;

use crate::Board;

mod packet;
use packet::read_packet;

pub fn start_server() {
    let port: String = get_tcp_port().expect("must provide TCP port");

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            return;
        }
    };

    match listener.accept() {
        Ok((socket, addr)) => {
            handle_client(socket);
        }
        Err(e) => {
            return;
        }
    }
}

fn get_tcp_port() -> Option<String> {
    let args: Vec<OsString> = env::args_os().collect();
    for arg in args {
        if arg.to_str().expect("").starts_with("tcp::") {
            let port = &arg.to_str().unwrap()[5..];
            return Some(String::from(port));
        }
    };
    return None;
}

fn get_elf_file_path() -> Option<PathBuf> {
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg == "-kernel" {
            let path = args.next()?;
            return Some(PathBuf::from(&path));
        }
    }
    return None;
}

fn parse_read_memory(mut data: &[u8]) -> Result<(u32, u32), ()> {
    data = &data[1..]; // remove "m"
    let mut parts = data.split(|c| *c == b',');

    let addr = parts.next().unwrap();
    let length = parts.next().unwrap();
    if !parts.next().is_none() {
        return Err(());
    }

    return Ok((hex_to_word(addr)?, hex_to_word(length)?));
}

fn handle_client(mut stream: TcpStream) {
    stream.set_nodelay(true).expect("cannot set no delay");
    let mut breakpoints: HashSet<u32> = HashSet::new();

    let mut board = Board::new();
    board.spawn_audio();

    match get_elf_file_path() {
        Some(p) => {
            board.load_elf_from_path(&p).expect("failed to load elf file");
        },
        None => {
            return;
        }
    }

    let mut data = [0 as u8; 2048];
    while match stream.read(&mut data) {
        Ok(size) => {
            if size <= 1 {
                // interrupt 0x03 or + or -, etc
            } else {
                let pack = match read_packet(&data[0..size]) {
                    Ok(p) => p,
                    Err(e) => {
                        return;
                    }
                };

                println!("received {:?}", std::str::from_utf8(pack.data.as_ref()));

                let out: Vec<u8> = if pack.data.starts_with(b"qSupported") {
                     build_reply(b"PacketSize=2048")
                } else if pack.data == b"!" || pack.data == b"Hg0" || pack.data.starts_with(b"Hc") || pack.data == b"qSymbol::"{
                    build_reply(b"OK")
                } else if pack.data == b"qTStatus" {
                    build_reply(b"T0")
                } else if pack.data.starts_with(b"v") || pack.data == b"qTfV" || pack.data == b"qTfP" {
                    build_reply(b"")
                } else if pack.data == b"?" {
                    build_reply(b"S05")
                } else if pack.data == b"qfThreadInfo" {
                    build_reply(b"m0")
                } else if pack.data == b"qsThreadInfo" {
                    build_reply(b"l")
                } else if pack.data == b"qC" {
                    build_reply(b"QC0")
                } else if pack.data == b"qAttached" {
                    build_reply(b"0")
                } else if pack.data == b"qOffsets" {
                    build_reply(b"Text=0;Data=0;Bss=0")
                } else if pack.data == b"g" {
                    let mut vals = String::new();
                    for i in 0..=14u32 {
                        let rval = board.read_reg(i).swap_bytes();
                        vals += &word_to_hex(rval);
                    }
                    build_reply(vals.as_ref())
                } else if pack.data.starts_with(b"c") {
                    // HACK: Really, the board should be running on a separate thread to
                    //       the TCP handler. However, right now we just intermittently
                    //       check the stream for the interupt. Bigger the skip size ->
                    //       fewer times we check -> faster emulation -> more latency
                    //       in the interrupt.
                    stream.set_nonblocking(true).expect("set_nonblocking call failed");
                    stream.write(b"+").unwrap();

                    while !breakpoints.contains(&board.cpu.read_instruction_pc()) {
                        match stream.read(&mut data) {
                            Ok(size) => {
                                if size == 1 && data[0] == 0x03 {
                                    println!("received interrupt");
                                    break;
                                } else {
                                    println!("got {:?}", &data[0..size]);
                                }
                            },
                            Err(_) => {}
                        };
                        for _ in 0..128 {
                            if !breakpoints.contains(&board.cpu.read_instruction_pc()) {
                                board.step().expect("failed to step board emulation");
                            }
                        }
                    }

                    stream.set_nonblocking(false).expect("set_nonblocking call failed");
                    build_reply(b"S05")
                } else if pack.data.starts_with(b"s") {
                    board.step().expect("failed to step board emulation");

                    match hex_to_word(&pack.data[1..]) {
                        Ok(addr) => {
                            while board.cpu.read_instruction_pc() != addr {
                                board.step().expect("failed to step board emulation");
                            }
                        },
                        Err(_) => {}
                    }

                    build_reply(b"S05")
                } else if pack.data.starts_with(b"Z") {
                    match pack.data.get(1) {
                        Some(b'0') => {
                            let addr = pack.data[3..].split(|c| *c == b',').next().expect("expected ',' in Z0 packet");
                            breakpoints.insert(hex_to_word(addr).expect("failed to read hex address in Z0 packet"));
                            build_reply(b"OK")
                        },
                        Some(_) | None => {
                            build_reply(b"")
                        },
                    }
                } else if pack.data.starts_with(b"z") {
                    match pack.data.get(1) {
                        Some(b'0') => {
                            let addr = pack.data[3..].split(|c| *c == b',').next().expect("expected ',' in Z0 packet");
                            breakpoints.remove(&hex_to_word(addr).expect("failed to read hex address in Z0 packet"));
                            build_reply(b"OK")
                        },
                        Some(_) | None => {
                            build_reply(b"")
                        },
                    }
                } else if pack.data.starts_with(b"p") {
                    // read register X where request is pX

                    let num = &pack.data[1..];
                    let k = hex_to_word(num).expect("failed to read hex register in p packet");
                    let rval = if k == 15 {
                        board.cpu.read_instruction_pc().swap_bytes()
                    } else if k < 15 {
                        board.read_reg(k).swap_bytes()
                    } else {
                        0
                    };

                    build_reply(word_to_hex(rval).as_bytes())
                } else if pack.data.starts_with(b"m") {
                    let (start, length) = parse_read_memory(&pack.data).unwrap();
                    let vals = board.read_memory_region(start, length).unwrap();

                    let mut strs: Vec<u8> = Vec::new();
                    for val in vals {
                        strs.extend(format!("{:02x}", val).bytes());
                    }

                    build_reply(strs.as_slice())
                } else {
                    return;
                };

                println!("sending {:?}", std::str::from_utf8(out.as_ref()));
                stream.write(out.as_ref()).unwrap();
            }
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {};
}
