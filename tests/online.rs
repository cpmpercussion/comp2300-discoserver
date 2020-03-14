#[allow(dead_code)]

extern crate rand;
use crate::rand::Rng;

use std::path::Path;
use crate::common::online::write_program;
use crate::common::online::build_program;
use std::io::Write;
use disco_emulator::Board;

pub mod common;
use crate::common::get_default_linker;
use crate::common::compile_program;
use common::online::Online;

fn run_program(name: &str, src_path: &Path) {
    let elf_path = compile_program(&src_path, &get_default_linker().unwrap()).unwrap();
    let mut board = Board::new();
    board.load_elf_from_path(&elf_path).unwrap();
    let mut online = Online::new(&elf_path).unwrap();
    write!(std::io::stdout(), "Running {}:", name).unwrap();

    let mut i = 0;
    loop {
        write!(std::io::stdout(), ".").unwrap();
        std::io::stdout().flush().unwrap();
        if let Err(e) = online.verify_state(&board) {
            println!("Step {} out of sync: {}", i, e);
            online.close();
            assert!(false);
        }

        if board.read_lr() == 0x444F4E45 {
            break;
        }

        board.step().unwrap();
        online.step();
        i += 1;
    }
    write!(std::io::stdout(), "\n").unwrap();
    online.close();
}


#[test]
fn test_online() {
    // These tests use the physical board, so we can only
    // run one at a time.

    // let programs = [
    //     "offline_mirror",
    // ];
    //
    // for program in programs.iter() {
    //     let src_path = common::get_online_src_path(program).unwrap();
    //     run_program(program, &src_path);
    // }

    for i in 1..=10 {
        writeln!(std::io::stdout(), "Fuzzing (repeat {})", i - 1).unwrap();
        fuzz_test(i * 10);
    }
}

fn fuzz_test(count: usize) {
    let tests: Vec<(&str, fn(usize) -> Vec<String>)> = vec![
        ("fuzz_cmp", fuzz_cmp),
        ("fuzz_cmn", fuzz_cmn),
        ("fuzz_clz", fuzz_clz),
        ("fuzz_exclusive", fuzz_exclusive),
        ("fuzz_bic", fuzz_bic),
        ("fuzz_bfc", fuzz_bfc),
        ("fuzz_bfi", fuzz_bfi),
        ("fuzz_asr", fuzz_asr),
        ("fuzz_and", fuzz_and),
        ("fuzz_adr", fuzz_adr),
        ("fuzz_add", fuzz_add),
        ("fuzz_adc", fuzz_adc),
        ("fuzz_sub", fuzz_sub),
    ];

    for (name, generator) in tests {
        let contents = generator(count);
        let src_path = write_program(name, &build_program(contents));
        run_program(name, &src_path);
    }
}

fn fuzz_add(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ADD (imm) T1
    for _ in 0..count {
        out.push(format!("adds.N r{}, r{}, {}", rng.reg_low(), rng.reg_low(), rng.imm3()));
    }

    // ADD (imm) T2
    for _ in 0..count {
        out.push(format!("adds.N r{}, {}", rng.reg_low(), rng.imm8()));
    }

    // ADD (imm) T3
    for _ in 0..count {
        out.push(format!("add{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // ADD (imm) T4
    for _ in 0..count {
        out.push(format!("add.W r{}, r{}, {}", rng.reg_high(), rng.reg_high(), rng.imm12()));
    }

    // ADD (reg) T1
    for _ in 0..count {
        out.push(format!("adds.N r{}, r{}, r{}", rng.reg_low(), rng.reg_low(), rng.reg_low()));
    }

    // ADD (reg) T2
    for _ in 0..count {
        out.push(format!("add.N r{}, r{}", rng.reg_high(), rng.reg_high()));
    }

    // ADD (reg) T3
    for _ in 0..count {
        out.push(format!("add{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

fn fuzz_sub(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // SUB (imm) T1
    for _ in 0..count {
        out.push(format!("subs.N r{}, r{}, {}", rng.reg_low(), rng.reg_low(), rng.imm3()));
    }

    // SUB (imm) T2
    for _ in 0..count {
        out.push(format!("subs.N r{}, {}", rng.reg_low(), rng.imm8()));
    }

    // SUB (imm) T3
    for _ in 0..count {
        out.push(format!("sub{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // SUB (imm) T4
    for _ in 0..count {
        out.push(format!("sub.W r{}, r{}, {}", rng.reg_high(), rng.reg_high(), rng.imm12()));
    }

    // SUB (reg) T1
    for _ in 0..count {
        out.push(format!("subs.N r{}, r{}, r{}", rng.reg_low(), rng.reg_low(), rng.reg_low()));
    }

    // SUB (reg) T2
    for _ in 0..count {
        out.push(format!("sub{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

fn fuzz_adc(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ADC (imm) T1
    for _ in 0..count {
        out.push(format!("adc{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // ADC (reg) T1
    for _ in 0..count {
        out.push(format!("adcs.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // ADC (reg) T2
    for _ in 0..count {
        out.push(format!("adc{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

fn fuzz_adr(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ADR T1
    for _ in 0..count {
        // Labels are messy to get right (and exhaustive), so we directly encode it instead
        out.push(format!(".hword 0b10100{:03b}{:08b}", rng.reg_low(), rng.imm8()));
    }

    // ADR T2
    for _ in 0..count {
        out.push(format!("adr.W r{}, -{}", rng.reg_high(), rng.imm12()));
    }

    // ADR T3
    for _ in 0..count {
        out.push(format!("adr.W r{}, {}", rng.reg_high(), rng.imm12()));
    }

    return out;
}

fn fuzz_and(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // AND (imm) T1
    for _ in 0..count {
        out.push(format!("and{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // AND (reg) T1
    for _ in 0..count {
        out.push(format!("ands.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // AND (reg) T2
    for _ in 0..count {
        out.push(format!("and{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

fn fuzz_asr(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ASR (imm) T1
    for _ in 0..count {
        out.push(format!("asrs.N r{}, r{}, {}", rng.reg_low(), rng.reg_low(), rng.range(1, 33)));
    }

    // ASR (imm) T2
    for _ in 0..count {
        out.push(format!("asr{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.range(1, 33)));
    }

    // ASR (reg) T1
    for _ in 0..count {
        out.push(format!("asrs.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // ASR (reg) T2
    for _ in 0..count {
        out.push(format!("asr{}.W r{}, r{}, r{}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high()));
    }

    return out;
}

// No fuzz for B

fn fuzz_bfc(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // BFC T1
    for _ in 0..count {
        let lsb = rng.range(0, 32);
        let width = rng.range(1, 33 - lsb);
        out.push(format!("bfc.W r{}, {}, {}", rng.reg_high(), lsb, width));
    }

    return out;
}

fn fuzz_bfi(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // BFI T1
    for _ in 0..count {
        let lsb = rng.range(0, 32);
        let width = rng.range(1, 33 - lsb);
        out.push(format!("bfi.W r{}, r{}, {}, {}", rng.reg_high(), rng.reg_high(), lsb, width));
    }

    return out;
}

fn fuzz_bic(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // BIC (imm) T1
    for _ in 0..count {
        out.push(format!("bic{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // BIC (reg) T1
    for _ in 0..count {
        out.push(format!("bics.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // BIC (reg) T2
    for _ in 0..count {
        out.push(format!("bic{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

// No fuzz for BKPT, BL, BLX, BX, CBZ

fn fuzz_exclusive(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // LDREX, STREX, CLREX
    // NOTE: This isn't particularly effective for coverage. It's more about
    // exclusive effects.
    out.push(format!("ldr r0, =0x20000000"));
    out.push(format!("mov r1, 150"));
    out.push(format!("str r1, [r0]"));
    for _ in 0..(count * 3) {
        match rng.range(0, 3) {
            0 => out.push(format!("ldrex r1, [r0]")),
            1 => {
                out.push(format!("add r1, 1"));
                out.push(format!("strex r2, r1, [r0]"));
            },
            _ => out.push(format!("clrex")),
        }
    }

    return out;
}

fn fuzz_clz(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // CLZ T1
    for _ in 0..count {
        let src_reg = rng.reg_high();
        out.push(format!("mov r{}, {}", src_reg, rng.thumb_expandable()));
        out.push(format!("clz r{}, r{}", rng.reg_high(), src_reg));
    }

    return out;
}

fn fuzz_cmn(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // CMN (imm) T1
    for _ in 0..count {
        let src_reg = rng.reg_high();
        let value = rng.thumb_expandable();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("cmn.W r{}, {}", src_reg, value));
        out.push(format!("cmn.W r{}, {}", src_reg, rng.thumb_expandable()));
    }

    // CMN (reg) T1
    for _ in 0..count {
        let src_reg = rng.reg_low();
        let dest_reg = rng.reg_low();
        let value = rng.thumb_expandable();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("mov r{}, {}", dest_reg, value));
        out.push(format!("cmn.N r{}, r{}", dest_reg, src_reg));
        out.push(format!("cmn.N r{}, r{}", dest_reg, rng.reg_low()));
    }

    // CMN (reg) T2
    for _ in 0..count {
        let src_reg = rng.reg_high();
        let dest_reg = rng.reg_high();
        out.push(format!("mov r{}, {}", src_reg, rng.thumb_expandable()));
        out.push(format!("cmn.W r{}, r{}", dest_reg, src_reg));
        out.push(format!("cmn.W r{}, r{}, {}", dest_reg, src_reg, rng.shift()));
        out.push(format!("cmn.W r{}, r{}, {}", dest_reg, rng.reg_high(), rng.shift()));
    }

    return out;
}

fn fuzz_cmp(count: usize) -> Vec<String> {
    let count = count / 2; // We have double the typical instructions per test
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // CMP (imm) T1
    for _ in 0..count {
        let src_reg = rng.reg_low();
        let value = rng.imm8();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("cmp.N r{}, {}", src_reg, value));
        out.push(format!("cmp.N r{}, {}", src_reg, rng.imm8()));
    }

    // CMP (imm) T2
    for _ in 0..count {
        let src_reg = rng.reg_high();
        let value = rng.thumb_expandable();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("cmp.W r{}, {}", src_reg, value));
        out.push(format!("cmp.W r{}, {}", src_reg, rng.thumb_expandable()));
    }

    // CMP (reg) T1
    for _ in 0..count {
        let src_reg = rng.reg_low();
        let dest_reg = rng.reg_low();
        let value = rng.thumb_expandable();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("mov r{}, {}", dest_reg, value));
        out.push(format!("cmp.N r{}, r{}", dest_reg, src_reg));
        out.push(format!("cmp.N r{}, r{}", dest_reg, rng.reg_low()));
    }

    // CMP (reg) T2
    for _ in 0..count {
        let src_reg = rng.reg_high();
        let dest_reg = if src_reg > 8 { rng.reg_high() } else { rng.range(8, 13) };
        let value = rng.thumb_expandable();
        out.push(format!("mov r{}, {}", src_reg, value));
        out.push(format!("mov r{}, {}", dest_reg, value));
        out.push(format!("cmp.N r{}, r{}", dest_reg, src_reg));
        out.push(format!("cmp.N r{}, r{}", dest_reg, if dest_reg > 8 { rng.reg_high() } else { rng.range(8, 13) }));
    }

    // CMN (reg) T3
    for _ in 0..count {
        let src_reg = rng.reg_high();
        let dest_reg = rng.reg_high();
        out.push(format!("mov r{}, {}", src_reg, rng.thumb_expandable()));
        out.push(format!("cmp.W r{}, r{}", dest_reg, src_reg));
        out.push(format!("cmp.W r{}, r{}, {}", dest_reg, src_reg, rng.shift()));
        out.push(format!("cmp.W r{}, r{}, {}", dest_reg, rng.reg_high(), rng.shift()));
    }

    return out;
}

struct EmuRng {
    rng: rand::prelude::StdRng,
}

impl EmuRng {
    fn new() -> EmuRng {
        let seed = rand::thread_rng().gen::<u64>();
        println!("Seed is {}", seed);
        return EmuRng {
            rng: rand::SeedableRng::seed_from_u64(seed),
        }
    }

    fn range(&mut self, low: u32, high: u32) -> u32 {
        self.rng.gen_range(low, high)
    }

    fn reg_low(&mut self) -> u32 {
        self.rng.gen_range(0, 8)
    }

    fn reg_high(&mut self) -> u32 {
        self.rng.gen_range(0, 13)
    }

    fn randomize_regs(&mut self, out: &mut Vec<String>) {
        for i in 0..=12 {
            out.push(format!("mov r{}, {}", i, self.thumb_expandable()));
        }
    }

    fn imm3(&mut self) -> u32 {
        self.rng.gen_range(0, 8)
    }

    fn imm8(&mut self) -> u32 {
        self.rng.gen_range(0, 0x100)
    }

    fn imm12(&mut self) -> u32 {
        self.rng.gen_range(0, 0x1000)
    }

    fn setflags(&mut self) -> String {
        if self.rng.gen_range(0, 2) == 1 { String::new() } else { String::from("s") }
    }

    fn thumb_expandable(&mut self) -> u32 {
        let byte = self.imm8();
        match self.range(0, 5) {
            0 => byte,
            1 => byte | byte << 16,
            2 => byte << 8 | byte << 24,
            3 => byte | byte << 8 | byte << 16 | byte << 24,
            _ => {
                let byte = 1 << 7 | byte;
                let shift = self.range(1, 25);
                byte << shift
            }
        }
    }

    fn shift(&mut self) -> String {
        match self.range(0, 5) {
            0 => format!("LSL {}", self.range(0, 32)),
            1 => format!("LSR {}", self.range(1, 33)),
            2 => format!("ASR {}", self.range(1, 33)),
            3 => format!("ROR {}", self.range(1, 32)),
            _ => String::from("RRX"),
        }
    }
}
