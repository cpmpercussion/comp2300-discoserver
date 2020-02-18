#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    Unimplemented, // N: orginal thumb[16], W: blank[16] - original thumb[32]
    AdcImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    AdcReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    AddImm, // N: blank[2]-rn[3]-rd[3]-imm8[8] / W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    AddReg, // N: blank[3]-setflags[1]-rn[4]-rm[4]-rd[4] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    AddSpImm, // N: blank[2]-rd[4]-offset[10]
    AddSpReg,
    Adr,    // N: blank[3]-rd[3]-offset[10] / W: blank[12]-rd[4]- + blank[17]-sign[1]-imm12[12]
    AndImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    AndReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    AsrImm, // N: blank[5]-shift[5]-rm[3]-rd[3] / W: blank[11]-setflags[1]-rm[4]-rd[4] + blank[24]-shift_n[6]
    AsrReg, // N: blank[10]-rm[3]-rdn[3]
    Branch, // N: blank[5]-imm11[11]
    BranchCond, // N: blank[4]-cond[4]-imm8[4]
    BicImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    BicReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    Bkpt,   // N: blank[8]-imm8[8]
    Bl,     // W: blank[16] + blank[6]-imm24[24]
    Blx,    // N: blank[12]-rm[4]
    Bx,     // N: blank[12]-rm[4]
    Cbz,    // N: blank[5]-nonzero[1]-rn[3]-imm7[7]
    Cdp,    // W: --unparsed--
    CmnImm, // W: blank[8]-rn[4]-spill[4] + modified[30]
    CmnReg, // N: blank[12]-rm[3]-rn[3] / W: blank[8]-rm[4]-rn[4] + blank[22]-shift_n[6]-shift_t[2]
    CmpImm, // N: blank[5]-rn[3]-imm8[8] / W: blank[8]-rn[4]-spill[4] + modified[30]
    CmpReg, // N: blank[10]-rm[4]-rn[4] / W: blank[8]-rm[4]-rn[4] + blank[22]-shift_n[6]-shift_t[2]
    Cps,    // N: blank[13]-nonzero[1]-I[1]-F[1]
    EorImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    EorReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    It,     // N: blank[8]-firstcond[4]-mask[4]
    LdcImm, // W: --unparsed--
    LdcLit, // W: --unparsed--
    Ldm,    // N: blank[4]-wback[1]-rn[3]-registers[8] / W: blank[12]-rn[4] + blank[13]-wback[1]-pc[1]-lr[1]-(sp)[1]-registers[13]
    Ldmdb,  // W: blank[12]-rn[4] + blank[13]-W[1]-registers[16]
    LdrImm, // N: rn[4]-rt[4]-imm8[8] / W: blank[8]-rn[4]-rt[4] + blank[15]-index[1]-wback[1]-imm13[13]
    LdrLit, // N: blank[3]-rt[3]-offset[10] / W: blank[12]-rt[4] + blank[17]-simm13[13]
    LdrReg, // N: blank[7]-rm[3]-rn[3]-rt[3] / W: blank[8]-rn[4]-rt[4] + blank[24]-l_shift[2]-rm[4]
    LdrbImm, // N: blank[5]-imm5[5]-rn[3]-rt[3]
    LdrbReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    LdrdImm, // W: blank[5]-P[1]-U[1]-W[1]-rt2[4]-rt[4] + blank[16]-rn[4]-imm10[10]
    Ldrex,  // W: blank[12]-rt[4] + blank[]-rn[4]-imm10[10]
    Ldrexb, // W: blank[12]-rt[4] + blank[26]-rn[4]
    Ldrexh, // W: blank[12]-rt[4] + blank[26]-rn[4]
    LdrhImm, // N: blank[4]-rn[3]-rt[3]-imm6[6]
    LdrhReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    LdrsbReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    LdrshReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    LslImm, // N: blank[5]-shift[5]-rm[3]-rd[3] / W: blank[7]-setflags[1]-rm[4]-rd[4] + blank[24]-shift_n[6]
    LslReg, // N: blank[10]-rm[3]-rdn[3]
    LsrImm, // N: blank[5]-shift[5]-rm[3]-rd[3]
    LsrReg, // N: blank[10]-rm[3]-rdn[3]
    Mcr,   // W: --unparsed--
    Mcrr,   // W: --unparsed--
    MovImm, // N: blank[5]-rd[3]-imm8[8] / W: blank[]-setflags[1]-rd[4]-spill[4] + modified[30]
    MovReg, // N: blank[7]-setflags[1]-rm[4]-rd[4] / W: blank[]-setflags[1]-rd[4] + blank[26]-rm[4]
    Movt,
    Mrc,    // W: --unparsed--
    Mrrc,   // W: --unparsed--
    Mul,    // N: blank[10]-rm[3]-rdn[3] / W: blank[8]-rn[4]-rd[4] + blank[26]-rm[4]
    MvnImm, // W: blank[7]-setflags[1]-rd[4]-spill[4] + modified[30]
    MvnReg, // N: blank[12]-rm[3]-rd[3] / W: blank[]-setflags[1]-rm[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    Nop,    // N: blank[16]
    OrnImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    OrnReg, // W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    OrrImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    OrrReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    Pkhbt,  // W: blank[4]-rm[4]-rn[4]-rd[4] + blank[]-tbform[1]-blank[1]-shift[6]
    Pop,    // N: blank[7]-pc[1]-regs[8] / W: blank[15]-mode[1] + (blank[15]-pc[1]-lr[1]-(sp)[1]-registers[13] | blank[26]-register[4])
    Push,   // N: blank[7]-lr[1]-regs[8] / W: blank[15]-mode[1] + (blank[15]-pc[1]-lr[1]-(sp)[1]-registers[13] | blank[26]-register[4])
    Rev,    // N: blank[12]-rm[3]-rd[3]
    Rev16,  // N: blank[12]-rm[3]-rd[3]
    Revsh,  // N: blank[12]-rm[3]-rd[3]
    RorImm,
    RorReg, // N: blank[10]-rm[3]-rdn[3]
    Rrx,
    RsbImm, // N: blank[12]-rn[3]-rd[3]
    RsbReg, // W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    SbcImm, // W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    SbcReg, // N: blank[10]-rm[3]-rdn[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_n[6]-shift_t[2]
    Sdiv,   // W: blank[8]-rn[4]-rd[4] + blank[26]-rm[4]
    Sev,
    Smull,  // W: blank[8]-rm[4]-rn[4] + blank[22]-rd_hi[4]-rd_lo[4]
    Ssat16, // W: blank[8]-rn[4]-rd[4] + blank[25]-saturate_to[5]
    Stc,    // W: --unparsed--
    Stm,    // N: blank[5]-rt[3]-registers[8] / W: blank[12]-rn[4] + blank[13]-wback[1]-(pc)[1]-lr[1]-(sp)[1]-registers[13]
    Stmdb,  // W: blank[12]-rn[4] + blank[13]-W[1]-registers[16]
    StrImm, // N: rn[4]-rt[4]-imm8[8] / W: blank[8]-rn[4]-rt[4] + blank[15]-index[1]-wback[1]-imm13[13]
    StrReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    StrbImm, // N: blank[5]-imm5[5]-rn[3]-rt[3]
    StrbReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    StrhImm, // N: blank[4]-rn[3]-rt[3]-imm6[6]
    StrhReg, // N: blank[7]-rm[3]-rn[3]-rt[3]
    StrdImm, // W: blank[5]-P[1]-U[1]-W[1]-rt2[4]-rt[4] + blank[16]-rn[4]-imm10[10]
    Strex,  // W: blank[8]-rd[4]-rt[4] + blank[16]-rn[4]-imm10[10]
    Strexb, // W: blank[8]-rd[4]-rt[4] + blank[26]-rn[4]
    Strexh, // W: blank[8]-rd[4]-rt[4] + blank[26]-rn[4]
    SubImm, // N: blank[2]-rn[3]-rd[3]-imm8[8] / W: blank[3]-setflags[1]-rn[4]-rd[4]-spill[4] + modified[30]
    SubReg, // N: blank[7]-rm[3]-rn[3]-rd[3] / W: blank[3]-setflags[1]-rm[4]-rn[4]-rd[4] + blank[22]-shift_t[2]-shift_n[6]
    SubSpImm, // N: blank[7]-imm9[9]
    Svc,    // N: blank[8]-imm8[8]
    Sxtb,   // N: blank[12]-rm[3]-rd[3]
    Sxth,   // N: blank[12]-rm[3]-rd[3]
    Tbb,    // W: blank[]-H[1]-rn[4] + blank[26]-rm[4]
    TeqImm, // W: blank[8]-rn[4]-spill[4] + modified[30]
    TeqReg, // W: blank[8]-rm[4]-rn[4] + blank[22]-shift_n[6]-shift_t[2]
    TstImm, // W: blank[8]-rn[4]-spill[4] + modified[30]
    TstReg, // N: blank[12]-rm[3]-rn[3] / W: blank[8]-rm[4]-rn[4] + blank[22]-shift_n[6]-shift_t[2]
    Udf,    // N: blank[8]-imm8[8]
    Udiv,
    Umull,  // W: blank[8]-rm[4]-rn[4] + blank[22]-rd_hi[4]-rd_lo[4]
    Uxtb,   // N: blank[12]-rm[3]-rd[3]
    Uxth,   // N: blank[12]-rm[3]-rd[3]
    Wfe,
    Wfi,
    Yield,
    Undefined,
    Other,
    // etc.
}

pub fn to_opcode(bits: u8) -> Opcode {
    return unsafe { std::mem::transmute::<u8, Opcode>(bits) };
}

pub fn from_opcode(opcode: Opcode) -> u8 {
    return opcode as u8;
}
