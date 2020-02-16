mod packet;
use packet::{read_packet, hex_to_word, word_to_hex, build_reply, validate_packet, Packet};

mod query;
use query::{Query, Set};


use std::collections::{HashSet, VecDeque};
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

#[derive(Debug)]
enum PacketState {
    Start,
    Data,
    Checksum,
}

#[derive(Debug)]
enum Signal {

}

#[derive(Debug)]
enum BreakpointType {
    Hardware,
    Software,
    WriteWatchpoint,
    ReadWatchpoint,
    AccessWatchpoint,
}

#[derive(Debug)]
enum BreakpointKind {
    Thumb16Bit,
    Thumb32Bit,
    Arm32Bit,
}

#[derive(Debug)]
enum Request {
    Unhandled,
    Interrupt,
    Acknowledge,
    AcknowledgeFailure,
    EnableExtendedMode,
    IndicateHaltReason,
    InitializeArgv { args: Vec<(u32, u32, Vec<u8>)> },
    EditBreakpointDeprecated { address: u32, set: bool },
    BackwardsContinue,
    BackwardsSingleStep,
    Continue { address: Option<u32>, signal: Option<Signal> },
    ToggleDebug,
    Detach { pid: Option<u32> },
    ReadRegisters,
    WriteRegisters { values: Vec<u32> },
    SetThreadSupport { values: Vec<u8> }, // deprecated over vCont
    StepClockCycle { address: Option<u32>, count: u32 },
    Kill,
    ReadMemory { address: u32, length: u32 },
    WriteMemory { address: u32, length: u32, bytes: Vec<u8> },
    ReadRegister { number: u32 },
    WriteRegister { number: u32, value: u32 },
    Query { query: Query },
    Set { set: Set },
    ResetSystem,
    RestartProgram,
    SingleStep { address: Option<u32>, signal: Option<Signal> },
    SearchBackwards { address: u32, pattern: u32, mask: u32 },
    ThreadAlive { id: i32 },
    MustReplyEmpty,
    UnknownVPacket,
    EditBreakpoint { address: u32, set: bool, btype: BreakpointType, kind: BreakpointKind },
}

fn is_hex_char(c: u8) -> bool {
    return match c {
        b'0'..=b'9' => true,
        b'a'..=b'f' => true,
        b'A'..=b'F' => true,
        _ => false,
    };
}

// Takes a ASCII hex number [0-9a-fA-F] and returns the value as a u8
fn hex_to_byte(c: u8) -> Result<u8, ()> {
    return match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(()),
    };
}

fn get_u8_from_hex(hex: (u8, u8)) -> Result<u8, ()> {
    if let (Ok(b1), Ok(b2)) = (hex_to_byte(hex.0), hex_to_byte(hex.1)) {
        return Ok(b1 * 16 + b2);
    } else {
        return Err(());
    }
}

struct GdbServer {
    stream: TcpStream,
    tcp_buffer: Box<[u8]>,
    packet_builder: Vec<u8>,
    packet_builder_state: PacketState,
    packet_checksum: Option<u8>,
    acknowledge: bool,
    packets: VecDeque<(Vec<u8>, bool)>,
    board: Board,
    hw_breakpoints: HashSet<u32>,
}

impl GdbServer {
    fn new(stream: TcpStream, buffer_size: usize) -> GdbServer {
        return GdbServer {
            stream: stream,
            tcp_buffer: vec![0; buffer_size].into_boxed_slice(),
            packet_builder: Vec::new(),
            packet_builder_state: PacketState::Start,
            packet_checksum: None,
            acknowledge: true,
            packets: VecDeque::new(),
            board: Board::new(),
            hw_breakpoints: HashSet::new(),
        }
    }

    fn run(&mut self, audio: bool) -> Result<(), ()> {
        if let Err(e) = self.stream.set_nodelay(true) {
            println!("cannot set no delay on TCP stream: {}", e);
        };

        if let Some(path) = get_elf_file_path() {
            self.board.load_elf_from_path(&path).expect("failed to load from ELF file");
        } else {
            println!("ELF file path not provided");
        }

        if audio {
            self.board.spawn_audio();
        }

        loop {
            let request = match self.receive_request() {
                Ok(r) => r,
                Err(_) => {
                    println!("failed to read request");
                    return Err(());
                }
            };

            println!("received request: {:?}", request);

            match request {
                Request::Acknowledge => {/* do nothing*/},
                Request::AcknowledgeFailure => {/* do nothing*/},
                Request::MustReplyEmpty => {
                    self.send_reply_empty();
                }
                Request::EnableExtendedMode => {
                    self.send_reply_ok();
                }
                Request::SetThreadSupport {..} => {
                    self.send_reply_ok();
                }
                Request::SingleStep { address, .. /*signal*/ } => {
                    if let Some(a) = address {
                        self.board.cpu.write_instruction_pc(a);
                    }
                    self.board.step().expect("failed to step board emulation");
                    self.send_reply(b"S05");
                }
                Request::Continue { address, .. /*signal*/ } => {
                    if let Some(a) = address {
                        self.board.cpu.write_instruction_pc(a);
                    }

                    // HACK: Really, the board should be running on a separate thread to
                    //       the TCP handler. However, right now we just intermittently
                    //       check the stream for the interupt. Bigger the skip size ->
                    //       fewer times we check -> faster emulation -> more latency
                    //       in the interrupt.
                    self.stream.set_nonblocking(true).expect("set_nonblocking call failed");
                    self.send_acknowledge();
                    while !self.hw_breakpoints.contains(&self.board.cpu.read_instruction_pc()) {
                        match self.stream.read(&mut self.tcp_buffer) {
                            Ok(size) => {
                                if size == 1 && self.tcp_buffer[0] == 0x03 {
                                    println!("received interrupt");
                                    break;
                                } else {
                                    println!("got unexpected {:?}", &self.tcp_buffer[0..size]);
                                }
                            },
                            Err(_) => {}
                        };
                        for _ in 0..128 {
                            if !self.hw_breakpoints.contains(&self.board.cpu.read_instruction_pc()) {
                                self.board.step().expect("failed to step board emulation");
                            } else {
                                break;
                            }
                        }
                    }

                    self.stream.set_nonblocking(false).expect("set_nonblocking call failed");
                    self.send_reply(b"S05");
                }
                Request::IndicateHaltReason => {
                    self.send_reply(b"S05");
                }
                Request::EditBreakpoint {address, set, .. } => {
                    if set {
                        self.hw_breakpoints.insert(address);
                    } else {
                        self.hw_breakpoints.remove(&address);
                    }
                    self.send_reply_ok();
                }
                Request::ReadRegisters => {
                    let mut vals = String::new();
                    for i in 0..=14u32 {
                        let rval = self.board.read_reg(i);
                        vals += &word_to_hex(rval.swap_bytes());
                    }
                    vals += &word_to_hex(self.board.cpu.read_instruction_pc().swap_bytes());
                    self.send_reply(vals.as_bytes());
                }
                Request::ReadRegister { number } => {
                    let value = match number {
                        0..=14u32 => self.board.read_reg(number),
                        15u32 => self.board.cpu.read_instruction_pc(),
                        16..=23u32 => 0xDEADBEEF, // F0..7 probably
                        25u32 => self.board.cpu.read_xpsr(),
                        _ => {
                            println!("Unknown register number");
                            0x0
                        }
                    };
                    self.send_reply(word_to_hex(value.swap_bytes()).as_bytes());
                }
                Request::ReadMemory { address, length } => {
                    let vals = self.board.read_memory_region(address, length).expect("cannot read board memory region");
                    let mut strs: Vec<u8> = Vec::new();
                    for val in vals {
                        strs.extend(format!("{:02x}", val).bytes());
                    }
                    self.send_reply(strs.as_slice());
                }
                Request::WriteMemory { address, length, bytes } => {
                    self.send_reply_empty();
                }
                Request::Query { query } => {
                    match query {
                        Query::CurrentThread => {
                            self.send_reply(b"QC0");
                        }
                        Query::SectionOffsets => {
                            self.send_reply(b"Text=0;Data=0;Bss=0");
                        }
                        Query::ExecCommand { command } => {
                            println!("executing {:?}", std::str::from_utf8(&command));
                            self.send_reply_ok();
                        }
                        Query::Supported {..} => {
                            let m = format!("PacketSize={:X?};QStartNoAckMode+", self.tcp_buffer.len());
                            self.send_reply(m.as_ref());
                        }
                        Query::TracepointStatus => {
                            self.send_reply(b"T0"); // no trace running
                        }
                        Query::ThreadInfoFirst => {
                            self.send_reply(b"m0");
                        }
                        Query::ThreadInfoSubsequent => {
                            self.send_reply(b"l");
                        }
                        Query::AttachedToProcess { process } => {
                            match process {
                                Some(_) => {
                                    println!("unhandled multiprocess attached query");
                                }
                                None => {
                                    self.send_reply(b"0");
                                }
                            }
                        }
                        _ => {
                            println!("Unhandled query: {:?}", query);
                            self.send_reply_empty();
                        }
                    }
                },
                Request::Set { set } => {
                    match set {
                        Set::NoAcknowledgmentMode => {
                            self.send_reply_ok();
                            self.acknowledge = false;
                        }
                        _ => {
                            println!("Unhandled set: {:?}", set);
                        }
                    }
                }
                _ => {
                    println!("Unhandled request: {:?}", request);
                    self.send_reply_empty();
                },
            };
        }
    }

    fn send_reply_empty(&mut self) {
        self.send_reply(b"");
    }

    fn send_reply_ok(&mut self) {
        self.send_reply(b"OK");
    }

    fn send_reply(&mut self, contents: &[u8]) {
        let mut out: Vec<u8> = Vec::new();
        self.send_acknowledge(); // ack request
        out.push(b'$');
        out.extend_from_slice(contents);
        out.push(b'#');
        out.extend_from_slice(get_checksum_hex(&contents).as_bytes());
        println!("sending reply: {:?}", std::str::from_utf8(out.as_ref()));
        self.stream.write(out.as_ref()).expect("failed to send message");
    }

    fn send_acknowledge(&mut self) {
        if self.acknowledge {
            self.stream.write(b"+").expect("failed to send acknowledgement");
        }
    }

    // Returns a fully formed instruction from GDB
    fn receive_request(&mut self) -> Result<Request, ()> {
        // println!("receiving request...");
        let (packet, acknowledge) = match self.receive_packet() {
            Ok(r) => r,
            Err(e) => {
                println!("failed to receive packet");
                return Err(());
            }
        };
        if acknowledge {
            return Ok(match packet[0] {
                0x03 => Request::Interrupt,
                b'+' => Request::Acknowledge,
                b'-' => Request::AcknowledgeFailure,
                _ => {
                    println!("unexpected single char packet: {:?}", packet[0]);
                    return Err(());
                }
            });
        }

        if packet.len() == 0 {
            println!("unexpected empty packet");
            return Err(());
        }

        let single = packet.len() == 1;
        return match packet[0] {
            b'!' if single => Ok(Request::EnableExtendedMode),
            b'?' if single => Ok(Request::IndicateHaltReason),
            b'A' => Ok(Request::Unhandled), // InitializeArgv
            b'b' => Ok(Request::Unhandled), // Baud rate & Backwards continue / single step
            b'B' => Ok(Request::Unhandled), // EditBreakpointDeprecated
            b'c' | b'C' => self.parse_continue(&packet),
            b'd' => Ok(Request::Unhandled), // toggle debug flag
            b'D' => Ok(Request::Unhandled), // detach
            b'F' => Ok(Request::Unhandled), // reply from GDB from 'F' request
            b'g' if single => Ok(Request::ReadRegisters),
            b'G' => self.parse_write_registers(&packet),
            b'H' => self.parse_thread_operator(&packet), // thread operation support
            b'i' => Ok(Request::Unhandled), // step clock cycle
            b'I' if single => Ok(Request::Unhandled), // signal & step clock cycle
            b'k' if single => Ok(Request::Kill),
            b'm' => self.parse_read_address(&packet),
            b'M' => self.parse_write_address(&packet),
            b'p' => self.parse_read_register(&packet),
            b'P' => self.parse_write_register(&packet),
            b'q' => self.parse_query(&packet),
            b'Q' => self.parse_set(&packet),
            b'r' if single => Ok(Request::ResetSystem),
            b'R' if single => Ok(Request::RestartProgram),
            b's' | b'S' => self.parse_step(&packet),
            b't' => Ok(Request::Unhandled), // backwards search
            b'T' => Ok(Request::Unhandled), // thread alive
            b'v' => self.parse_v_packet(&packet),
            b'X' => Ok(Request::Unhandled), // write memory
            b'z' | b'Z' => self.parse_edit_breakpoint(&packet), // edit breakpoint
            _ => Ok(Request::Unhandled), // whatever falls through
        };
    }

    fn parse_continue(&mut self, packet: &[u8]) -> Result<Request, ()> {
        // TODO: Support reading optional params
        return Ok(Request::Continue {
            address: None,
            signal: None,
        });
    }

    fn parse_step(&mut self, packet: &[u8]) -> Result<Request, ()> {
        // TODO: Support reading optional params
        return Ok(Request::SingleStep {
            address: None,
            signal: None,
        });
    }

    fn parse_write_registers(&mut self, packet: &[u8]) -> Result<Request, ()> {
        return Err(());
    }

    fn parse_edit_breakpoint(&mut self, packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'z' || packet[0] == b'Z');
        let mut iter = packet[1..].split(|&c| c == b',' || c == b';');

        match (iter.next(), iter.next(), iter.next()) {
            (Some(t), Some(addr), Some(k)) => {
                return Ok(Request::EditBreakpoint {
                    address: hex_to_word(addr)?,
                    set: packet[0] == b'Z',
                    btype: match t {
                        b"0" => BreakpointType::Hardware,
                        b"1" => BreakpointType::Software,
                        b"2" => BreakpointType::WriteWatchpoint,
                        b"3" => BreakpointType::ReadWatchpoint,
                        b"4" => BreakpointType::AccessWatchpoint,
                        _ => {
                            println!("unrecognised breakpoint type");
                            return Err(());
                        }
                    },
                    kind: match k {
                        b"2" => BreakpointKind::Thumb16Bit,
                        b"3" => BreakpointKind::Thumb32Bit,
                        b"4" => BreakpointKind::Arm32Bit,
                        _ => {
                            println!("unrecognised breakpoint kind");
                            return Err(());
                        }
                    },
                });
            }
            _ => {
                println!("failed to parse breakpoint edit");
                return Err(());
            }
        }
    }

    fn parse_read_address(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'm');
        packet = &packet[1..];
        let mut iter = packet.split(|&c| c == b',');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(a), Some(l), None) => {
                return Ok(Request::ReadMemory {
                    address: hex_to_word(a)?,
                    length: hex_to_word(l)?,
                });
            }
            _ => {
                println!("invalid read memory instruction");
                return Err(());
            }
        }
    }

    fn parse_write_address(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'M');
        println!("parsing write address");
        packet = &packet[1..];
        let mut iter = packet.split(|&c| c == b',' || c == b':');
        match (iter.next(), iter.next(), iter.next(), iter.next()) {
            (Some(a), Some(l), Some(b), None) => {
                return Ok(Request::WriteMemory {
                    address: hex_to_word(a)?,
                    length: hex_to_word(l)?,
                    bytes: Vec::new(),
                    // bytes: parse_hex_bytes(b)?,
                });
            }
            _ => {
                println!("invalid write memory instruction");
                return Err(());
            }
        }
    }

    fn parse_read_register(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'p');
        packet = &packet[1..];
        match hex_to_word(&packet) {
            Ok(i) => {
                return Ok(Request::ReadRegister { number: i });
            },
            Err(_) => {
                println!("failed to read register index");
                return Err(());
            }
        }
    }

    fn parse_write_register(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'P');
        packet = &packet[1..];
        let mut iter = packet.split(|&c| c == b'=');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(r), Some(v), None) => {
                return Ok(Request::WriteRegister {
                    number: hex_to_word(r)?,
                    value: hex_to_word(v)?,
                });
            },
            _ => {
                println!("failed to parse write register");
                return Err(());
            }
        }
    }

    fn parse_thread_operator(&mut self, packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'H');
        return Ok(Request::SetThreadSupport { values: packet[1..].to_vec() });
    }

    fn parse_query(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'q');
        packet = &packet[1..];
        let command = leading_alpha(&packet);
        let all = command.len() == packet.len();

        println!("handling {:?}", command);

        return Ok(Request::Query { query: match command {
            b"C" if all => Query::CurrentThread,
            b"Supported" => {
                // we don't really care what it declares
                Query::Supported { features: Vec::new() }
            },
            b"Rcmd" => Query::ExecCommand { command: packet[5..].to_vec() },
            b"Offsets" if all => Query::SectionOffsets,
            b"fThreadInfo" if all => Query::ThreadInfoFirst,
            b"sThreadInfo" if all => Query::ThreadInfoSubsequent,
            b"TStatus" if all => Query::TracepointStatus,
            b"Attached" if all => Query::AttachedToProcess { process: None },
            b"TfP" if all => Query::TracepointPieceFirst,
            b"TsP" if all => Query::TracepointPieceSubsequent,
            b"TfV" if all => Query::TracevariableFirst,
            b"TsV" if all => Query::TracevariableSubsequent,
            _ => {
                return Ok(Request::Unhandled);
            }
        }});
    }

    fn parse_set(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'Q');
        packet = &packet[1..];
        let command = leading_alpha(&packet);
        let all = command.len() == packet.len();

        println!("handling {:?}", command);
        return Ok(Request::Set { set: match command {
            b"StartNoAckMode" if all => Set::NoAcknowledgmentMode,
            _ => {
                println!("unrecognised command: {:?}", std::str::from_utf8(command));
                return Err(());
            }
        }});
    }

    fn parse_v_packet(&mut self, mut packet: &[u8]) -> Result<Request, ()> {
        assert!(packet[0] == b'v');
        // println!("parsing v packet");
        packet = &packet[1..];
        let command = leading_alpha(&packet);
        println!("handling {:?}", command);
        return Ok(match command {
            b"MustReplyEmpty" => Request::MustReplyEmpty,
            _ => Request::Unhandled,
        });
    }

    // Receives a packet from the TCP stream
    fn receive_packet(&mut self) -> Result<(Vec<u8>, bool), ()> {
        loop {
            // println!("receiving packet...");
            if let Some(p) = self.packets.pop_front() {
                return Ok(p);
            }

            // No packets means we don't have enough data for a full one.
            // The only way to get more is reading the stream
            if let Err(e) = self.process_tcp_packet() {
                println!("failed to read packet");
                return Err(());
            };
        }
    }

    fn process_tcp_packet(&mut self) -> Result<(), ()> {
        // println!("reading tcp...");
        let size = match self.stream.read(&mut self.tcp_buffer) {
            Ok(s) => s,
            Err(e) => {
                println!("failed to read from tcp stream: {}", e);
                return Err(());
            }
        };
        println!("TCP: {:?}", std::str::from_utf8(self.tcp_buffer[..size].as_ref()));

        if size == 0 {
            return Ok(());
        }

        for &c in &self.tcp_buffer[0..size] {
            match self.packet_builder_state {
                PacketState::Start => {
                    match c {
                        0x03 | b'+' | b'-' => {
                            self.packets.push_back((vec![c], true));
                        }
                        b'$' => {
                            self.packet_builder_state = PacketState::Data;
                        },
                        _ => {
                            println!("unexpected start of packet");
                            return Err(());
                        }
                    }
                },
                PacketState::Data => {
                    match c {
                        b'$' => {
                            println!("unexpected $ in packet data stream");
                            return Err(());
                        },
                        b'#' => {
                            self.packet_builder_state = PacketState::Checksum;
                        },
                        _ => {
                            self.packet_builder.push(c);
                        }
                    }
                },
                PacketState::Checksum => {
                    match self.packet_checksum {
                        Some(b) => {
                            self.packet_checksum = None;
                            let checksum = match get_u8_from_hex((b, c)) {
                                Ok(s) => s,
                                Err(_) => {
                                    println!("failed to parse checksum");
                                    return Err(());
                                }
                            };
                            if !validate_packet(self.packet_builder.as_ref(), checksum) {
                                println!("checksum check failed");
                                return Err(());
                            }
                            self.packets.push_back((self.packet_builder.clone(), false));
                            self.packet_builder = Vec::new();
                            self.packet_builder_state = PacketState::Start;
                        },
                        None => {
                            self.packet_checksum = Some(c);
                        }
                    }
                }
            }
        }

        return Ok(());
    }
}

fn get_checksum_hex(packet: &[u8]) -> String {
    let mut sum: u8 = 0;
    for &b in packet {
        sum = sum.wrapping_add(b);
    };
    return format!("{:02X?}", sum);
}

pub fn start_server() {
    let port: String = get_tcp_port().expect("must provide TCP port");

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            println!("errr");
            return;
        }
    };

    match listener.accept() {
        Ok((socket, addr)) => {
            println!("connected");
            handle_client(socket);
        }
        Err(e) => {
            return;
        }
    }
}

fn leading_alpha(data: &[u8]) -> &[u8] {
    for i in 0..data.len() {
        match data[i] {
            b'a'..=b'z' | b'A'..=b'Z' => {},
            _ => {
                return &data[0..i];
            }
        }
    }
    return data;
}

enum Support<'a> {
    Yes,
    No,
    Maybe,
    Value(&'a [u8]),
}

fn parse_gdbfeature(feature: &[u8]) -> Result<(&[u8], Support), ()> {
    return Err(());
}

fn get_tcp_port() -> Option<String> {
    let args: Vec<OsString> = env::args_os().collect();
    for arg in args {
        if arg.to_str().expect("").starts_with("tcp::") {
            let port = &arg.to_str().expect("cannot read OS arg as string")[5..];
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

    let addr = parts.next().expect("cannot parse address of read memory");
    let length = parts.next().expect("cannot parse length of read memory");
    if !parts.next().is_none() {
        return Err(());
    }

    return Ok((hex_to_word(addr)?, hex_to_word(length)?));
}

fn parse_hex_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    if data.len() % 2 != 0 {
        return Err(());
    }

    let mut out = Vec::new();
    let mut iter = data.chunks_exact(2);
    while let Some(&[a, b]) = iter.next() {
        out.push(get_u8_from_hex((a, b))?);
    }
    return Ok(out);
}

fn handle_client(stream: TcpStream) {
    let mut server = GdbServer::new(stream, 2048);

    if let Err(e) = server.run(false) {
        println!("server error");
    };

    // stream.set_nodelay(true).expect("cannot set no delay");
    // let mut breakpoints: HashSet<u32> = HashSet::new();
    //
    // let mut board = Board::new();
    // // board.spawn_audio();
    //
    // match get_elf_file_path() {
    //     Some(p) => {
    //         board.load_elf_from_path(&p).expect("failed to load elf file");
    //     },
    //     None => {
    //         return;
    //     }
    // }
    //
    // let mut data = [0 as u8; 2048];
    // while match stream.read(&mut data) {
    //     Ok(size) => {
    //         if size <= 1 {
    //             // interrupt 0x03 or + or -, etc
    //         } else {
    //             let pack = match read_packet(&data[0..size]) {
    //                 Ok(p) => p,
    //                 Err(e) => {
    //                     println!("error reading packet");
    //                     // return;
    //                     // continue;
    //                     Packet::new(b"0", 0)
    //                 }
    //             };
    //
    //             println!("received {:?}", std::str::from_utf8(pack.data.as_ref()));
    //             // println!("received {:?}", std::str::from_utf8(pack.data[0..std::cmp::min(12, pack.data.len())].as_ref()));
    //
    //             let out: Vec<u8> = if pack.data.starts_with(b"qSupported") {
    //                  build_reply(b"PacketSize=2048")
    //             } else if pack.data.starts_with(b"X") || pack.data == b"!" || pack.data == b"Hg0" || pack.data.starts_with(b"Hc") || pack.data == b"qSymbol::"{
    //                 build_reply(b"OK")
    //             } else if pack.data == b"qTStatus" {
    //                 build_reply(b"T0")
    //             } else if pack.data.starts_with(b"v") || pack.data == b"qTfV" || pack.data == b"qTfP" {
    //                 build_reply(b"")
    //             } else if pack.data == b"?" {
    //                 build_reply(b"S05")
    //             } else if pack.data == b"qfThreadInfo" {
    //                 build_reply(b"m0")
    //             } else if pack.data == b"qsThreadInfo" {
    //                 build_reply(b"l")
    //             } else if pack.data == b"qC" {
    //                 build_reply(b"QC0")
    //             } else if pack.data == b"qAttached" {
    //                 build_reply(b"0")
    //             } else if pack.data == b"qOffsets" {
    //                 build_reply(b"Text=0;Data=0;Bss=0")
    //             } else if pack.data == b"g" {
    //                 let mut vals = String::new();
    //                 for i in 0..=14u32 {
    //                     let rval = board.read_reg(i).swap_bytes();
    //                     vals += &word_to_hex(rval);
    //                 }
    //                 build_reply(vals.as_ref())
    //             } else if pack.data.starts_with(b"c") {
    //                 // HACK: Really, the board should be running on a separate thread to
    //                 //       the TCP handler. However, right now we just intermittently
    //                 //       check the stream for the interupt. Bigger the skip size ->
    //                 //       fewer times we check -> faster emulation -> more latency
    //                 //       in the interrupt.
    //                 stream.set_nonblocking(true).expect("set_nonblocking call failed");
    //                 stream.write(b"+").unwrap();
    //
    //                 while !breakpoints.contains(&board.cpu.read_instruction_pc()) {
    //                     match stream.read(&mut data) {
    //                         Ok(size) => {
    //                             if size == 1 && data[0] == 0x03 {
    //                                 println!("received interrupt");
    //                                 break;
    //                             } else {
    //                                 println!("got {:?}", &data[0..size]);
    //                             }
    //                         },
    //                         Err(_) => {}
    //                     };
    //                     for _ in 0..128 {
    //                         if !breakpoints.contains(&board.cpu.read_instruction_pc()) {
    //                             board.step().expect("failed to step board emulation");
    //                         }
    //                     }
    //                 }
    //
    //                 stream.set_nonblocking(false).expect("set_nonblocking call failed");
    //                 build_reply(b"S05")
    //             } else if pack.data.starts_with(b"s") {
    //                 board.step().expect("failed to step board emulation");
    //
    //                 match hex_to_word(&pack.data[1..]) {
    //                     Ok(addr) => {
    //                         while board.cpu.read_instruction_pc() != addr {
    //                             board.step().expect("failed to step board emulation");
    //                         }
    //                     },
    //                     Err(_) => {}
    //                 }
    //
    //                 build_reply(b"S05")
    //             } else if pack.data.starts_with(b"Z") {
    //                 match pack.data.get(1) {
    //                     Some(b'0') => {
    //                         let addr = pack.data[3..].split(|c| *c == b',').next().expect("expected ',' in Z0 packet");
    //                         breakpoints.insert(hex_to_word(addr).expect("failed to read hex address in Z0 packet"));
    //                         build_reply(b"OK")
    //                     },
    //                     Some(_) | None => {
    //                         build_reply(b"")
    //                     },
    //                 }
    //             } else if pack.data.starts_with(b"z") {
    //                 match pack.data.get(1) {
    //                     Some(b'0') => {
    //                         let addr = pack.data[3..].split(|c| *c == b',').next().expect("expected ',' in Z0 packet");
    //                         breakpoints.remove(&hex_to_word(addr).expect("failed to read hex address in Z0 packet"));
    //                         build_reply(b"OK")
    //                     },
    //                     Some(_) | None => {
    //                         build_reply(b"")
    //                     },
    //                 }
    //             } else if pack.data.starts_with(b"p") {
    //                 // read register X where request is pX
    //
    //                 let num = &pack.data[1..];
    //                 let k = hex_to_word(num).expect("failed to read hex register in p packet");
    //                 let rval = if k == 15 {
    //                     board.cpu.read_instruction_pc().swap_bytes()
    //                 } else if k < 15 {
    //                     board.read_reg(k).swap_bytes()
    //                 } else {
    //                     0
    //                 };
    //
    //                 build_reply(word_to_hex(rval).as_bytes())
    //             } else if pack.data.starts_with(b"m") {
    //                 let (start, length) = parse_read_memory(&pack.data).expect("cannot parse read memory");
    //                 let vals = board.read_memory_region(start, length).expect("cannot read board memory region");
    //
    //                 let mut strs: Vec<u8> = Vec::new();
    //                 for val in vals {
    //                     strs.extend(format!("{:02x}", val).bytes());
    //                 }
    //
    //                 build_reply(strs.as_slice())
    //             } else if pack.data.starts_with(b"qRcmd") || pack.data.starts_with(b"X") || pack.data.starts_with(b"M") {
    //                 build_reply(b"OK")
    //             } else {
    //                 println!("unrecognised command");
    //                 build_reply(b"+")
    //             };
    //
    //             println!("sending {:?}", std::str::from_utf8(out.as_ref()));
    //             stream.write(out.as_ref()).expect("couldn't send reply");
    //         }
    //         true
    //     },
    //     Err(_) => {
    //         println!("An error occurred, terminating connection with {:?}", stream.peer_addr());
    //         stream.shutdown(Shutdown::Both).expect("failed to shutdown");
    //         false
    //     }
    // } {};
}
