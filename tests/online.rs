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
        ("fuzz_orr", fuzz_orr),
        ("fuzz_orn", fuzz_orn),
        ("fuzz_nop", fuzz_nop),
        ("fuzz_mvn", fuzz_mvn),
        ("fuzz_mul", fuzz_mul),
        ("fuzz_movt", fuzz_movt),
        ("fuzz_mov", fuzz_mov),
        ("fuzz_mla_mls", fuzz_mla_mls),
        ("fuzz_lsr", fuzz_lsr),
        ("fuzz_lsl", fuzz_lsl),
        ("fuzz_ldr_str", fuzz_ldr_str),
        ("fuzz_ldm", fuzz_ldm),
        ("fuzz_eor", fuzz_eor),
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

fn fuzz_eor(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // EOR (imm) T1
    for _ in 0..count {
        out.push(format!("eor{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
    }

    // EOR (reg) T1
    for _ in 0..count {
        out.push(format!("eors.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // EOR (reg) T2
    for _ in 0..count {
        out.push(format!("eor{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
    }

    return out;
}

// TODO: fuzz for IT

fn fuzz_ldm(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    out.push(format!("mov r0, 0x20000000"));
    out.push(format!("mov r1, 0xD1000000"));
    for _ in 0..=15 {
        out.push(format!("str r1, [r0], 4"));
        out.push(format!("add r1, 1"));
    }

    // LDM T1
    for _ in 0..count {
        let reg = rng.reg_low();
        out.push(format!("mov r{}, 0x20000000", reg));
        // Written LDM in binary to make randomised registers easier
        out.push(format!(".hword 0b11001{:03b}{:08b}", reg, rng.range(1, 256)));
    }

    // LDM T2
    for _ in 0..count {
        // NOTE: Certain combinations of WBACK and register are UNPREDICTABLE (same for
        // LDMBD). However, we'll wait until a test fails before investigating. Otherwise,
        // it just means we emulate with even more accuracy than necessary.
        let reg = rng.reg_low();
        out.push(format!("mov r{}, 0x20000000", reg));
        // Written LDM in binary to make randomised registers easier
        out.push(format!(".hword 0b1110100010{:01b}1{:04b}", rng.range(0, 2), reg));
        out.push(format!(".hword 0b0{:01b}0{:013b}", rng.range(0, 2), rng.imm13()));
    }

    // LDMDB T1
    for _ in 0..count {
        let reg = rng.reg_high();
        out.push(format!("ldr r{}, =0x20000010", reg));
        // Written LDMDB in binary to make randomised registers easier
        out.push(format!(".hword 0b1110100100{:01b}1{:04b}", rng.range(0, 2), reg));
        out.push(format!(".hword 0b0{:01b}0{:013b}", rng.range(0, 2), rng.imm13()));
    }

    return out;
}

fn fuzz_ldr_str(count: usize) -> Vec<String> {
    let count = std::cmp::min(count, 60);

    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // LDR (imm) T1 / STR (imm) T1
    for _ in 0..count {
        let address = rng.range(0x2000_0000, 0x2001_7FFD);
        let offset  = rng.imm5() << 2;

        let src_reg = rng.reg_low();
        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("str.N r{}, [r{}]", rng.reg_low(), src_reg));
        out.push(format!("ldr.N r{}, [r{}]", rng.reg_low(), src_reg));

        if address + offset <= 0x2001_7FFC {
            out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
            out.push(format!("str.N r{}, [r{}, {}]", rng.reg_low(), src_reg, offset));
            out.push(format!("ldr.N r{}, [r{}, {}]", rng.reg_low(), src_reg, offset));
        }
    }

    // LDR (imm) T2 / STR (imm) T2
    out.push(format!("ldr sp, =0x20000400"));
    for _ in 0..count {
        let offset = rng.imm8() << 2;
        out.push(format!("str.N r{}, [sp, {}]", rng.reg_low(), offset));
        out.push(format!("ldr.N r{}, [sp, {}]", rng.reg_low(), offset));
    }

    // LDR (imm) T3 / STR (imm) T3
    for _ in 0..count {
        let address = rng.range(0x2000_0000, 0x2001_7FFD);
        let offset = rng.imm12();
        let src_reg = rng.reg_high();
        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("str.W r{}, [r{}]", rng.reg_high(), src_reg));
        out.push(format!("ldr.W r{}, [r{}]", rng.reg_high(), src_reg));

        if address + offset <= 0x2001_7FFC {
            out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
            out.push(format!("str.W r{}, [r{}, {}]", rng.reg_high(), src_reg, offset));
            out.push(format!("ldr.W r{}, [r{}, {}]", rng.reg_high(), src_reg, offset));
        }
    }

    // LDR (imm) T4 / STR (imm) T4
    for _ in 0..(count / 3) {
        let address = rng.range(0x2000_0000, 0x2001_7FFD);
        let offset = rng.imm8();
        let src_reg = rng.reg_high();
        let sign = if rng.range(0, 2) == 0 { "-" } else { "+" };

        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("str.W r{}, [r{}, -{}]", rng.reg_high_not(src_reg), src_reg, offset));
        out.push(format!("ldr.W r{}, [r{}, -{}]", rng.reg_high_not(src_reg), src_reg, offset));

        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("str.W r{}, [r{}], {}{}", rng.reg_high_not(src_reg), src_reg, sign, offset));
        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("ldr.W r{}, [r{}], {}{}", rng.reg_high_not(src_reg), src_reg, sign, offset));

        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("str.W r{}, [r{}, {}{}]!", rng.reg_high_not(src_reg), src_reg, sign, offset));
        out.push(format!("ldr r{}, =0x{:08X}", src_reg, address));
        out.push(format!("ldr.W r{}, [r{}, {}{}]!", rng.reg_high_not(src_reg), src_reg, sign, offset));
    }

    return out;
}

fn fuzz_lsl(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // LSL (imm) T1
    for _ in 0..count {
        out.push(format!("lsls.N r{}, r{}, {}", rng.reg_low(), rng.reg_low(), rng.range(0, 32)));
    }

    // LSL (imm) T2
    for _ in 0..count {
        out.push(format!("lsl{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.range(0, 32)));
    }

    // LSL (reg) T1
    for _ in 0..count {
        out.push(format!("lsls.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // LSL (reg) T2
    for _ in 0..count {
        out.push(format!("lsl{}.W r{}, r{}, r{}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high()));
    }

    return out;
}

fn fuzz_lsr(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // LSR (imm) T1
    for _ in 0..count {
        out.push(format!("lsrs.N r{}, r{}, {}", rng.reg_low(), rng.reg_low(), rng.range(1, 33)));
    }

    // LSR (imm) T2
    for _ in 0..count {
        out.push(format!("lsr{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.range(1, 33)));
    }

    // LSR (reg) T1
    for _ in 0..count {
        out.push(format!("lsrs.N r{}, r{}", rng.reg_low(), rng.reg_low()));
    }

    // LSR (reg) T2
    for _ in 0..count {
        out.push(format!("lsr{}.W r{}, r{}, r{}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high()));
    }

    return out;
}

fn fuzz_mla_mls(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // MLA T1, MLS T1
    for _ in 0..count {
        out.push(format!("mla r{}, r{}, r{}, r{}", rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.reg_high()));
        out.push(format!("mls r{}, r{}, r{}, r{}", rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.reg_high()));
    }

    return out;
}

fn fuzz_mov(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // MOV (imm) T1 & MOV (imm) T2
    // NOTE: Combined because mov.n shouldn't change carry, but mov.w can
    for _ in 0..count {
        out.push(format!("movs.N r{}, {}", rng.reg_low(), rng.imm8()));
        out.push(format!("mov{}.W r{}, {}", rng.setflags(), rng.reg_high(), rng.thumb_expandable()));
    }

    // MOV (imm) T3
    for _ in 0..count {
        out.push(format!("mov.W r{}, {}", rng.reg_high(), rng.imm16()));
    }

    // MOV (reg) T1
    for i in 0..count {
        out.push(format!("mov.N r{}, r{}", rng.reg_high(), rng.reg_high()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    // MOV (reg) T2
    for i in 0..count {
        out.push(format!("movs.N r{}, r{}", rng.reg_low(), rng.reg_low()));
        if i % 10 == 0 {
            rng.randomize_regs_low(&mut out);
        }
    }

    // MOV (reg) T3
    for i in 0..count {
        out.push(format!("mov{}.W r{}, r{}", rng.setflags(), rng.reg_high(), rng.reg_high()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    return out;
}

fn fuzz_movt(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // MOVT T1
    for _ in 0..count {
        out.push(format!("movt r{}, {}", rng.reg_high(), rng.imm16()));
    }

    return out;
}

fn fuzz_mul(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // MUL T1
    for _ in 0..count {
        out.push(format!("muls.N r{0}, r{1}, r{0}", rng.reg_low(), rng.reg_low()));
    }

    // MUL T2
    for _ in 0..count {
        out.push(format!("mul.W r{}, r{}, r{}", rng.reg_high(), rng.reg_high(), rng.reg_high()));
    }

    return out;
}

fn fuzz_mvn(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();

    // MVN (imm) T1
    for _ in 0..count {
        out.push(format!("mvn{}.W r{}, {}", rng.setflags(), rng.reg_high(), rng.thumb_expandable()));
    }

    // MVN (reg) T1
    for i in 0..count {
        out.push(format!("mvns.N r{}, r{}", rng.reg_low(), rng.reg_low()));
        if i % 10 == 0 {
            rng.randomize_regs_low(&mut out);
        }
    }

    // MVN (reg) T2
    for i in 0..count {
        out.push(format!("mvn{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.shift()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    return out;
}

fn fuzz_nop(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // NOP T1
    for _ in 0..std::cmp::min(count, 3) {
        out.push(format!("nop"));
    }

    return out;
}

fn fuzz_orn(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ORN (imm) T1
    for i in 0..count {
        out.push(format!("orn{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    // ORN (reg) T1
    for i in 0..count {
        out.push(format!("orn{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    return out;
}

fn fuzz_orr(count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rng = EmuRng::new();
    rng.randomize_regs(&mut out);

    // ORR (imm) T1
    for i in 0..count {
        out.push(format!("orr{}.W r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.thumb_expandable()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
    }

    // ORR (reg) T1
    for i in 0..count {
        out.push(format!("orrs.N r{}, r{}", rng.reg_low(), rng.reg_low()));
        if i % 10 == 0 {
            rng.randomize_regs_low(&mut out);
        }
    }

    // ORR (reg) T2
    for i in 0..count {
        out.push(format!("orr{}.W r{}, r{}, r{}, {}", rng.setflags(), rng.reg_high(), rng.reg_high(), rng.reg_high(), rng.shift()));
        if i % 10 == 0 {
            rng.randomize_regs(&mut out);
        }
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

    fn reg_high_not(&mut self, not: u32) -> u32 {
        let mut reg = self.reg_high();
        if reg == not {
            if not == 0 {
                reg += 1;
            } else {
                reg -= 1;
            }
        }
        reg
    }

    fn randomize_regs(&mut self, out: &mut Vec<String>) {
        for i in 0..=12 {
            out.push(format!("mov r{}, {}", i, self.thumb_expandable()));
        }
    }

    fn randomize_regs_low(&mut self, out: &mut Vec<String>) {
        for i in 0..=7 {
            out.push(format!("mov r{}, {}", i, self.thumb_expandable()));
        }
    }

    fn imm3(&mut self) -> u32 {
        self.rng.gen_range(0, 8)
    }

    fn imm5(&mut self) -> u32 {
        self.rng.gen_range(0, 0b100000)
    }

    fn imm8(&mut self) -> u32 {
        self.rng.gen_range(0, 0x100)
    }

    fn imm12(&mut self) -> u32 {
        self.rng.gen_range(0, 0x1000)
    }

    fn imm13(&mut self) -> u32 {
        self.rng.gen_range(0, 0x2000)
    }

    fn imm16(&mut self) -> u32 {
        self.rng.gen_range(0, 0x1_0000)
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
        // LSL 0 has a relatively higher chance of occuring
        match self.range(0, 6) {
            0 => format!("LSL {}", self.range(0, 32)),
            1 => format!("LSR {}", self.range(1, 33)),
            2 => format!("ASR {}", self.range(1, 33)),
            3 => format!("ROR {}", self.range(1, 32)),
            4 => String::from("RRX"),
            _ => String::from("LSL 0"),
        }
    }
}
