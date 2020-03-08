use crate::{ByteInstruction};
use crate::utils::bits::{bitset, matches};
use super::{ItPos, InstructionContext};
use super::opcode::{Opcode};
use super::tag;

type Context = InstructionContext;

pub fn decode_thumb_wide(word: u32, c: InstructionContext) -> ByteInstruction {
    // A5.3
    assert!(matches(word, 29, 0b111, 0b111));

    let op2 = (word >> 20) & 0b111_1111;
    return match (word >> 27) & 0b11 {
        0b01 => {
            if bitset(op2, 6) {
                id_coprocessor_instr(word, c)
            } else if bitset(op2, 5) {
                id_data_processing_shifted_register(word, c)
            } else if bitset(op2, 2) {
                id_ldr_str_dual(word, c)
            } else {
                id_ldr_str_multiple(word, c)
            }
        }
        0b10 => {
            if bitset(word, 15) {
                id_branch_and_misc(word, c)
            } else if bitset(op2, 5) {
                id_data_proc_plain_binary_immediate(word, c)
            } else {
                id_data_proc_modified_immediate(word, c)
            }
        }
        0b11 => {
            if bitset(op2, 6) {
                id_coprocessor_instr(word, c)
            } else if bitset(op2, 5) {
                if bitset(op2, 4) {
                    if bitset(op2, 3) {
                        id_long_multiply_div(word, c)
                    } else {
                        id_multiply_diff(word, c)
                    }
                } else {
                    id_data_proc_register(word, c)
                }
            } else if (op2 & 0b1110001) == 0 {
                id_store_single(word, c)
            } else {
                match op2 & 0b111 {
                    0b001 => id_load_byte(word, c),
                    0b011 => id_load_half_word(word, c),
                    0b101 => id_load_word(word, c),
                    _ => tag::get_undefined_wide(c, word),
                }
            }
        }
        _ => unreachable!(), // 0b00 would be a narrow instruction
    };
}

fn id_coprocessor_instr(word: u32, c: Context) -> ByteInstruction {
    // A5.3.18 // Need to parse params
    assert!(matches(word, 26, 0b111_0_11, 0b111_0_11));
    let op1 = (word >> 20) & 0x3F;
    let _coproc = (word >> 8) & 0xF;
    let op = (word >> 4) & 0b1;

    let p = (word >> 24) & 0b1;
    let _u = (word >> 23) & 0b1;
    let _d = (word >> 22) & 0b1; // Bug in manual
    let w = (word >> 21) & 0b1;
    let rn = (word >> 16) & 0xF;
    let crd = (word >> 12) & 0xF;
    let _imm8 = word & 0xFF;

    if (op1 & 0b10_0001 == 0b00_0000) && (op1 & 0b11_1010 != 0b00_0000) {
        let mut base = tag::get_wide(Opcode::Stc, c, 0, 0);
        if rn == 15 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 & 0b10_0001 == 0b00_0001) && (op1 & 0b11_1010 != 0b00_0000) {
        let mut base = tag::get_wide(if rn == 15 { Opcode::LdcLit} else { Opcode::LdcImm }, c, 0, 0);
        if rn == 15 && (w == 1 || p == 1) {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if op1 == 0b00_0100 {
        let rt = crd;
        let rt2 = rn;
        let mut base = tag::get_wide(Opcode::Mcrr, c, 0, 0);
        if rt == 15 || rt2 == 15 || rt == 13 || rt2 == 13 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if op1 == 0b00_0101 {
        let rt = crd;
        let rt2 = rn;
        let mut base = tag::get_wide(Opcode::Mrrc, c, 0, 0);
        if rt == 15 || rt2 == 15 || rt == rt2 || rt == 13 || rt2 == 13 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 & 0b11_0000 == 0b10_0000) && op == 0 {
        return tag::get_wide(Opcode::Cdp, c, 0, 0);
    }

    if (op1 & 0b11_0001 == 0b10_000) && op == 1 {
        let rt = crd;
        let mut base = tag::get_wide(Opcode::Mcr, c, 0, 0);
        if rt == 15 || rt == 13 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 & 0b11_0001 == 0b10_001) && op == 1 {
        let rt = crd;
        let mut base = tag::get_wide(Opcode::Mrc, c, 0, 0);
        if rt == 13 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    return tag::get_undefined_wide(c, word);
}

// Takes type[2] and imm5[5] from encoding, returns shift_t[3] and shift_n[6]
fn decode_imm_shift(initial_t: u32, initial_n: u32) -> (u32, u32) {
    let shift_n = if initial_n == 0 && (initial_t == 0b01 || initial_t == 0b10) {
        32
    } else {
        initial_n
    };

    return if initial_t == 0b11 {
        if initial_n == 0 {
            (0b11, 1)
        } else {
            (0b100, shift_n)
        }
    } else {
        (initial_t, shift_n)
    }
}

fn id_data_processing_shifted_register(word: u32, c: Context) -> ByteInstruction {
    // A5.3.11 // DONE
    assert!(matches(word, 25, 0b111_1111, 0b111_0101));
    let rn = (word >> 16) & 0xF;
    let rd = (word >> 8) & 0xF;
    let rm = word & 0xF;
    let setflags = bitset(word, 20);

    let default_data = rd | rn << 4 | rm << 8 | (setflags as u32) << 12;
    let default_data_comp = (default_data >> 4) & 0xFF;
    let default_data_alt = rd | rm << 4 | (setflags as u32) << 8;

    let (shift_t, shift_n) = decode_imm_shift((word >> 4) & 0b11, (word >> 6) & 0b11 | (word & (0b111 << 12)) >> 10);
    let pro_extra = shift_t | shift_n << 3;

    let mut instr = match (word >> 21) & 0xF {
        0b0000 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::TstReg, c, default_data_comp, pro_extra) // A7.7.189 T2
            } else {
                tag::get_wide(Opcode::AndReg, c, default_data, pro_extra) // A7.7.9 T2
            };
            if rd == 13 || (rd == 15 && !setflags) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0001 => {
            let mut base = tag::get_wide(Opcode::BicReg, c, default_data, pro_extra); // A7.7.16 T2
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0010 => {
            if rn == 15 {
                if shift_t == 0 && shift_n == 0 {
                    let mut base = tag::get_wide(Opcode::MovReg, c, (word & (1 << 20)) >> 16 | (word >> 8) & 0xF, word & 0xF); // A7.7.77 T3
                    if rd == 15 || rm == 15 || (rd == 13 && rm == 13) || (setflags && (rd == 13 || rm == 13)) {
                        base = tag::as_unpred_w(base);
                    }
                    base
                } else {
                    let opcode = match shift_t {
                        0b00 => Opcode::LslImm, // A7.7.68 T2
                        0b01 => Opcode::LsrImm, // A7.7.70 T2
                        0b10 => Opcode::AsrImm, // A7.7.10 T2
                        0b11 => Opcode::Rrx, // A7.7.118 T1
                        0b100 => Opcode::RorImm, // A7.7.116 T1
                        _ => unreachable!(),
                    };
                    let mut base = tag::get_wide(opcode, c, rd | rm << 4 | (setflags as u32) << 8, shift_n);
                    if rd == 13 || rd == 15 || rm == 13 || rm == 15 {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
            } else {
                let mut base = tag::get_wide(Opcode::OrrReg, c, default_data, pro_extra); // A7.7.92 T2
                if rd == 13 || rd == 15 || rn == 13 || rm == 13 || rm == 15 {
                    base = tag::as_unpred_w(base);
                }
                base
            }
        }
        0b0011 => {
            let mut base = if rn == 15 {
                tag::get_wide(Opcode::MvnReg, c, default_data_alt, pro_extra) // A7.7.86 T2
            } else {
                tag::get_wide(Opcode::OrnReg, c, default_data, pro_extra) // A7.7.90 T2
            };
            if rd == 13 || rd == 15 || rm == 13 || rm == 15 || rn == 13 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0100 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::TeqReg, c, default_data_comp, pro_extra) // A7.7.187 T1
            } else {
                tag::get_wide(Opcode::EorReg, c, default_data, pro_extra) // A7.7.36 T2
            };
            if rd == 13 || (rd == 15 && !setflags) || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0110 => {
            let mut base = if setflags || bitset(word, 4) {
                tag::get_undefined_wide(c, word)
            } else {
                tag::get_wide(Opcode::Pkhbt, c, default_data, pro_extra) // A7.7.93 T1
            };
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1000 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::CmnReg, c, default_data_comp, pro_extra) // A7.7.26 T2
            } else {
                tag::get_wide(Opcode::AddReg, c, default_data, pro_extra) // A7.7.4 T3
            };
            if rd == 13 || (rd == 15 && !setflags) || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1010 => {
            let mut base = tag::get_wide(Opcode::AdcReg, c, default_data, pro_extra); // A7.7.2 T2
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1011 => {
            let mut base = tag::get_wide(Opcode::SbcReg, c, default_data, pro_extra); // A7.7.125 T2
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1101 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::CmpReg, c, default_data_comp, pro_extra) // A7.7.28 T3
            } else {
                tag::get_wide(Opcode::SubReg, c, default_data, pro_extra) // A7.7.175 T2
            };
            if rd == 13 || (rd == 15 && !setflags) || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1110 => {
            let mut base = tag::get_wide(Opcode::RsbReg, c, default_data, pro_extra); // A7.7.120 T1
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        _ => tag::get_undefined_wide(c, word),
    };

    if bitset(word, 15) {
        instr = tag::as_unpred_w(instr);
    }
    return instr;
}

fn id_ldr_str_dual(word: u32, c: Context) -> ByteInstruction {
    // A5.3.6 // DONE
    assert!(matches(word, 22, 0b111_1111_00_1, 0b111_0100_00_1));
    let op1 = (word >> 23) & 0b11;
    let op2 = (word >> 20) & 0b11;
    let op3 = (word >> 4) & 0xF;

    let imm8 = word & 0xFF;
    let rd = (word >> 8) & 0xF;
    let rt = (word >> 12) & 0xF;
    let rn = (word >> 16) & 0xF;

    let rt2 = rd;
    let p = (word >> 24) & 0b1;
    let u = (word >> 23) & 0b1;
    let w = (word >> 21) & 0b1;

    let default_data = rt | rd << 4;
    let default_extra = rn << 10 | imm8 << 2;

    return match (op1, op2) {
        (0b00, 0b00) => {
            let mut base = tag::get_wide(Opcode::Strex, c, default_data, default_extra);
            if (rd == 13 || rd == 15) || (rt == 13 || rt == 15) || rn == 15 || rd == rn || rd == rt {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b00, 0b01) => {
            let mut base = tag::get_wide(Opcode::Ldrex, c, default_data & 0xF, default_extra);
            if rd != 15 || (rt == 13 || rt == 15) || rn == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b00, 0b10) |
        (0b01, 0b10) |
        (0b10, 0b00) |
        (0b10, 0b10) |
        (0b11, 0b00) |
        (0b11, 0b10) => {
            let data = default_data | p << 10 | u << 9 | w << 8;
            let mut base = tag::get_wide(Opcode::StrdImm, c, data, default_extra);
            if (w > 0 && (rn == rt || rn == rt2)) || (rn == 15 || (rt == 13 || rt == 15) || (rt2 == 13 || rt2 == 15)) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b00, 0b11) |
        (0b01, 0b11) |
        (0b10, 0b01) |
        (0b10, 0b11) |
        (0b11, 0b01) |
        (0b11, 0b11) => {
            let data = default_data | p << 10 | u << 9 | w << 8;
            let mut base = tag::get_wide(Opcode::LdrdImm, c, data, default_extra);
            if (w > 0 && (rn == rt || rn == rt2)) || (rn == 15 || (rt == 13 || rt == 15) || (rt2 == 13 || rt2 == 15)) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b01, 0b00) if op3 == 0b0100 => {
            let rd = word & 0xF;
            let mut base = tag::get_wide(Opcode::Strexb, c, default_data & 0xF | rd << 4, rn);
            if (rd == 13 || rd == 15) || (rt == 13 || rt == 15) || rn == 15 || rd == rn || rd == rt {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b01, 0b00) if op3 == 0b0101 => {
            let rd = word & 0xF;
            let mut base = tag::get_wide(Opcode::Strexh, c, default_data & 0xF | rd << 4, rn);
            if (rd == 13 || rd == 15) || (rt == 13 || rt == 15) || rn == 15 || rd == rn || rd == rt {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b01, 0b01) => {
            match op3 {
                0b0000 | 0b0001 => {
                    let rm = word & 0xF;
                    let h = (word >> 4) & 0b1;
                    let mut base = tag::get_wide(Opcode::Tbb, c, rn | h << 4, rm);
                    if rn == 13 || (rm == 13 || rm == 15) {
                        base = tag::as_unpred_w(base);
                    }
                    if c.it_pos == ItPos::Within {
                        base = tag::as_unpred_it_w(base);
                    }
                    base
                }
                0b0100 => {
                    let mut base = tag::get_wide(Opcode::Ldrexb, c, rt, rn);
                    if (rt == 13 || rt == 15) || rn == 15 || rd != 15 || (word & 0xF) != 15 {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b0101 => {
                    let mut base = tag::get_wide(Opcode::Ldrexh, c, rt, rn);
                    if (rt == 13 || rt == 15) || rn == 15 || rd != 15 || (word & 0xF) != 15 {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                _ => tag::get_undefined_wide(c, word),
            }
        }
        _ => tag::get_undefined_wide(c, word),
    }

}

fn id_ldr_str_multiple(word: u32, c: Context) -> ByteInstruction {
    // A5.3.5 // DONE
    assert!(matches(word, 22, 0b111_1111_00_1, 0b111_0100_00_0));
    let op1 = (word >> 23) & 0b11;
    let w = (word >> 21) & 0b1;
    let l = (word >> 20) & 0b1;
    let rn = (word >> 16) & 0xF;

    let wrn = (rn | w << 4) == 0b11101;
    let registers = word & 0xFFFF;
    let p = (word >> 15) & 0b1;
    let m = (word >> 14) & 0b1;

    return match (op1, l) {
        (0b01, 0) => {
            let mut base = tag::get_wide(Opcode::Stm, c, rn, registers | w << 16); // A7.7.159 T2
            if bitset(word, 13) || bitset(word, 15) || rn == 15 || registers.count_ones() < 2 || (w == 1 && bitset(registers, rn)) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b01, 1) if !wrn => {
            let mut base = tag::get_wide(Opcode::Ldm, c, rn, registers | w << 16); // A7.7.41 T2
            if bitset(word, 13) || rn == 15 || registers.count_ones() < 2 || (p == 1 && m == 1) || (w == 1 && bitset(registers, rn)) {
                base = tag::as_unpred_w(base);
            }
            if bitset(registers, 15) && c.it_pos == ItPos::Within {
                base = tag::as_unpred_it_w(base);
            }
            base
        }
        (0b01, 1) if wrn => {
            let mut base = tag::get_wide(Opcode::Pop, c, 0, registers); // A7.7.99 T2
            if bitset(word, 13) || registers.count_ones() < 2 || (p == 1 && m == 1) {
                base = tag::as_unpred_w(base);
            }
            if bitset(registers, 15) && c.it_pos == ItPos::Within {
                base = tag::as_unpred_it_w(base);
            }
            base
        }
        (0b10, 0) if !wrn => {
            let mut base = tag::get_wide(Opcode::Stmdb, c, rn, registers | w << 16); // A7.7.160 T1
            if bitset(word, 15) || bitset(word, 13) || rn == 15 || registers.count_ones() < 2 || (w == 1 && bitset(registers, rn)) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b10, 0) if wrn => {
            let mut base = tag::get_wide(Opcode::Push, c, 0, registers); // A7.7.101 T2
            if bitset(word, 15) || bitset(word, 13) || registers.count_ones() < 2 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b10, 1) => {
            let mut base = tag::get_wide(Opcode::Ldmdb, c, rn, registers | w << 16); // A7.7.42 T1
            if bitset(word, 13) || rn == 15 || registers.count_ones() < 2 || (p == 1 && m == 1) || (w == 1 && bitset(registers, rn)) {
                base = tag::as_unpred_w(base);
            }
            if bitset(registers, 15) && c.it_pos == ItPos::Within {
                base = tag::as_unpred_it_w(base);
            }
            base
        }
        _ => tag::get_undefined_wide(c, word),
    };
}

fn id_branch_and_misc(word: u32, c: Context) -> ByteInstruction {
    // A5.3.4 // DONE
    assert!(matches(word, 15, 0b111_11_0000000_0000_1, 0b111_10_0000000_0000_1));
    let op = (word >> 20) & 0x7F;
    let op1 = (word >> 12) & 0b111;

    let sysm = word & 0xFF;
    let mask = (word >> 10) & 0b11;
    let rn = (word >> 16) & 0xF;
    let rd = (word >> 8) & 0xF;
    let imm11 = word & 0x7FF;
    let j1 = (word >> 13) & 0b1;
    let j2 = (word >> 11) & 0b1;
    let s = (word >> 26) & 0b1;
    let i1 = !(j1 ^ s) & 0b1;
    let i2 = !(j2 ^ s) & 0b1;
    let imm10 = (word >> 16) & 0x3FF;
    let imm24 = imm11 | imm10 << 11 | i2 << 21 | i1 << 22 | s << 23;

    if (op1 & 0b101) == 0b000 {
        if (op & 0b0111000) != 0b0111000 {
            let imm6 = (word >> 16) & 0x3F;
            let cond = (word >> 22) & 0xF;
            let imm20 = imm11 | imm6 << 11 | j1 << 17  | j2 << 18 | s << 19;
            let mut base = tag::get_wide(Opcode::BranchCond, c, cond, imm20); // A7.7.12 T3
            if c.it_pos != ItPos::None {
                base = tag::as_unpred_it_w(base);
            }
            return base;
        }

        if (op & 0b1111110) == 0b0111110 {
            let mut base = tag::get_wide(Opcode::Msr, c, rn, sysm | mask << 8); // A7.7.82 T1, B5.2.3 T1
            if bitset(word, 20) || bitset(word, 13) || bitset(word, 9) || bitset(word, 8) {
                base = tag::as_unpred_w(base);
            }
            if mask == 0b00 || (mask != 0b10 && !(sysm <= 3)) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            match sysm {
                0..=3 |
                5..=9 |
                16..=20 => {},
                _ => {
                    base = tag::as_unpred_w(base);
                },
            }
            return base;
        }

        let option = word & 0xF;
        if op == 0b0111010 {
            // Hint instructions
            let op2 = word & 0xFF;

            if op1 != 0b000 {
                return tag::get_undefined_wide(c, word);
            }

            let mut base = match op2 {
                0b0000_0000 => tag::get_wide(Opcode::Nop, c, 0, 0), // A7.7.88 T2
                0b0000_0001 => tag::get_wide(Opcode::Yield, c, 0, 0), // A7.7.263 T2
                0b0000_0010 => tag::get_wide(Opcode::Wfe, c, 0, 0), // A7.7.261 T2
                0b0000_0011 => tag::get_wide(Opcode::Wfi, c, 0, 0), // A7.7.262 T2
                0b0000_0100 => tag::get_wide(Opcode::Sev, c, 0, 0), // A7.7.129 T2
                0b0001_0100 => {
                    let mut base = tag::get_wide(Opcode::Csdb, c, 0, 0); // A7.7.31 T1
                    if c.it_pos != ItPos::None {
                        base = tag::as_unpred_it_w(base);
                    }
                    base
                }
                0b1111_0000..=0b1111_1111 => tag::get_wide(Opcode::Dbg, c, option, 0), // A7.7.32 T1
                _ => return tag::get_wide(Opcode::Nop, c, 0, 0),
            };
            if (word >> 11) & 0b1111_00_1_0_1 != 0b1111_00_0_0_0 {
                base = tag::as_unpred_w(base);
            }
            return base;
        }

        if op == 0b0111011 {
            // Misc control instructions
            let opc = (word >> 4) & 0xF;

            let mut base = match opc {
                0b0010 => {
                    let mut base = tag::get_wide(Opcode::Clrex, c, 0, 0); // A7.7.23 T1
                    if option != 0b1111 {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b0100 if option != 0b0000 && option != 0b0100 => tag::get_wide(Opcode::Dsb, c, option, 0), // A7.7.34 T1
                0b0100 if option == 0b0000 => {
                    let mut base = tag::get_wide(Opcode::Ssbb, c, 0, 0); // A7.7.155 T1
                    if c.it_pos != ItPos::None {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b0100 if option == 0b0100 => {
                    let mut base = tag::get_wide(Opcode::Pssbb, c, 0, 0); // A7.7.100 T1
                    if c.it_pos != ItPos::None {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b0101 => tag::get_wide(Opcode::Dmb, c, 0, 0), // A7.7.33 T1
                0b0110 => {
                    let mut base = tag::get_wide(Opcode::Isb, c, option, 0); // A7.7.37 T1
                    if c.it_pos != ItPos::None {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                _ => return tag::get_undefined_wide(c, word),
            };
            if (word >> 8) & 0b1111_00_1_0_1111 != 0b1111_00_0_0_1111 {
                base = tag::as_unpred_w(base);
            }
            return base;
        }

        if (op & 0b1111110) == 0b01111110 {
            let mut base = tag::get_wide(Opcode::Mrs, c, rd, sysm); // A7.7.82 T1, B5.2.2 T1
            if bitset(word, 20) || bitset(word, 13) || rn != 0b1111 {
                base = tag::as_unpred_w(base);
            }
            if rd == 13 || rd == 15 {
                base = tag::as_unpred_w(base);
            }
            match sysm {
                0..=3 | 5..=9 | 16..=20 => {},
                _ => {
                    base = tag::as_unpred_w(base);
                }
            }
            return base;
        }
    }

    if op1 == 0b010 && op == 0b1111111 {
        let imm4 = (word >> 16) & 0xF;
        let imm12 = word & 0xFFF;
        let imm16 = imm12 | imm4 << 12;
        return tag::get_wide(Opcode::Udf, c, imm16, 0); // A7.7.194
    }

    if (op1 & 0b101) == 0b001 {
        let mut base = tag::get_wide(Opcode::Branch, c, 0, imm24); // A7.7.12 T4
        if c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    if (op1 & 0b101) == 0b101 {
        let mut base = tag::get_wide(Opcode::Bl, c, 0, imm24); // A7.7.18 T1
        if c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    return tag::get_undefined_wide(c, word);
}

fn id_data_proc_plain_binary_immediate(word: u32, c: Context) -> ByteInstruction {
    // A5.3.3 // DONE
    assert!(matches(word, 15, 0b111_11_0_1_00000_0000_1, 0b111_10_0_1_00000_0000_0));
    let rd = (word >> 8) & 0xF;
    let rn = (word >> 16) & 0xF;

    let imm8 = word & 0xFF;
    let imm3 = (word >> 12) & 0x7;
    let imm2 = (word >> 6) & 0x3;
    let i = (word >> 26) & 0x1;

    let imm12 = imm8 | imm3 << 8 | i << 11;
    let imm5 = imm2 | imm3 << 2;
    let sat_imm5 = word & 0x1F;
    let shift_n = imm5;


    return match (word >> 20) & 0x1F {
        0b00000 => {
            let mut base = if rn == 15 {
                tag::get_wide(Opcode::Adr, c, rd, imm12) // A7.7.7 T3
            } else {
                tag::get_wide(Opcode::AddImm, c, rd << 4 | rn << 8, imm12) // A7.7.3 T4
            };
            if rd == 13 || rd == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b00100 => {
            let mut base = tag::get_wide(Opcode::MovImm, c, rd << 4, (word & (0xF << 16)) >> 4 | imm12); // A7.7.76 T3
            if rd == 13 || rd == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b01010 => {
            let mut base = if rn == 15 {
                tag::get_wide(Opcode::Adr, c, rd, get_negated_simm13(imm12)) // A7.7.7 T2
            } else {
                tag::get_wide(Opcode::SubImm, c, rd << 4 | rn << 8, imm12) // A7.7.174 T4
            };
            if rd == 13 || rd == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b01100 => {
            let mut base = tag::get_wide(Opcode::Movt, c, rd, (word & (0xF << 16)) >> 4 | imm12); // A7.7.79 T1
            if rd == 13 || rd == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b10010 if imm5 != 0 => {
            // needs to go before SSAT
            let saturate_to = (sat_imm5 & 0xF) + 1;
            let mut base = tag::get_wide(Opcode::Ssat16, c, rd | rn << 4, saturate_to); // A7.7.153 T1
            if bitset(word, 26) || bitset(word, 5) || bitset(word, 4) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b10000 | 0b10010 => {
            let sh = (word & (0b1 << 21)) >> 20;
            let saturate_to = sat_imm5 + 1;
            let mut base = tag::get_wide(Opcode::Ssat, c, rd | rn << 4, shift_n << 7 | sh << 6 | saturate_to); // A7.7.152 T1
            if bitset(word, 26) || bitset(word, 5) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b10100 => {
            let widthm1 = sat_imm5;
            let lsbit = imm5;
            let mut base = tag::get_wide(Opcode::Sbfx, c, rd | rn << 4, widthm1 | lsbit << 5); // A7.7.126 T1
            if bitset(word, 26) || bitset(word, 5) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b10110 if rn != 15 => {
            let msbit = sat_imm5;
            let lsbit = shift_n;
            let mut base = tag::get_wide(Opcode::Bfi, c, rd | rn << 4, msbit | lsbit << 5); // A7.7.14 T1
            if bitset(word, 26) || bitset(word, 15) || bitset(word, 5) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b10110 if rn == 15 => {
            let msbit = sat_imm5;
            let lsbit = imm5;
            let mut base = tag::get_wide(Opcode::Bfc, c, rd, msbit | lsbit << 5); // A7.7.13 T1
            if bitset(word, 26) || bitset(word, 5) || (rd == 13 || rd == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b11010 if imm5 != 0 => {
            // needs to go before USAT
            let saturate_to = sat_imm5 & 0xF;
            let mut base = tag::get_wide(Opcode::Usat16, c, rd | rn << 4, saturate_to); // A7.7.214 T1
            if bitset(word, 26) || bitset(word, 5) || bitset(word, 4) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b11000 | 0b11010 => {
            let sh = (word & (0b1 << 21)) >> 20;
            let saturate_to = sat_imm5;
            let mut base = tag::get_wide(Opcode::Usat, c, rd | rn << 4, shift_n << 6 | sh << 5 | saturate_to); // A7.7.213 T1
            if bitset(word, 26) || bitset(word, 5) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b11100 => {
            // UBFX
            let widthm1 = sat_imm5;
            let lsbit = imm5;
            let mut base = tag::get_wide(Opcode::Ubfx, c, rd | rn << 4, widthm1 | lsbit << 5); // A7.7.193 T1
            if bitset(word, 26) || bitset(word, 5) || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        _ => tag::get_undefined_wide(c, word),
    }
}

fn thumb_expand_imm_c_alt(word: u32) -> (u32, u32) {
    let full;
    let mut spill = 0;
    let base = word & 0xFF;
    if !bitset(word, 26) && !bitset(word, 14) {
        full = match (word >> 12) & 0b11 {
            0b00 => base,
            0b01 => base | base << 16,
            0b10 => (base | base << 16) << 8,
            0b11 => base | base << 8 | base << 16 | base << 24,
            _ => unreachable!(),
        };
    } else {
        let i = (word >> 26) & 0b1;
        let imm3 = (word >> 12) & 0b111;
        let a = (word >> 7) & 0b1;
        let encoded_shift = a | imm3 << 1 | i << 4;

        full = (base | 1 << 7) << (0x20 - encoded_shift);
        spill |= 0b1000;
        if bitset(full, 31) {
            spill |= 0b0100;
        }
    };
    spill |= full >> 30;
    return (spill, full & !(0b11 << 30));
}

fn id_data_proc_modified_immediate(word: u32, c: Context) -> ByteInstruction {
    // A5.3.1 // DONE
    assert!(matches(word, 15, 0b111_11_0_1_00000_0000_1, 0b111_10_0_0_00000_0000_0));
    let rn = (word >> 16) & 0xF;
    let rd = (word >> 8) & 0xF;
    let setflags = bitset(word, 20);
    let (spill, extra) = thumb_expand_imm_c_alt(word);
    let default_w_spill = rd << 4 | rn << 8 | (setflags as u32) << 12 | spill;

    return match (word >> 21) & 0b1111 {
        0b0000 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::TstImm, c, (word & (0xF << 16)) >> 12 | spill, extra) // A7.7.188
            } else {
                tag::get_wide(Opcode::AndImm, c, default_w_spill, extra) // A7.7.8 T1
            };
            if (rn == 13 || rn == 15) || rd == 13 || (rd == 15 && !setflags) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0001 => {
            let mut base = tag::get_wide(Opcode::BicImm, c, default_w_spill, extra); // A7.7.15 T1
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0010 => {
            let mut base = if rn == 15 {
                tag::get_wide(Opcode::MovImm, c, (word & (1 << 20)) >> 12 | (word & (0xF << 8)) >> 4 | spill, extra) // A7.7.76 T2
            } else {
                tag::get_wide(Opcode::OrrImm, c, default_w_spill, extra) // A7.7.91 T1
            };
            if rd == 13 || rd == 15 || rn == 13 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0011 => {
            let mut base = if rn == 15 {
                tag::get_wide(Opcode::MvnImm, c, (word & (1 << 20)) >> 12 | (word & (0xF << 8)) >> 4 | spill, extra) // A7.7.85 T1
            } else {
                tag::get_wide(Opcode::OrnImm, c, default_w_spill, extra) // A7.7.89 T1
            };
            if rd == 13 || rd == 15 || rn == 13 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b0100 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::TeqImm, c, (word & (0xF << 16)) >> 12 | spill, extra) // A7.7.186 T1
            } else {
                tag::get_wide(Opcode::EorImm, c, default_w_spill, extra) // A7.7.35 T1
            };
            if rn == 13 || rn == 15 || rd == 13 || (rd == 15 && !setflags) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1000 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::CmnImm, c, (word & (0xF << 16)) >> 12 | spill, extra) // A7.7.25 T1
            } else {
                tag::get_wide(Opcode::AddImm, c, default_w_spill, extra) // A7.7.3 T3
            };
            if rn == 15 || rd == 13 || (rd == 15 && !setflags) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1010 => {
            let mut base = tag::get_wide(Opcode::AdcImm, c, default_w_spill, extra); // A7.7.1 T1
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1011 => {
            let mut base = tag::get_wide(Opcode::SbcImm, c, default_w_spill, extra); // A7.7.124 T1
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1101 => {
            let mut base = if rd == 15 {
                tag::get_wide(Opcode::CmpImm, c, (word & (0xF << 16)) >> 12 | spill, extra) // A7.7.27 T2
            } else {
                tag::get_wide(Opcode::SubImm, c, default_w_spill, extra) // A7.7.174 T3
            };
            if rn == 15 || rd == 13 || (rd == 15 && !setflags) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b1110 => {
            let mut base = tag::get_wide(Opcode::RsbImm, c, default_w_spill, extra); // A7.7.119 T2
            if rd == 13 || rd == 15 || rn == 13 || rn == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        _ => tag::get_undefined_wide(c, word),
    };
}

fn id_long_multiply_div(word: u32, c: Context) -> ByteInstruction {
    // A5.3.17 // TODO
    assert!(matches(word, 23, 0b111_1111_11, 0b111_1101_11));
    let rn = (word >> 16) & 0xF;
    let rm = word & 0xF;
    let op1 = (word >> 20) & 0b111;
    let op2 = (word >> 4) & 0xF;

    let rd_lo = (word >> 12) & 0xF;
    let rd_hi = (word >> 8) & 0xF;
    let rd = rd_hi;

    return match (op1, op2) {
        (0b000, 0b0000) => {
            let mut base = tag::get_wide(Opcode::Smull, c, rn | rm << 4, rd_lo | rd_hi << 4); // A7.7.149 T1
            if (rd_lo == 13 || rd_lo == 15) || (rd_hi == 13 || rd_hi == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || rd_hi == rd_lo {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b001, 0b1111) => {
            let mut base = tag::get_wide(Opcode::Sdiv, c, rd | rn << 4, rm); // A7.7.127 T1
            if rd_lo != 15 || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b010, 0b0000) => {
            let mut base = tag::get_wide(Opcode::Umull, c, rn | rm << 4, rd_lo | rd_hi << 4); // A7.7.204 T1
            if (rd_lo == 13 || rd_lo == 15) || (rd_hi == 13 || rd_hi == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || rd_hi == rd_lo {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b011, 0b1111) => {
            let mut base = tag::get_wide(Opcode::Udiv, c, rd | rn << 4, rm); // A7.7.195 T1
            if rd_lo != 15 || (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b100, 0b0000) => {
            let mut base = tag::get_wide(Opcode::Smlal, c, rn | rm << 4, rd_lo | rd_hi << 4); // A7.7.138 T1
            if (rd_lo == 13 || rd_lo == 15) || (rd_hi == 13 || rd_hi == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || rd_hi == rd_lo {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b110, 0b0000) => {
            let mut base = tag::get_wide(Opcode::Umlal, c, rn | rm << 4, rd_lo | rd_hi << 4); // A7.7.203 T1
            if (rd_lo == 13 || rd_lo == 15) || (rd_hi == 13 || rd_hi == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || rd_hi == rd_lo {
                base = tag::as_unpred_w(base);
            }
            base
        }
        (0b110, 0b0110) => {
            let mut base = tag::get_wide(Opcode::Umaal, c, rn | rm << 4, rd_lo | rd_hi << 4); // A7.7.202 T1
            if (rd_lo == 13 || rd_lo == 15) || (rd_hi == 13 || rd_hi == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || rd_hi == rd_lo {
                base = tag::as_unpred_w(base);
            }
            base
        }
        _ => tag::get_unimplemented_wide(c, word),
    };
}

fn id_multiply_diff(word: u32, c: Context) -> ByteInstruction {
    // A5.3.16 // TODO
    assert!(matches(word, 23, 0b111_1111_11, 0b111_1101_10));
    if (word >> 6) & 0b11 != 0b00 {
        return tag::get_undefined_wide(c, word);
    }

    let op1 = (word >> 20) & 0b111;
    let op2 = (word >> 4) & 0b11;
    let ra = (word >> 12) & 0xF;

    let rn = (word >> 16) & 0xF;
    let rd = (word >> 8) & 0xF;
    let rm = word & 0xF;

    return match op1 {
        0b000 => {
            match op2 {
                0b00 if ra != 15 => {
                    let mut base = tag::get_wide(Opcode::Mla, c, rd | rn << 4, rm | ra << 4); // A7.7.74 T1
                    if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || ra == 13 {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b00 if ra == 15 => {
                    let mut base = tag::get_wide(Opcode::Mul, c, rd | rn << 4, rm); // A7.7.84 T1
                    if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                0b01 => {
                    let mut base = tag::get_wide(Opcode::Mls, c, rd | rn << 4, rm | ra << 4); // A7.7.75 T1
                    if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) || (ra == 13 || ra == 15) {
                        base = tag::as_unpred_w(base);
                    }
                    base
                }
                _ => tag::get_undefined_wide(c, word),
            }
        }
        _ => tag::get_unimplemented_wide(c, word),
    }
}

fn id_data_proc_register(word: u32, c: Context) -> ByteInstruction {
    // A5.3.12 // TODO
    assert!(matches(word, 24, 0b111_1111_1, 0b111_1101_0));
    if (word >> 12) & 0xF != 0b1111 {
        return tag::get_undefined_wide(c, word);
    }

    let op1 = (word >> 20) & 0xF;
    let rn = (word >> 16) & 0xF;
    let op2 = (word >> 4) & 0xF;

    let rd = (word >> 8) & 0xF;
    let rm = word & 0xF;
    let s = (word >> 20) & 0b1;

    if (op1 == 0b0000 || op1 == 0b0001) && op2 == 0b0000 {
        let mut base = tag::get_wide(Opcode::LslReg, c, rd | rn << 4 | s << 8, rm); // A7.7.69 T2
        if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 == 0b0010 || op1 == 0b0011) && op2 == 0b0000 {
        let mut base = tag::get_wide(Opcode::LsrReg, c, rd | rn << 4 | s << 8, rm); // A7.7.71 T2
        if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 == 0b0100 || op1 == 0b0101) && op2 == 0b0000 {
        let mut base = tag::get_wide(Opcode::AsrReg, c, rd | rn << 4 | s << 8, rm); // A7.7.11 T2
        if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 == 0b0110 || op1 == 0b0111) && op2 == 0b0000 {
        let mut base = tag::get_wide(Opcode::RorReg, c, rd | rn << 4 | s << 8, rm); // A7.7.117 T2
        if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if (op1 >> 2 == 0b10) && (op2 >> 2 == 0b10) {
        // A5.3.15 Miscellaneous operations
        let op1 = op1 & 0b11;
        let op2 = op2 & 0b11;

        return match (op1, op2) {
            (0b00, 0b00) => {
                let mut base = tag::get_wide(Opcode::Qadd, c, rd | rn << 4, rm); // A7.7.102 T1
                if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b00, 0b01) => {
                let mut base = tag::get_wide(Opcode::Qdadd, c, rd | rn << 4, rm); // A7.7.106 T1
                if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b00, 0b10) => {
                let mut base = tag::get_wide(Opcode::Qsub, c, rd | rn << 4, rm); // A7.7.109 T1
                if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b00, 0b11) => {
                let mut base = tag::get_wide(Opcode::Qdsub, c, rd | rn << 4, rm); // A7.7.107 T1
                if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b01, 0b00) => {
                let mut base = tag::get_wide(Opcode::Rev, c, rd, rm); // A7.7.113 T2
                if rn != rm || (rd == 13 || rd == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b01, 0b01) => {
                let mut base = tag::get_wide(Opcode::Rev16, c, rd, rm); // A7.7.114 T2
                if rn != rm || (rd == 13 || rd == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b01, 0b10) => {
                let mut base = tag::get_wide(Opcode::Rbit, c, rd, rm); // A7.7.112 T1
                if rn != rm || (rd == 13 || rd == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b01, 0b11) => {
                let mut base = tag::get_wide(Opcode::Revsh, c, rd, rm); // A7.7.115 T2
                if rn != rm || (rd == 13 || rd == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b10, 0b00) => {
                let mut base = tag::get_wide(Opcode::Sel, c, rd | rn << 4, rm); // A7.7.128 T1
                if (rd == 13 || rd == 15) || (rn == 13 || rn == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            (0b11, 0b00) => {
                let mut base = tag::get_wide(Opcode::Clz, c, rd, rm); // A7.7.24 T2
                if rn != rm || (rd == 13 || rd == 15) || (rm == 13 || rm == 15) {
                    base = tag::as_unpred_w(base);
                }
                base
            }
            _ => tag::get_undefined_wide(c, word),
        }
    }

    return tag::get_unimplemented_wide(c, word);
}

fn id_store_single(word: u32, c: Context) -> ByteInstruction {
    // A5.3.10 // DONE
    assert!(matches(word, 20, 0b111_1111_1_000_1, 0b111_1100_0_000_0));

    let op1 = (word >> 21) & 0x7;
    let op2 = bitset(word, 11);

    let rn = (word >> 16) & 0xF;
    let rt = (word >> 12) & 0xF;
    let rm = word & 0xF;
    let imm12 = word & 0xFFF;
    let imm8 = word & 0xFF;
    let p = (word >> 10) & 0b1;
    let u = (word >> 9) & 0b1;
    let w = (word >> 8) & 0b1;
    let imm2 = (word >> 4) & 0b11;

    let imm13 = if u == 1 {
        imm8
    } else {
        get_negated_simm13(imm8)
    };

    return match op1 {
        0b100 => {
            let mut base = tag::get_wide(Opcode::StrbImm, c, rt | rn << 4, imm12 | 1 << 14 | 0 << 13); // A7.7.163 T2
            if rt == 13 || rt == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b000 if op2 => {
            let mut base = tag::get_wide(Opcode::StrbImm, c, rt | rn << 4, imm13 | p << 14 | w << 13); // A7.7.163 T3
            if (rt == 13 || rt == 15) || (w == 1 && rn == rt) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b000 if !op2 => {
            let mut base = tag::get_wide(Opcode::StrbReg, c, rt | rn << 4, rm | imm2 << 4); // A7.7.164 T2
            if (rt == 13 || rt == 15) || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b101 => {
            let mut base = tag::get_wide(Opcode::StrhImm, c, rt | rn << 4, imm12 | 1 << 14 | 0 << 13); // A7.7.170 T2
            if rt == 13 || rt == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b001 if op2 => {
            let mut base = tag::get_wide(Opcode::StrhImm, c, rt | rn << 4, imm13 | p << 14 | w << 13); // A7.7.170 T3
            if (rt == 13 || rt == 15) || (w == 1 && rn == rt) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b001 if !op2 => {
            let mut base = tag::get_wide(Opcode::StrhReg, c, rt | rn << 4, rm | imm2 << 4); // A7.7.171 T2
            if (rt == 13 || rt == 15) || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b110 => {
            let mut base = tag::get_wide(Opcode::StrImm, c, rt | rn << 4, imm12 | 1 << 14 | 0 << 13); // A7.7.161 T3
            if rt == 15 {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b010 if op2 => {
            let mut base = if rn == 13 && p == 1 && u == 0 && w == 1 && imm8 == 0b00000100 {
                tag::get_wide(Opcode::Push, c, 1, rt) // A7.7.101 T4
            } else {
                tag::get_wide(Opcode::StrImm, c, rt | rn << 4, imm13 | p << 14 | w << 13) // A7.7.161 T4
            };
            if rt == 15 || (w == 1 && rn == rt) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        0b010 if !op2 => {
            let mut base = tag::get_wide(Opcode::StrReg, c, rt | rn << 4, rm | imm2 << 4); // A7.7.162 T2
            if rt == 15 || (rm == 13 || rm == 15) {
                base = tag::as_unpred_w(base);
            }
            base
        }
        _ => tag::get_undefined_wide(c, word),
    }
}

fn id_load_byte(word: u32, c: Context) -> ByteInstruction {
    // A5.3.9
    assert!(matches(word, 20, 0b111_1111_00_11_1, 0b111_1100_00_00_1));

    let _op1 = (word >> 23) & 0b11;
    let _rn = (word >> 16) & 0xF;
    let _rt = (word >> 12) & 0xF;
    let _op2 = (word >> 6) & 0x3F;

    return tag::get_unimplemented_wide(c, word);
}

fn id_load_half_word(word: u32, c: Context) -> ByteInstruction {
    return tag::get_unimplemented_wide(c, word);
}

// imm12 -> negate and mask to imm13
fn get_negated_simm13(imm12: u32) -> u32 {
    return ((!(imm12 & 0xFFF)).wrapping_add(1)) & 0x1FFF;
}

fn msk(value: u32, mask: u32, expected: u32) -> bool {
    return (value & mask) == expected;
}

fn id_load_word(word: u32, c: Context) -> ByteInstruction {
    // A5.3.7
    assert!(matches(word, 20, 0b111_1111_00_11_1, 0b111_1100_00_10_1));

    let op1 = (word >> 23) & 0b11;
    let op2 = (word >> 6) & 0x3F;
    let rn = (word >> 16) & 0xF;
    let rt = (word >> 12) & 0xF;

    let rm = word & 0xF;
    let imm2 = (word >> 4) & 0b11;
    let imm12 = word & 0xFFF;
    let imm8 = word & 0xFF;
    let p = (word >> 10) & 0b1;
    let u = (word >> 9) & 0b1;
    let w = (word >> 8) & 0b1;

    let imm13 = if u == 1 {
        imm8
    } else {
        get_negated_simm13(imm8)
    };

    if op1 == 0b01 && rn != 15 {
        let mut base = tag::get_wide(Opcode::LdrImm, c, rt | rn << 4, imm12 | 1 << 14 | 0 << 13); // A7.7.43 T3
        if rt == 15 && c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    if op1 == 0b00 && (msk(op2, 0b100100, 0b100100) || msk(op2, 0b111100, 0b110000)) && rn != 15 {
        let mut base = if rn == 13 && p == 0 && u == 1 && w == 1 && imm8 == 0b0000_0100 {
            tag::get_wide(Opcode::Pop, c, 1, rt) // A7.7.99 T4
        } else {
            tag::get_wide(Opcode::LdrImm, c, rt | rn << 4, imm13 | p << 14 | w << 13) // A7.7.43 T4
        };
        if w == 1 && rn == rt {
            base = tag::as_unpred_w(base);
        }
        if rt == 15 && c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    if op1 == 0b00 && msk(op2, 0b111100, 0b111000) && rn != 15 {
        let mut base = tag::get_wide(Opcode::Ldrt, c, rt | rn << 4, imm8); // A7.7.67 T1
        if rt == 13 || rt == 15 {
            base = tag::as_unpred_w(base);
        }
        return base;
    }

    if op1 == 0b00 && op2 == 0b000000 && rn != 15 {
        let mut base = tag::get_wide(Opcode::LdrReg, c, rt | rn << 4, rm | imm2 << 4); // A7.7.45 T2
        if rm == 13 || rm == 15 {
            base = tag::as_unpred_w(base);
        }
        if rt == 15 && c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    if (op1 == 0b00 || op1 == 0b01) && rn == 15 {
        let imm13 = if bitset(word, 23) {
            imm12
        } else {
            get_negated_simm13(imm12)
        };
        let mut base = tag::get_wide(Opcode::LdrLit, c, rt, imm13); // A7.7.44 T2
        if rt == 15 && c.it_pos == ItPos::Within {
            base = tag::as_unpred_it_w(base);
        }
        return base;
    }

    return tag::get_undefined_wide(c, word);
}
