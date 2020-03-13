#![allow(dead_code)]

extern crate goblin;

mod peripherals;
use peripherals::Peripherals;

mod audio;
use audio::{AudioHandler};

mod bytecode;
use bytecode::{InstructionCache, InstructionContext, decode_thumb, tag, opcode::{Opcode}};

mod cpu;
use cpu::{CPU, ExecMode, Condition};

mod utils;
use utils::bits::{self, bitset, add_with_carry, shift, shift_c, align, word_align, sign_extend, shifted_sign_extend};

use goblin::elf::Elf;
use std::path::Path;
use std::hint::unreachable_unchecked;
use std::collections::HashMap;
use std::{fmt, fs, string::String, option::Option};

pub type ByteInstruction = (u32, u32); // Intermediate bytecode format for more efficient decode and execution

#[derive(Copy, Clone, Debug)]
enum RegFormat {
    Bin, // binary
    Oct, // octal
    Dec, // unsigned decimal
    Sig, // signed decimal
    Hex, // hexadecimal
}

#[derive(Debug)]
enum Exception {
    Reset,
    NonMaskableInterrupt,
    HardFault,
    MemManage,
    BusFault,
    UsageFault,
    DebugMonitor,
    SupervisorCall,
    PendSV,
    SysTick,
}

#[derive(Debug)]
enum AccessType {
    Normal,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Location {
    Flash(usize),
    Ram(usize),
    Ram2(usize),
    Peripheral(u32), // we keep the passed address, and resolve in more detail
}

#[derive(Debug)]
pub struct ExclusiveMonitors {
    region: Option<()>,
}

// NOTE: This implementation does not reflect the intricacy
//       of the actual process. It serves for just LDREX and
//       STREX support on a single processor though.
impl ExclusiveMonitors {
    fn new() -> ExclusiveMonitors {
        return ExclusiveMonitors {
            region: None,
        }
    }

    fn set_exclusive_monitors(&mut self, _address: u32, _size: u32) {
        self.region = Some(());
    }

    fn exclusive_monitors_pass(&mut self, address: u32, size: u32) -> Result<bool, Exception> {
        if address != bits::align(address, size) {
            // UFSR.UNALIGNED = ‘1’;
            return Err(Exception::UsageFault);
        }

        let passed = self.is_exclusive_local(address, size);
        if passed {
            self.clear_exclusive_local();
        }
        return Ok(passed);
    }

    fn is_exclusive_local(&self, _address: u32, _size: u32) -> bool {
        return match self.region {
            Some(()) => true,
            None => false,
        }
    }

    fn clear_exclusive_local(&mut self) {
        self.region = None;
    }
}

pub struct MemoryBus {
    flash: Box<[u8]>,
    data: Box<[u8]>,
    data2: Box<[u8]>,
    peripherals: Peripherals,
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "<data>");
    }
}

fn read_value(bank: &[u8], base: usize, size: usize) -> Result<u32, MemError> {
    assert!(size == 1 || size == 2 || size == 4);
    if base + size > bank.len() {
        // println!("{} + {} > {}", base, size, bank.len());
        return Err(MemError::OutOfBounds);
    }

    let mut result: u32 = 0;
    for i in (0..size).rev() {
        result = result << 8;
        result += bank[base + i] as u32;
    }
    return Ok(result);
}

fn write_value(mut value: u32, bank: &mut[u8], base: usize, size: usize) -> Result<(), MemError> {
    assert!(size == 1 || size == 2 || size == 4);
    if base + size > bank.len() {
        return Err(MemError::OutOfBounds);
    }

    for i in 0..size {
        bank[base + i] = (value & 0xFF) as u8;
        value = value >> 8;
    }
    return Ok(());
}

#[derive(Debug)]
pub enum MemError {
    OutOfBounds,
    ReadOnly,
    Unaligned,
    Unimplemented,
}

impl fmt::Display for MemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl MemoryBus {
    fn new() -> MemoryBus {
        return MemoryBus {
            flash: vec![0xFF; 1024 * 1024].into_boxed_slice(),
            data: vec![0xFF; 0x18000].into_boxed_slice(),
            data2: vec![0xFF; 0x8000].into_boxed_slice(),
            peripherals: Peripherals::new(),
        };
    }

    fn load_elf(&mut self, elf: Elf, bytes: &[u8]) -> Result<(), String> {
        for header in elf.program_headers.iter() {
            if header.p_type != goblin::elf::program_header::PT_LOAD {
                return Err(String::from("Unexpected program header type"));
            }

            if header.p_vaddr < 0x0800_0000 {
                continue;
            }

            let phys_adr = header.p_paddr as usize;
            let offset = header.p_offset as usize;
            let size = header.p_filesz as usize;

            let start_index = phys_adr - 0x0800_0000;

            if start_index + size > self.flash.len() {
                return Err(String::from("Flash too small to fit content"));
            }

            for i in 0..size {
                self.flash[i + start_index] = bytes[i + offset];
            }
        }
        return Ok(());
    }

    fn get_instr_word(&self, address: u32) -> Result<u32, String> {
        if 0x0800_0000 <= address && address <= 0x0800_0000 + (self.flash.len() as u32) {
            let base = (address - 0x0800_0000) as usize;
            let b1 = self.flash[base] as u32;
            let b2 = self.flash[base + 1] as u32;
            let b3 = self.flash[base + 2] as u32;
            let b4 = self.flash[base + 3] as u32;
            return Ok((b2 << 24) + (b1 << 16) + (b4 << 8) + b3);
        }

        return Err(format!("Out of bounds access for instruction address 0x{:08X}", address));
    }

    fn read_mem_a(&self, address: u32, size: usize) -> Result<u32, MemError> {
        // B2.3.4 p583
        return self.read_mem_a_with_priv(address, size, &AccessType::Normal);
    }

    fn read_mem_a_with_priv(&self, address: u32, size: usize, _access_type: &AccessType) -> Result<u32, MemError> {
        // B2.3.4 p583
        if address != align(address, size as u32) {
            // Set UFSR.UNALIGNED = true;
            println!("UsageFault: unaligned memory access");
            return Err(MemError::Unaligned);
        }

        // let memaddrdesc = validate_address(address, access_type, false); // TODO
        let location = self.address_to_physical(address)?;
        return match location {
            Location::Flash(i) => read_value(&*self.flash, i, size),
            Location::Ram(i) => read_value(&*self.data, i, size),
            Location::Ram2(i) => read_value(&*self.data2, i, size),
            Location::Peripheral(i) => self.peripherals.read(i, size),
        };
    }

    pub fn read_mem_u(&self, address: u32, size: usize) -> Result<u32, MemError> {
        // B2.3.5 p584
        return self.read_mem_u_with_priv(address, size, &AccessType::Normal);
    }

    fn read_mem_u_with_priv(&self, address: u32, size: usize, access_type: &AccessType) -> Result<u32, MemError> {
        // B2.3.5 p585
        if address == align(address, size as u32) {
            return self.read_mem_a_with_priv(address, size, access_type);
        } else if /* CCR.UNALIGN_TRP */ false {
            // USFR.UNALIGNED = true;
            return Err(MemError::Unaligned);
        } else {
            let mut result: u32 = 0;
            for i in 0..(size as u32) {
                result += self.read_mem_a_with_priv(address + i, 1, &access_type)? << (8 * i);
            }
            return Ok(result);
        }
    }

    fn address_to_physical(&self, address: u32) -> Result<Location, MemError> {
        let address = address as usize;
        let location = match address {
            0x0000_0000..=0x000F_FFFF => Location::Flash(address),
            0x0800_0000..=0x080F_FFFF => Location::Flash(address - 0x0800_0000),
            0x1000_0000..=0x1000_7FFF => Location::Flash(address - 0x1000_0000),
            0x2000_0000..=0x2001_7FFF => Location::Ram(address - 0x2000_0000),
            0x4000_0000..=0x5FFF_FFFF => Location::Peripheral(address as u32),
            _ => {
                return Err(MemError::OutOfBounds);
            }
        };
        return Ok(location);
    }

    fn write_mem_u(&mut self, address: u32, size: usize, value: u32) -> Result<(), MemError> {
        let location = self.address_to_physical(address)?;
        return match location {
            Location::Flash(_) => Err(MemError::ReadOnly),
            Location::Ram(i) => {
                write_value(value, &mut *self.data, i, size)
            }
            Location::Ram2(i) => {
                write_value(value, &mut *self.data2, i, size)
            }
            Location::Peripheral(_) => self.peripherals.write(address, value, size),
        }
    }

    fn write_mem_a(&mut self, address: u32, size: usize, value: u32) -> Result<(), MemError> {
        // TODO
        return self.write_mem_u(address, size, value);
    }
}

#[derive(Debug)]
pub struct Board {
    tick: u128,
    audio_handler: AudioHandler,
    instruction_cache: InstructionCache,
    pub cpu: CPU,
    pub memory: MemoryBus,
    register_formats: [RegFormat; 16],
    branch_map: HashMap<u32, String>,

    // WIP: Still needs testing against actual board
    exclusive_monitors: ExclusiveMonitors,

    // HACK: To trigger default handler
    pending_default_handler: std::cell::Cell<bool>,
}

/**
 * Basically a VM. Uses standard fetch-decode-execute on an intermediate bytecode format
 * better suited to detecting bad things like unpredictable instructions and jumping into
 * IT blocks, while also being more efficient to execute in software compared to the Thumb encoding
 */
impl Board {
    pub fn new() -> Board {
        return Board {
            tick: 0,
            audio_handler: AudioHandler::new(),
            cpu: CPU::new(),
            instruction_cache: InstructionCache::new(),
            memory: MemoryBus::new(),
            register_formats: [RegFormat::Hex; 16],
            branch_map: HashMap::new(),
            exclusive_monitors: ExclusiveMonitors::new(),
            pending_default_handler: std::cell::Cell::new(false),
        };
    }

    pub fn step(&mut self) -> Result<(), String> {
        match self.fetch() {
            Ok((i, w)) => {
                if let Err(e) = self.execute(i, w) {
                    println!("failed to execute instruction: {}", e);
                    self.pending_default_handler.set(true);
                }
            }
            Err(e) => {
                println!("failed to fetch instruction: {}", e);
                self.pending_default_handler.set(true);
            }
        };

        if self.pending_default_handler.get() {
            match self.try_goto_default_handler() {
                Ok(_) => {},
                Err(e) => {
                    println!("FATAL ERROR: {}", e);
                    return Err(e);
                }
            }
        }

        return Ok(());
    }

    pub fn step_n(&mut self, steps: u32) -> Result<(), String> {
        for _ in 0..steps {
            self.step()?;
        }
        return Ok(());
    }

    pub fn spawn_audio(&mut self) {
        self.audio_handler.spawn_audio();
    }

    pub fn spawn_buffered_audio(&mut self, buffer_ms_size: u32) {
        self.audio_handler.spawn_buffered_audio(buffer_ms_size);
    }

    fn get_default_handler(&self) -> Result<u32, String> {
        // TODO: Goto fault handler (but account for when it doesn't goto valid location)
        if let Ok(hard_fault_addr) = self.memory.read_mem_u(3 * 4, 4) {
            if 0x0800_0000 <= hard_fault_addr && hard_fault_addr <= 0x0800_0000 + (self.memory.flash.len() as u32) {
                return Ok(hard_fault_addr);
            }
        }
        return Err(format!("Could not find fault handler"));
    }

    fn try_goto_default_handler(&mut self) -> Result<(), String> {
        return match self.get_default_handler() {
            Ok(h) => {
                self.exclusive_monitors_clear();
                self.pending_default_handler.set(false);
                self.branch_write_pc(h);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn read_mem_u(&self, address: u32, size: usize) -> u32 {
        return match self.memory.read_mem_u(address, size) {
            Ok(v) => v,
            Err(_) => {
                self.pending_default_handler.set(true);
                0
            }
        }
    }

    fn read_mem_a(&self, address: u32, size: usize) -> u32 {
        return match self.memory.read_mem_a(address, size) {
            Ok(v) => v,
            Err(_) => {
                self.pending_default_handler.set(true);
                0
            }
        }
    }

    fn read_word(&self, address: u32) -> u32 {
        return self.read_mem_u(address, 4);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        self.write_mem_u(address, 4, value);
    }

    fn write_mem_u(&mut self, address: u32, size: usize, value: u32) {
        // NOTE: The board has a 1-2 step delay before going to going to handler,
        //       we just go immediately.
        if let Err(e) = self.memory.write_mem_u(address, size, value) {
            match e {
                MemError::OutOfBounds => self.pending_default_handler.set(true),
                MemError::ReadOnly => {
                    println!("attempted to write to readonly memory: 0x{:08X}", address);
                },
                MemError::Unaligned => {},
                MemError::Unimplemented => {
                    println!("EMULATOR ERROR: unimplemented memory write");
                }
            }
        }
    }

    /**
     * Fetch: This stage
     * 1. Retrieves the address of the instruction to be executed
     * 2. Updates the PC value visible to instructions to be this + 4
     * 3. Attempts to find a cached instruction
     * 3a. If not cached, fetches direct bytes and decodes into the intermediate bytecode format
     * 3b. Caches decoded instruction
     * 4. Updates instruction pointed to by instruction PC to next instruction
     * 5. Returns fetched intermediate bytecode instruction & bool of width
     */
    fn fetch(&mut self) -> Result<(ByteInstruction, bool), String> {
        let pc = self.cpu.update_instruction_address();
        let mut instruction = self.instruction_cache.get_cached(pc)?;
        let mut start = tag::from(instruction);
        if !tag::has_cached(start) {
            let raw = self.memory.get_instr_word(pc)?;
            let decoded = decode_thumb(raw, InstructionContext::new(pc, self.cpu.itstate.position()));
            instruction = decoded.0;
            start = tag::from(instruction);
            if decoded.1 {
                self.instruction_cache.write_cache_wide(pc, instruction);
            } else {
                self.instruction_cache.write_cache_narrow(pc, instruction);
            }
        }
        let wide = tag::is_wide(start);
        self.cpu.inc_pc(wide);
        return Ok((instruction, wide));
    }

    /**
     * Execute: Takes the instruction and opcode, and executes
     * the instruction based on the opcode. It assumes
     */
    fn execute(&mut self, instr: ByteInstruction, wide: bool) -> Result<(), String> {
        self.tick += 1;
        let opcode = tag::get_opcode(instr.0);
        let data = instr.0 & 0xFFFF;
        let extra = instr.1 & !(0b11 << 30);

        if self.cpu.itstate.active() {
            let execute = self.cpu.check_condition(self.cpu.itstate.condition());
            self.cpu.itstate.advance();
            if !execute {
                println!("IT condition failed");
                return Ok(());
            }
        }

        return if wide {
            self.execute_wide(opcode, data, extra)
        } else {
            self.execute_narrow(opcode, data)
        }
    }

    fn execute_wide(&mut self, opcode: Opcode, data: u32, extra: u32) -> Result<(), String> {
        match opcode {
            Opcode::AdcImm => self.w_adc_imm(data, extra),
            Opcode::AdcReg => self.w_adc_reg(data, extra),
            Opcode::AddImm => self.w_add_imm(data, extra),
            Opcode::AddReg => self.w_add_reg(data, extra),
            Opcode::Adr    => self.w_adr(data, extra),
            Opcode::AndImm => self.w_and_imm(data, extra),
            Opcode::AndReg => self.w_and_reg(data, extra),
            Opcode::AsrImm => self.w_asr_imm(data, extra),
            Opcode::AsrReg => self.w_asr_reg(data, extra),
            Opcode::Branch => self.w_branch(data, extra),
            Opcode::BranchCond => self.w_branch_cond(data, extra),
            Opcode::Bfc    => self.w_bfc(data, extra),
            Opcode::Bfi    => self.w_bfi(data, extra),
            Opcode::BicImm => self.w_bic_imm(data, extra),
            Opcode::BicReg => self.w_bic_reg(data, extra),
            Opcode::Bl     => self.w_bl(data, extra),
            Opcode::Clrex  => self.w_clrex(data, extra),
            Opcode::Clz    => self.w_clz(data, extra),
            Opcode::CmnImm => self.w_cmn_imm(data, extra),
            Opcode::CmnReg => self.w_cmn_reg(data, extra),
            Opcode::CmpImm => self.w_cmp_imm(data, extra),
            Opcode::CmpReg => self.w_cmp_reg(data, extra),
            Opcode::EorImm => self.w_eor_imm(data, extra),
            Opcode::EorReg => self.w_eor_reg(data, extra),
            Opcode::Ldm    => self.w_ldm(data, extra),
            Opcode::Ldmdb  => self.w_ldmdb(data, extra),
            Opcode::LdrImm => self.w_ldr_imm(data, extra),
            Opcode::LdrLit => self.w_ldr_lit(data, extra),
            Opcode::LdrReg => self.w_ldr_reg(data, extra),
            Opcode::Ldrex  => self.w_ldrex(data, extra),
            Opcode::LslImm => self.w_lsl_imm(data, extra),
            Opcode::LslReg => self.w_lsl_reg(data, extra),
            Opcode::LsrImm => self.w_lsr_imm(data, extra),
            Opcode::LsrReg => self.w_lsr_reg(data, extra),
            Opcode::Mla    => self.w_mla(data, extra),
            Opcode::Mls    => self.w_mls(data, extra),
            Opcode::MovImm => self.w_mov_imm(data, extra),
            Opcode::MovReg => self.w_mov_reg(data, extra),
            Opcode::Movt   => self.w_movt(data, extra),
            Opcode::Mul    => self.w_mul(data, extra),
            Opcode::MvnImm => self.w_mvn_imm(data, extra),
            Opcode::MvnReg => self.w_mvn_reg(data, extra),
            Opcode::OrnImm => self.w_orn_imm(data, extra),
            Opcode::OrnReg => self.w_orn_reg(data, extra),
            Opcode::OrrImm => self.w_orr_imm(data, extra),
            Opcode::OrrReg => self.w_orr_reg(data, extra),
            Opcode::Pkhbt  => self.w_pkhbt(data, extra),
            Opcode::Pop    => self.w_pop(data, extra),
            Opcode::Push   => self.w_push(data, extra),
            Opcode::Qadd   => self.w_qadd(data, extra),
            Opcode::Qsub   => self.w_qsub(data, extra),
            Opcode::Rbit   => self.w_rbit(data, extra),
            Opcode::Rev    => self.w_rev(data, extra),
            Opcode::Rev16  => self.w_rev16(data, extra),
            Opcode::Revsh  => self.w_revsh(data, extra),
            Opcode::RorImm => self.w_ror_imm(data, extra),
            Opcode::RorReg => self.w_ror_reg(data, extra),
            Opcode::Rrx    => self.w_rrx(data, extra),
            Opcode::RsbImm => self.w_rsb_imm(data, extra),
            Opcode::RsbReg => self.w_rsb_reg(data, extra),
            Opcode::SbcImm => self.w_sbc_imm(data, extra),
            Opcode::SbcReg => self.w_sbc_reg(data, extra),
            Opcode::Sdiv   => self.w_sdiv(data, extra),
            Opcode::Smlal  => self.w_smlal(data, extra),
            Opcode::Smull  => self.w_smull(data, extra),
            Opcode::Stm    => self.w_stm(data, extra),
            Opcode::Stmdb  => self.w_stmdb(data, extra),
            Opcode::StrImm => self.w_str_imm(data, extra),
            Opcode::StrReg => self.w_str_reg(data, extra),
            Opcode::Strex  => self.w_strex(data, extra),
            Opcode::SubImm => self.w_sub_imm(data, extra),
            Opcode::SubReg => self.w_sub_reg(data, extra),
            Opcode::Tbb    => self.w_tbb(data, extra),
            Opcode::TeqImm => self.w_teq_imm(data, extra),
            Opcode::TeqReg => self.w_teq_reg(data, extra),
            Opcode::TstImm => self.w_tst_imm(data, extra),
            Opcode::TstReg => self.w_tst_reg(data, extra),
            Opcode::Udf    => self.w_udf(data, extra),
            Opcode::Udiv   => self.w_udiv(data, extra),
            Opcode::Umaal  => self.w_umaal(data, extra),
            Opcode::Umlal  => self.w_umlal(data, extra),
            Opcode::Umull  => self.w_umull(data, extra),
            _ => {
                // unsafe { unreachable_unchecked() }
                println!("Unimplemented wide instruction {:?} : {:#06X} + {:#010X}", opcode, data, extra);
            }
        }
        return Ok(());
    }

    fn execute_narrow(&mut self, opcode: Opcode, data: u32) -> Result<(), String> {
        match opcode {
            Opcode::AdcReg => self.n_adc_reg(data),
            Opcode::AddImm => self.n_add_imm(data),
            Opcode::AddReg => self.n_add_reg(data),
            Opcode::AddSpImm => self.n_add_sp_imm(data),
            Opcode::Adr    => self.n_adr(data),
            Opcode::AndReg => self.n_and_reg(data),
            Opcode::AsrImm => self.n_asr_imm(data),
            Opcode::AsrReg => self.n_asr_reg(data),
            Opcode::Branch => self.n_branch(data),
            Opcode::BranchCond => self.n_branch_cond(data),
            Opcode::BicReg => self.n_bic_reg(data),
            Opcode::Bkpt   => self.n_bkpt(data),
            Opcode::Blx    => self.n_blx_reg(data),
            Opcode::Bx     => self.n_bx(data),
            Opcode::Cbz    => self.n_cbz(data),
            Opcode::CmnReg => self.n_cmn_reg(data),
            Opcode::CmpImm => self.n_cmp_imm(data),
            Opcode::CmpReg => self.n_cmp_reg(data),
            Opcode::Cps    => self.n_cps(data),
            Opcode::EorReg => self.n_eor_reg(data),
            Opcode::It     => self.n_it(data),
            Opcode::Ldm    => self.n_ldm(data),
            Opcode::LdrImm => self.n_ldr_imm(data),
            Opcode::LdrLit => self.n_ldr_lit(data),
            Opcode::LdrReg => self.n_ldr_reg(data),
            Opcode::LdrbImm => self.n_ldrb_imm(data),
            Opcode::LdrbReg => self.n_ldrb_reg(data),
            Opcode::LdrhImm => self.n_ldrh_imm(data),
            Opcode::LdrhReg => self.n_ldrh_reg(data),
            Opcode::LdrsbReg => self.n_ldrsb_reg(data),
            Opcode::LdrshReg => self.n_ldrsh_reg(data),
            Opcode::LslImm => self.n_lsl_imm(data),
            Opcode::LslReg => self.n_lsl_reg(data),
            Opcode::LsrImm => self.n_lsr_imm(data),
            Opcode::LsrReg => self.n_lsr_reg(data),
            Opcode::MovImm => self.n_mov_imm(data),
            Opcode::MovReg => self.n_mov_reg(data),
            Opcode::Mul    => self.n_mul(data),
            Opcode::MvnReg => self.n_mvn_reg(data),
            Opcode::Nop    => self.n_nop(data),
            Opcode::OrrReg => self.n_orr_reg(data),
            Opcode::Pop    => self.n_pop(data),
            Opcode::Push   => self.n_push(data),
            Opcode::Rev    => self.n_rev(data),
            Opcode::Rev16  => self.n_rev16(data),
            Opcode::Revsh  => self.n_revsh(data),
            Opcode::RorReg => self.n_ror_reg(data),
            Opcode::RsbImm => self.n_rsb_imm(data),
            Opcode::SbcReg => self.n_sbc_reg(data),
            Opcode::Stm    => self.n_stm(data),
            Opcode::StrImm => self.n_str_imm(data),
            Opcode::StrReg => self.n_str_reg(data),
            Opcode::StrbImm => self.n_strb_imm(data),
            Opcode::StrbReg => self.n_strb_reg(data),
            Opcode::StrhImm => self.n_strh_imm(data),
            Opcode::StrhReg => self.n_strh_reg(data),
            Opcode::SubImm => self.n_sub_imm(data),
            Opcode::SubReg => self.n_sub_reg(data),
            Opcode::SubSpImm => self.n_sub_sp_imm(data),
            Opcode::Svc    => self.n_svc(data),
            Opcode::Sxtb   => self.n_sxtb(data),
            Opcode::Sxth   => self.n_sxth(data),
            Opcode::TstReg => self.n_tst_reg(data),
            Opcode::Udf    => self.n_udf(data),
            Opcode::Uxtb   => self.n_uxtb(data),
            Opcode::Uxth   => self.n_uxth(data),
            _ => {
                // unsafe { unreachable_unchecked() }
                println!("Unimplemented narrow instruction {:?} - {:#06X}", opcode, data);
            }
        }

        return Ok(());
    }

    /**
     * Takes a path to an ELF file and initialises the board with its contents
     */
    pub fn load_elf_from_path(&mut self, path: &Path) -> Result<(), String> {
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return Err(format!("Failed to read file \"{:?}\": {}", path, e));
            }
        };

        let elf = match Elf::parse(&bytes) {
            Ok(e) => e,
            Err(e) => {
                return Err(format!("Failed to parse elf file \"{:?}\": {}", path, e));
            }
        };

        for sym in elf.syms.iter() {
            let offset = sym.st_name;
            let name = match elf.strtab.get(offset) {
                Some(Ok(s)) => s,
                _ => return Err(String::from("missing symbols")),
            };

            match name {
                "SystemInit" |
                "__libc_init_array" |
                "init" |
                "audio_init" |
                "audio_play_sample" |
                "init_joystick" |
                "joystick_init_all" |
                "joystick_enable_interrupts_all" |
                "lcd_init" |
                "lcd_write_char" |
                "lcd_write_string" |
                "lcd_update_display" |
                "maximise_clock_speed" |
                "BSP_AUDIO_OUT_Play_Sample" => {
                    self.branch_map.insert((sym.st_value as u32) & !0b1, name.to_string());
                }
                _ => {}
            }
        }

        match self.memory.load_elf(elf, &bytes) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        // https://developer.arm.com/docs/dui0553/a/the-cortex-m4-processor/programmers-model/core-registers
        self.cpu.write_reg(13, self.memory.read_mem_a(0x0000_0000, 4).expect("failed to read memory at 0x0000_0000")); // set to value at address 0x0000_0000 on reset
        self.cpu.write_reg(14, 0xFFFF_FFFF); // set to 0xFFFF_FFFF on reset
        let pc = self.memory.read_mem_a(0x0000_0004, 4).expect("failed to read memory at 0x0000_0004"); // set to value at 0x0000_0004 on reset
        println!("setting emulator pc to 0x{:08X}", pc);
        self.cpu.write_reg(15, pc & !0b1);
        self.bx_write_pc(pc);

        return Ok(());
    }

    pub fn read_memory_region(&self, start: u32, bytes: u32) -> Result<Vec<u8>, String> {
        let mut out: Vec<u8> = Vec::new();
        for i in start..(start.saturating_add(bytes)) {
            match self.memory.read_mem_u(i, 1) {
                Ok(i) => out.push(i as u8),
                Err(_) => {
                    return Ok(vec![0; bytes as usize]);
                }
            };
        }
        return Ok(out);
    }

    pub fn read_reg<T: Into<u32>>(&self, reg: T) -> u32 {
        let reg = reg.into();
        return self.cpu.read_reg(reg);
    }

    fn write_reg<T: Into<u32>>(&mut self, reg: T, val: u32) {
        // TODO: Follow B1.4.7 p521
        let reg = reg.into();
        self.cpu.write_reg(reg, val);
    }

    fn get_register_display_value(&self, reg: u8) -> String {
        assert!(reg <= 15);
        let val = match reg {
            15 => self.cpu.read_instruction_pc(),
            _ => self.read_reg(reg),
        };
        return match self.register_formats[reg as usize] {
            RegFormat::Bin => format!("{:#034b}", val),
            RegFormat::Oct => format!("{:#013o}", val),
            RegFormat::Dec => format!("{}", val),
            RegFormat::Sig => format!("{}", val as i32),
            RegFormat::Hex => format!("{:#010X}", val),
        };
    }

    pub fn read_sp(&self) -> u32 {
        return self.cpu.read_sp();
    }

    fn write_sp(&mut self, value: u32) {
        self.cpu.write_sp(value);
    }

    pub fn read_lr(&self) -> u32 {
        return self.cpu.read_lr();
    }

    fn write_lr(&mut self, value: u32) {
        self.cpu.write_lr(value);
    }

    pub fn read_pc(&self) -> u32 {
        return self.cpu.read_pc();
    }

    fn set_pc(&mut self, address: u32) {
        self.cpu.write_instruction_pc(address);
    }

    fn get_shifted_register(&self, reg_val: u32, shift_t: u32, shift_n: u32) -> u32 {
        return shift(reg_val, shift_t, shift_n, self.cpu.read_carry_flag() as u32);
    }

    fn get_shift_with_carry(&self, reg_val: u32, shift_t: u32, shift_n: u32) -> (u32, bool) {
        return shift_c(reg_val, shift_t, shift_n, self.cpu.read_carry_flag() as u32);
    }

    fn lsl_c(&self, reg_val: u32, shift_n: u32) -> (u32, bool) {
        return if shift_n == 0 {
            (reg_val, self.cpu.read_carry_flag())
        } else {
            bits::lsl_c(reg_val, shift_n)
        }
    }

    fn lsr_c(&self, reg_val: u32, shift_n: u32) -> (u32, bool) {
        return if shift_n == 0 {
            (reg_val, self.cpu.read_carry_flag())
        } else {
            bits::lsr_c(reg_val, shift_n)
        }
    }

    fn asr_c(&self, reg_val: u32, shift_n: u32) -> (u32, bool) {
        return if shift_n == 0 {
            (reg_val, self.cpu.read_carry_flag())
        } else {
            bits::asr_c(reg_val, shift_n)
        }
    }

    fn ror_c(&self, reg_val: u32, shift_n: u32) -> (u32, bool) {
        return if shift_n == 0 {
            (reg_val, self.cpu.read_carry_flag())
        } else {
            bits::ror_c(reg_val, shift_n)
        }
    }

    fn add_with_carry_w_c(&self, reg_val: u32, imm32: u32) -> (u32, bool, bool) {
        return add_with_carry(reg_val, imm32, u32::from(self.cpu.read_carry_flag()));
    }

    fn set_flags_nz(&mut self, result: u32) {
        self.cpu.set_negative_flag(bitset(result, 31));
        self.cpu.set_zero_flag(result == 0);
        // c unchanged
        // v unchanged
    }

    fn set_flags_nzc(&mut self, result: u32, carry: bool) {
        self.set_flags_nz(result);
        self.cpu.set_carry_flag(carry);
        // v unchanged
    }

    fn set_flags_nzcv(&mut self, result: u32, carry: bool, overflow: bool) {
        self.set_flags_nzc(result, carry);
        self.cpu.set_overflow_flag(overflow);
    }

    fn set_flags_nz_alt_c(&mut self, result: u32, spill_tag: u32) {
        self.set_flags_nz(result);
        match (spill_tag >> 2) & 0b11 {
            0b00 => {}
            0b01 => {}
            0b10 => self.cpu.set_carry_flag(false),
            0b11 => self.cpu.set_carry_flag(true),
            _ => unsafe { unreachable_unchecked() },
        }
        // v unchanged
    }

    pub fn in_it_block(&self) -> bool {
        return self.cpu.itstate.active();
    }

    /**
     * Helper pseudocode functions
     */

    fn branch_to(&mut self, address: u32) {
        // B1.4.7 p522
        self.set_pc(address);
    }

    fn branch_write_pc(&mut self, address: u32) {
        // A2.3.1 p30
        self.branch_to(address & !0b1);
    }

    fn bx_write_pc(&mut self, address: u32) {
        // A2.3.1 p31
        if self.cpu.current_mode == ExecMode::ModeHandler && (address >> 28) == 0xF {
            println!("TODO: ExceptionReturn(address & !(0xF << 28))");
            self.pending_default_handler.set(true);
        } else {
            self.blx_write_pc(address);
        }
    }

    fn blx_write_pc(&mut self, address: u32) {
        // A2.3.1 p31
        self.cpu.set_thumb_mode(bitset(address, 0));
        if !self.cpu.read_thumb_mode() {
            println!("self.raise_exception(Exception::UsageFault('Invalid State'))");
            self.pending_default_handler.set(true);
        }
        self.branch_to(address & !0b1);
    }

    fn load_write_pc(&mut self, address: u32) {
        // A2.3.1 p31
        self.bx_write_pc(address);
    }

    fn alu_write_pc(&mut self, address: u32) {
        // A2.3.1 p31
        self.branch_write_pc(address);
    }

    fn set_exclusive_monitors(&mut self, address: u32, length: u32) {
        self.exclusive_monitors.set_exclusive_monitors(address, length);
    }

    fn exclusive_monitors_pass(&mut self, address: u32, length: u32) -> bool {
        return match self.exclusive_monitors.exclusive_monitors_pass(address, length) {
            Ok(passed) => passed,
            Err(_) => {
                self.pending_default_handler.set(true);
                return false;
            }
        }
    }

    fn exclusive_monitors_clear(&mut self) {
        self.exclusive_monitors.clear_exclusive_local();
    }

    /**
     * Instruction handlers
     */

    fn w_adc_imm(&mut self, data: u32, extra: u32) {
        // A7.7.1
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let (result, carry, overflow) = self.add_with_carry_w_c(self.read_reg(rn), imm32);
        self.write_reg(rd, result);
        if bitset(data, 12) {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_adc_reg(&mut self, data: u32) {
        // A7.7.2
        let rd = data & 0b111;
        let rm = data >> 3;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rd), self.read_reg(rm), self.cpu.carry());
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_adc_reg(&mut self, data: u32, extra: u32) {
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = self.add_with_carry_w_c(self.read_reg(rn), shifted);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_add_imm(&mut self, data: u32) {
        // A7.7.3
        let imm32 = data & 0xFF;
        let rn = data >> 11;
        let rd = (data >> 8) & 0b111;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), imm32, 0);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_add_imm(&mut self, data: u32, extra: u32) {
        // A7.7.3
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), imm32, 0);
        self.write_reg(rd, result);
        if bitset(data, 12) {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_add_reg(&mut self, data: u32) {
        // A7.7.4
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), self.read_reg(rm), 0);
        if rd == 15 {
            self.alu_write_pc(result);
        } else {
            self.write_reg(rd, result);
            if (data >> 12) > 0 {
                self.set_flags_nzcv(result, carry, overflow);
            }
        }
    }

    fn w_add_reg(&mut self, data: u32, extra: u32) {
        // A7.7.4
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), shifted, 0);
        if rd == 15 {
            self.alu_write_pc(result);
        } else {
            self.write_reg(rd, result);
            if setflags {
                self.set_flags_nzcv(result, carry, overflow);
            }
        }
    }

    fn n_add_sp_imm(&mut self, data: u32) {
        // A7.7.5
        let imm10 = data & 0x3FF;
        let rd = data >> 10;
        self.write_reg(rd, self.read_sp().wrapping_add(imm10));
    }

    fn n_adr(&mut self, data: u32) {
        let imm10 = data & 0x3FF;
        let rd = data >> 10;
        let result = word_align(self.read_pc()).wrapping_add(imm10);
        self.write_reg(rd, result);
    }

    fn w_adr(&mut self, data: u32, extra: u32) {
        let rd = data;
        let imm32 = sign_extend(12, extra);
        let result = word_align(self.read_pc()).wrapping_add(imm32);
        self.write_reg(rd, result);
    }

    fn w_and_imm(&mut self, data: u32, extra: u32) {
        // A7.7.8
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let result = self.read_reg(rn) & imm32;
        self.write_reg(rd, result);
        if bitset(data, 12) {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn n_and_reg(&mut self, data: u32) {
        // A7.7.9
        let rd = data & 0b111;
        let rm = data >> 3;
        let result = self.read_reg(rd) & self.read_reg(rm);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_and_reg(&mut self, data: u32, extra: u32) {
        // A7.7.9
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) & shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_asr_imm(&mut self, data: u32) {
        // A7.7.10
        let rd = data & 0x7;
        let rm = (data >> 3) & 0x7;
        let shift_n = data >> 6;

        let (result, carry) = self.asr_c(self.read_reg(rm), shift_n);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_asr_imm(&mut self, data: u32, extra: u32) {
        // A7.7.10
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let shift_n = extra;

        let (result, carry) = self.asr_c(self.read_reg(rm), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_asr_reg(&mut self, data: u32) {
        // A7.7.11
        let rdn = data & 0x7;
        let rm = data >> 3;

        let shift = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.asr_c(self.read_reg(rdn), shift);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_asr_reg(&mut self, data: u32, extra: u32) {
        // A7.7.11
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let rm = extra;

        let shift_n = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.asr_c(self.read_reg(rn), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_branch(&mut self, data: u32) {
        // A7.7.12
        let imm11 = data;
        let imm32 = shifted_sign_extend(imm11, 10, 1);
        self.branch_write_pc(self.read_pc().wrapping_add(imm32));
    }

    fn w_branch(&mut self, _data: u32, extra: u32) {
        // A7.7.12
        let imm24 = extra;
        let imm32 = shifted_sign_extend(imm24, 23, 1);
        self.branch_write_pc(self.read_pc().wrapping_add(imm32));
    }

    fn n_branch_cond(&mut self, data: u32) {
        // A7.7.12
        if self.cpu.check_condition(Condition::new(data >> 8)) {
            self.branch_write_pc(self.read_pc().wrapping_add(shifted_sign_extend(data, 7, 1)));
        }
    }

    fn w_branch_cond(&mut self, data: u32, extra: u32) {
        // A7.7.12
        if self.cpu.check_condition(Condition::new(data)) {
            self.branch_write_pc(self.read_pc().wrapping_add(shifted_sign_extend(extra, 19, 1)));
        }
    }

    fn w_bfc(&mut self, data: u32, extra: u32) {
        // A7.7.13
        let rd = data;
        let msbit = extra & 0x1F;
        let lsbit = extra >> 5;

        if msbit >= lsbit {
            let result = bits::bit_field_clear(self.read_reg(rd), msbit, lsbit);
            self.write_reg(rd, result);
        } else {
            println!("UNPREDICTABLE BFC");
            self.pending_default_handler.set(true);
        }
    }

    fn w_bfi(&mut self, data: u32, extra: u32) {
        // A7.7.14
        let rd = data & 0xF;
        let rn = data >> 4;
        let msbit = extra & 0x1F;
        let lsbit = extra >> 5;

        if msbit >= lsbit {
            let result = bits::bit_field_insert(self.read_reg(rd), self.read_reg(rn), msbit, lsbit);
            self.write_reg(rd, result);
        } else {
            println!("UNPREDICTABLE BFC");
            self.pending_default_handler.set(true);
        }
    }

    fn w_bic_imm(&mut self, data: u32, extra: u32) {
        // A7.7.15
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let result = self.read_reg(rn) & !imm32;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn n_bic_reg(&mut self, data: u32) {
        // A7.7.16
        let rdn = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rdn) & !self.read_reg(rm);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_bic_reg(&mut self, data: u32, extra: u32) {
        // A7.7.16
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) & !shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_bkpt(&mut self, _data: u32) {
        // A7.7.17
        // TODO: When return values supported, cause a DebugMonitor exception with the input id
    }

    fn w_bl(&mut self, _data: u32, extra: u32) {
        // A7.7.18
        let pc = self.read_pc();
        self.write_lr(pc | 0b1);
        let address = pc.wrapping_add(shifted_sign_extend(extra, 23, 1));
        match self.branch_map.get(&address) {
            Some(name) => {
                if name == "BSP_AUDIO_OUT_Play_Sample" || name == "audio_play_sample" {
                    self.audio_handler.handle((self.read_reg(0u32) & 0xFFFF) as i16);
                } else {
                    println!("Skipping call to {}", name);
                }
            }
            None => {
                self.branch_write_pc(address);
            }
        }
    }

    fn n_blx_reg(&mut self, data: u32) {
        // A7.7.19
        let target = self.read_reg(data);
        let next_instr_address = self.read_pc() - 2;
        self.write_lr(next_instr_address | 0b1);
        self.blx_write_pc(target);
    }

    fn n_bx(&mut self, data: u32) {
        // A7.7.20
        let rm = data;
        self.bx_write_pc(self.read_reg(rm));
    }

    fn n_cbz(&mut self, data: u32) {
        // A7.7.21
        let imm7 = data & 0x7F;
        let rn = (data >> 7) & 0x7;
        let nonzero = bitset(data, 10);
        if nonzero != (self.read_reg(rn) == 0) {
            self.branch_write_pc(self.read_pc() + imm7);
        }
    }

    fn w_clrex(&mut self, _data: u32, _extra: u32) {
        self.exclusive_monitors_clear();
    }

    fn w_clz(&mut self, data: u32, extra: u32) {
        // A7.7.24
        let rd = data;
        let rm = extra;

        self.write_reg(rd, self.read_reg(rm).leading_zeros());
    }

    fn w_cmn_imm(&mut self, data: u32, extra: u32) {
        // A7.7.25
        let imm32 = data << 30 | extra;
        let rn = (data >> 4) & 0xF;

        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), imm32, 0);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn n_cmn_reg(&mut self, data: u32) {
        // A7.7.26
        let rn = data & 0x7;
        let rm = data >> 3;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), self.read_reg(rm), 0);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn w_cmn_reg(&mut self, data: u32, extra: u32) {
        // A7.7.26
        let rn = data & 0xF;
        let rm = data >> 4;
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), shifted, 0);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn n_cmp_imm(&mut self, data: u32) {
        // A7.7.27
        let rn = data >> 8;
        let imm32 = data & 0xFF;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !imm32, 1);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn w_cmp_imm(&mut self, data: u32, extra: u32) {
        let imm32 = data << 30 | extra;
        let rn = data >> 4;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !imm32, 1);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn n_cmp_reg(&mut self, data: u32) {
        // A7.7.28
        let rn = data & 0xF;
        let rm = data >> 4;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !self.read_reg(rm), 1);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn w_cmp_reg(&mut self, data: u32, extra: u32) {
        // A7.7.28
        let rn = data & 0xF;
        let rm = data >> 4;
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !shifted, 0);
        self.set_flags_nzcv(result, carry, overflow);
    }

    fn n_cps(&mut self, _data: u32) {
        // A7.7.29
        // B5.2.1
        // TODO
    }

    // A7.7.30 is CPY, a deprecated alias for MOV

    fn w_eor_imm(&mut self, data: u32, extra: u32) {
        // A7.7.35
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let result = self.read_reg(rn) ^ imm32;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn n_eor_reg(&mut self, data: u32) {
        // A7.7.36
        let rdn = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rdn) ^ self.read_reg(rm);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_eor_reg(&mut self, data: u32, extra: u32) {
        // A7.7.36
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_n = extra >> 3;
        let shift_t = extra & 0b111;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) ^ shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_it(&mut self, data: u32) {
        // A7.7.38
        self.cpu.itstate.state = data;
    }

    fn n_ldm(&mut self, data: u32) {
        let rn = (data >> 8) & 0x7;
        let registers = data;
        let mut address = self.read_reg(rn);
        for i in 0..=7u32 {
            if bitset(registers, i) {
                self.write_reg(i, self.read_word(address));
                address += 4;
            }
        }
        if bitset(data, 11) {
            self.write_reg(rn, address);
        }
    }

    fn w_ldm(&mut self, data: u32, extra: u32) {
        // A7.7.41
        let rn = data;
        let registers = extra;
        let wback = bitset(extra, 16);

        let mut address = self.read_reg(rn);
        for i in 0..=14u32 { // TODO: Skip stack pointer
            if bitset(registers, i) {
                self.write_reg(i, self.read_word(address));
                address += 4;
            }
        }
        if bitset(registers, 15) {
            self.load_write_pc(self.read_word(address));
        }
        if wback && !bitset(registers, rn) {
            self.write_reg(rn, address);
        }
    }

    fn w_ldmdb(&mut self, data: u32, extra: u32) {
        // A7.7.42
        let rn = data;
        let registers = extra & 0xFFFF;
        let wback = bitset(extra, 16);

        let mut address = self.read_reg(rn) - 4 * registers.count_ones();
        for i in 0..=14u32 {
            if bitset(registers, i) {
                self.write_reg(i, self.read_mem_a(address, 4));
                address += 4;
            }
        }
        if bitset(registers, 15) {
            self.load_write_pc(self.read_mem_a(address, 4));
        }
        if wback && !bitset(registers, rn) {
            self.write_reg(rn, self.read_reg(rn) - 4 * registers.count_ones());
        }
    }

    fn n_ldr_imm(&mut self, data: u32) {
        let rt = (data >> 8) & 0xF;
        let rn = data >> 12;
        let imm32 = (data & 0xFF) << 2;
        let address = self.read_reg(rn).wrapping_add(imm32);
        self.write_reg(rt, self.read_word(address));
    }

    fn w_ldr_imm(&mut self, data: u32, extra: u32) {
        // A7.7.43
        let rt = data & 0xF;
        let rn = data >> 4;
        let index = bitset(extra, 14);
        let wback = bitset(extra, 13);

        let offset_address = self.read_reg(rn).wrapping_add(sign_extend(extra, 12));
        let address = if index { offset_address } else { self.read_reg(rn) };
        let data = self.read_word(address);
        if wback { self.write_reg(rn, offset_address); }
        if rt == 15 {
            if (address & 0b11) == 0 {
                self.load_write_pc(data);
            } else {
                println!("Unpredictable");
                self.pending_default_handler.set(true);
            }
        } else {
            self.write_reg(rt, data);
        }
    }

    fn n_ldr_lit(&mut self, data: u32) {
        let rt = data >> 10;
        let imm10 = data & 0x3FF;
        let address = word_align(self.read_pc()).wrapping_add(imm10);
        let value = self.read_word(address);
        if rt == 15 {
            if (address & 0b11) == 0 {
                self.load_write_pc(value);
            } else {
                println!("Unpredictable");
                self.pending_default_handler.set(true);
            }
        } else {
            self.write_reg(rt, value);
        }
    }

    fn w_ldr_lit(&mut self, data: u32, extra: u32) {
        // A7.7.44
        let rt = data;
        let address = word_align(self.read_pc()).wrapping_add(sign_extend(extra, 12));
        let data = self.read_word(address);
        if rt == 15 {
            if (address & 0b11) == 0 {
                self.load_write_pc(data);
            } else {
                println!("Unpredictable");
                self.pending_default_handler.set(true);
            }
        } else {
            self.write_reg(rt, data);
        }
    }

    fn n_ldr_reg(&mut self, data: u32) {
        // A7.7.45
        let rt = data & 0b111;
        let rn = (data >> 3) & 0b111;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        let value = self.read_word(address);
        self.write_reg(rt, value);
    }

    fn w_ldr_reg(&mut self, data: u32, extra: u32) {
        // A7.7.45
        let rt = data & 0xF;
        let rn = data >> 4;
        let rm = extra & 0xF;
        let shift_n = extra >> 4;

        let offset = self.read_reg(rm) << shift_n;

        // NOTE: Manual does not define `add`, `index`, or `wback` so we just assume it matches T1
        let address = self.read_reg(rn).wrapping_add(offset);
        let value = self.read_mem_u(address, 4);
        if rt == 15 {
            if address & 0b11 != 0 {
                println!("UNPREDICTABLE: ldr.W");
                self.pending_default_handler.set(true);
            }
            self.load_write_pc(value);
        } else {
            self.write_reg(rt, value);
        }
    }

    fn n_ldrb_imm(&mut self, data: u32) {
        // A7.7.46
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let imm5 = data >> 6;
        let address = self.read_reg(rn).wrapping_add(imm5);
        let loaded = self.read_mem_u(address, 1);
        self.write_reg(rt, loaded);
    }

    fn n_ldrb_reg(&mut self, data: u32) {
        // A7.7.48
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        let loaded = self.read_mem_u(address, 1);
        self.write_reg(rt, loaded);
    }

    fn w_ldrex(&mut self, data: u32, extra: u32) {
        let rt = data;
        let imm10 = extra & 0x3FF;
        let rn = extra >> 10;

        let address = self.read_reg(rn).wrapping_add(imm10);
        self.set_exclusive_monitors(address, 4);
        self.write_reg(rt, self.read_mem_a(address, 4));
    }

    fn n_ldrh_imm(&mut self, data: u32) {
        // A7.7.55
        let imm6 = data & 0x3F;
        let rt = (data >> 6) & 0x7;
        let rn = data >> 9;
        let address = self.read_reg(rn).wrapping_add(imm6);
        let loaded = self.read_mem_u(address, 2);
        self.write_reg(rt, loaded);
    }

    fn n_ldrh_reg(&mut self, data: u32) {
        // A7.7.57
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        let loaded = self.read_mem_u(address, 2);
        self.write_reg(rt, loaded);
    }

    fn n_ldrsb_reg(&mut self, data: u32) {
        // A7.7.61
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        let loaded = self.read_mem_u(address, 1);
        self.write_reg(rt, sign_extend(loaded, 7));
    }

    fn n_ldrsh_reg(&mut self, data: u32) {
        // A7.7.61
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        let loaded = self.read_mem_u(address, 2);
        self.write_reg(rt, sign_extend(loaded, 15));
    }

    fn n_lsl_imm(&mut self, data: u32) {
        // A7.7.68
        let rd = data & 0x7;
        let rm = (data >> 3) & 0x7;
        let shift = data >> 6;

        let (result, carry) = self.lsl_c(self.read_reg(rm), shift);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_lsl_imm(&mut self, data: u32, extra: u32) {
        // A7.7.68
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let shift_n = extra;

        let (result, carry) = self.lsl_c(self.read_reg(rm), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_lsl_reg(&mut self, data: u32) {
        // A7.7.69
        let rdn = data & 0x7;
        let rm = data >> 3;
        let shift = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.lsl_c(self.read_reg(rdn), shift);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_lsl_reg(&mut self, data: u32, extra: u32) {
        // A7.7.69
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let rm = extra;

        let shift_n = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.lsl_c(self.read_reg(rn), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_lsr_imm(&mut self, data: u32) {
        // A7.7.70
        let rd = data & 0x7;
        let rm = (data >> 3) & 0x7;
        let shift = data >> 6;

        let (result, carry) = self.lsr_c(self.read_reg(rm), shift);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_lsr_imm(&mut self, data: u32, extra: u32) {
        // A7.7.70
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let shift_n = extra;

        let (result, carry) = self.lsr_c(self.read_reg(rm), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_lsr_reg(&mut self, data: u32) {
        // A7.7.71
        let rdn = data & 0x7;
        let rm = data >> 3;

        let shift = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.lsr_c(self.read_reg(rdn), shift);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_lsr_reg(&mut self, data: u32, extra: u32) {
        // A7.7.71
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let rm = extra;

        let shift_n = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.lsr_c(self.read_reg(rn), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_mla(&mut self, data: u32, extra: u32) {
        // A7.7.74
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra & 0xF;
        let ra = extra >> 4;

        let op1 = self.read_reg(rn);
        let op2 = self.read_reg(rm);
        let addend = self.read_reg(ra);
        let result = op1.wrapping_mul(op2).wrapping_add(addend);
        self.write_reg(rd, result);
        // if setflags... wait... hmmm.
    }

    fn w_mls(&mut self, data: u32, extra: u32) {
        // A7.7.75
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra & 0xF;
        let ra = extra >> 4;

        let op1 = self.read_reg(rn);
        let op2 = self.read_reg(rm);
        let addend = self.read_reg(ra);
        let result = addend.wrapping_sub(op1.wrapping_mul(op2));
        self.write_reg(rd, result);
    }

    fn n_mov_imm(&mut self, data: u32) {
        // A7.7.76
        let rd = data >> 8;
        let imm8 = data & 0xFF;
        self.write_reg(rd, imm8);
        if !self.in_it_block() {
            self.set_flags_nz(imm8);
        }
    }

    fn w_mov_imm(&mut self, data: u32, extra: u32) {
        // A7.7.76
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        self.write_reg(rd, imm32);
        if bitset(data, 8) {
            self.set_flags_nz_alt_c(imm32, data);
        }
    }

    fn n_mov_reg(&mut self, data: u32) {
        // A7.7.77
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);

        let result = self.read_reg(rm);
        if rd == 15 {
            self.alu_write_pc(result);
        } else {
            self.write_reg(rd, result);
            if setflags {
                self.set_flags_nz(result);
            }
        }
    }

    fn w_mov_reg(&mut self, data: u32, extra: u32) {
        // A7.7.77
        let rd = data & 0xF;
        let setflags = bitset(data, 4);
        let rm = extra;

        let result = self.read_reg(rm);
        if rd == 15 {
            self.alu_write_pc(result);
        } else {
            self.write_reg(rd, result);
            if setflags {
                self.set_flags_nz(result);
            }
        }
    }

    fn w_movt(&mut self, data: u32, extra: u32) {
        // A7.7.79
        let rd = data;
        let imm16 = extra;

        let original = self.read_reg(rd);
        let modified = imm16 << 16 | (original & 0xFFFF);
        self.write_reg(rd, modified);
    }

    fn n_mul(&mut self, data: u32) {
        let rdm = data & 0x7;
        let rn = data >> 3;
        let result = self.read_reg(rdm).wrapping_mul(self.read_reg(rn));
        self.write_reg(rdm, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_mul(&mut self, data: u32, extra: u32) {
        // A7.7.84
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra;
        let op1 = self.read_reg(rn);
        let op2 = self.read_reg(rm);
        let result = op1.wrapping_mul(op2);
        self.write_reg(rd, result);
    }

    fn w_mvn_imm(&mut self, data: u32, extra: u32) {
        // A7.7.85
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);

        let result = !imm32;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn n_mvn_reg(&mut self, data: u32) {
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = !self.read_reg(rm);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_mvn_reg(&mut self, data: u32, extra: u32) {
        // A7.7.86
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = !shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    // A7.7.87 NEG: See RSB (imm)

    fn n_nop(&mut self, _data: u32) {
        // A7.7.88
        // do nothing
    }

    fn w_orn_imm(&mut self, data: u32, extra: u32) {
        // A7.7.89
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let result = self.read_reg(rn) | !imm32;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn w_orn_reg(&mut self, data: u32, extra: u32) {
        // A7.7.90
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 12);
        let shift_n = extra >> 3;
        let shift_t = extra & 0b111;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) | !shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_orr_imm(&mut self, data: u32, extra: u32) {
        // A7.7.91
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let result = self.read_reg(rn) | imm32;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nz_alt_c(result, data);
        }
    }

    fn n_orr_reg(&mut self, data: u32) {
        // A7.7.92
        let rdn = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rdn) | self.read_reg(rm);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nz(result);
        }
    }

    fn w_orr_reg(&mut self, data: u32, extra: u32) {
        // A7.7.92
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) | shifted;
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_pkhbt(&mut self, data: u32, extra: u32) {
        // A7.7.93
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let shift_n = extra >> 3;
        let shift_t = extra & 0b111;
        let tbform = bitset(extra, 1);

        let rn_val = self.read_reg(rn);
        let operand2 = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let result = if tbform {
            (rn_val & (0xFFFF << 16)) | (operand2 & 0xFFFF)
        } else {
            (operand2 & (0xFFFF << 16)) | (rn_val & 0xFFFF)
        };
        self.write_reg(rd, result);
    }

    fn n_pop(&mut self, data: u32) {
        // A7.7.99
        let mut address = self.read_sp();
        for i in 0..=7u32 {
            if bitset(data, i) {
                self.write_reg(i, self.read_mem_a(address, 4));
                address += 4;
            }
        }
        if bitset(data, 8) {
            self.load_write_pc(self.read_mem_a(address, 4));
            address += 4;
        }
        self.write_sp(address);
    }

    fn w_pop(&mut self, data: u32, extra: u32) {
        // A7.7.99
        let single_mode = bitset(data, 0);
        let mut address = self.read_sp();

        if single_mode {
            let rt = extra;
            let result = self.read_mem_u(address, 4);
            if rt == 15 {
                self.load_write_pc(result);
            } else {
                self.write_reg(rt, result);
            }
            address += 4;
        } else {
            let registers = extra & 0xFFFF;
            for i in 0..=14u32 {
                if bitset(registers, i) {
                    self.write_reg(i, self.read_mem_a(address, 4));
                    address += 4;
                }
            }
            if bitset(registers, 15) {
                self.load_write_pc(self.read_mem_u(address, 4));
                address += 4;
            }
        }
        self.write_sp(address);
    }

    fn n_push(&mut self, data: u32) {
        // A7.7.101
        let mut address = self.read_sp();
        if bitset(data, 8) {
            address -= 4;
            self.write_word(address, self.read_lr());
        }
        for i in (0..8u32).rev() {
            if bitset(data, i) {
                address -= 4;
                self.write_word(address, self.read_reg(i));
            }
        }
        self.write_sp(address);
    }

    fn w_push(&mut self, data: u32, extra: u32) {
        // A7.7.101
        let single_mode = bitset(data, 0);
        let mut address = self.read_sp();

        if single_mode {
            // TODO: When does "UnalignedAllowed = TRUE;" matter? SP is enforced word aligned by default
            let rt = extra;
            address -= 4;
            self.write_mem_u(address, 4, self.read_reg(rt));
        } else {
            let registers = extra & 0xFFFF;
            for i in (0..=14u32).rev() {
                if bitset(registers, i) {
                    address -= 4;
                    self.write_mem_u(address, 4, self.read_reg(i));
                }
            }
        }
        self.write_sp(address);
    }

    fn w_qadd(&mut self, data: u32, extra: u32) {
        // A7.7.102
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra;

        let rm_val = self.read_reg(rm) as i32;
        let rn_val = self.read_reg(rn) as i32;
        let (result, sat) = rm_val.overflowing_add(rn_val);
        if sat {
            self.write_reg(rd, rm_val.saturating_add(rn_val) as u32);
            self.cpu.set_saturation_flag(true);
        } else {
            self.write_reg(rd, result as u32);
        }
    }

    fn w_qsub(&mut self, data: u32, extra: u32) {
        // A7.7.109
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra;

        let rm_val = self.read_reg(rm) as i32;
        let rn_val = self.read_reg(rn) as i32;
        let (result, sat) = rm_val.overflowing_sub(rn_val);
        if sat {
            self.write_reg(rd, rm_val.saturating_sub(rn_val) as u32);
            self.cpu.set_saturation_flag(true);
        } else {
            self.write_reg(rd, result as u32);
        }
    }

    fn w_rbit(&mut self, data: u32, extra: u32) {
        // A7.7.112
        let rd = data;
        let rm = extra;

        self.write_reg(rd, self.read_reg(rm).reverse_bits());
    }

    fn n_rev(&mut self, data: u32) {
        // A7.7.113
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rm).swap_bytes();
        self.write_reg(rd, result);
    }

    fn w_rev(&mut self, data: u32, extra: u32) {
        // A7.7.113
        let rd = data;
        let rm = extra;

        let result = self.read_reg(rm).swap_bytes();
        self.write_reg(rd, result);
    }

    fn n_rev16(&mut self, data: u32) {
        // A7.7.114
        let rd = data & 0x7;
        let rm = data >> 3;

        let result = self.read_reg(rm).rotate_left(16).swap_bytes();
        self.write_reg(rd, result);
    }

    fn w_rev16(&mut self, data: u32, extra: u32) {
        // A7.7.114
        let rd = data;
        let rm = extra;

        let result = self.read_reg(rm).rotate_left(16).swap_bytes();
        self.write_reg(rd, result);
    }

    fn n_revsh(&mut self, data: u32) {
        // A7.7.115
        let rd = data & 0x7;
        let rm = data >> 3;

        let val = self.read_reg(rm);
        let result = shifted_sign_extend(val, 7, 8) + ((val >> 8) & 0xFF);
        self.write_reg(rd, result);
    }

    fn w_revsh(&mut self, data: u32, extra: u32) {
        // A7.7.115
        let rd = data;
        let rm = extra;

        let val = self.read_reg(rm);
        let result = shifted_sign_extend(val, 7, 8) + ((val >> 8) & 0xFF);
        self.write_reg(rd, result);
    }

    fn w_ror_imm(&mut self, data: u32, extra: u32) {
        // A7.7.116
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let shift_n = extra;

        let (result, carry) = self.ror_c(self.read_reg(rm), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_ror_reg(&mut self, data: u32) {
        // A7.7.117
        let rdn = data & 0x7;
        let rm = data >> 3;

        let shift = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.ror_c(self.read_reg(rdn), shift);
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_ror_reg(&mut self, data: u32, extra: u32) {
        // A7.7.117
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);
        let rm = extra;

        let shift_n = self.read_reg(rm) & 0xFF;
        let (result, carry) = self.ror_c(self.read_reg(rn), shift_n);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn w_rrx(&mut self, data: u32, _extra: u32) {
        // A7.7.118
        let rd = data & 0xF;
        let rm = (data >> 4) & 0xF;
        let setflags = bitset(data, 8);

        let (result, carry) = bits::rrx_c(self.read_reg(rm), self.cpu.read_carry_flag() as u32);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzc(result, carry);
        }
    }

    fn n_rsb_imm(&mut self, data: u32) {
        let rd = data & 0x7;
        let rn = data >> 3;
        let (result, carry, overflow) = add_with_carry(!self.read_reg(rn), 0, 1);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_rsb_imm(&mut self, data: u32, extra: u32) {
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let (result, carry, overflow) = add_with_carry(!self.read_reg(rn), imm32, 1);
        self.write_reg(rd, result);
        if bitset(data, 12) {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_rsb_reg(&mut self, data: u32, extra: u32) {
        // A7.7.120
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = add_with_carry(!self.read_reg(rn), shifted, 1);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_sbc_imm(&mut self, data: u32, extra: u32) {
        // A7.7.124
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let (result, carry, overflow) = self.add_with_carry_w_c(self.read_reg(rn), !imm32);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_sbc_reg(&mut self, data: u32) {
        // A7.7.125
        let rdn = data & 0b111;
        let rm = data >> 3;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rdn), !self.read_reg(rm), self.cpu.carry());
        self.write_reg(rdn, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_sbc_reg(&mut self, data: u32, extra: u32) {
        // A7.7.125
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);
        let shift_n = extra >> 3;
        let shift_t = extra & 0b111;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = self.add_with_carry_w_c(self.read_reg(rn), !shifted);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_sdiv(&mut self, data: u32, extra: u32) {
        // A7.7.127
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra;

        let rm_val = self.read_reg(rm) as i32;
        let result = if rm_val == 0 {
            if /*IntegerZeroDivideTrappingEnabled*/ false {
                // GenerateIntegerZeroDivide();
                0
            } else {
                0
            }
        } else {
            (self.read_reg(rn) as i32).wrapping_div(rm_val)
        } as u32;

        self.write_reg(rd, result);
    }

    fn w_smlal(&mut self, data: u32, extra: u32) {
        // A7.7.138
        let rn = data & 0xF;
        let rm = data >> 4;
        let rd_lo = extra & 0xF;
        let rd_hi = extra >> 4;

        let rd_lo_val = self.read_reg(rd_lo) as u64;
        let rd_hi_val = self.read_reg(rd_hi) as u64;
        let addend = rd_hi_val << 32 + rd_lo_val;
        let rn_val = i64::from(self.read_reg(rn) as i32);
        let rm_val = i64::from(self.read_reg(rm) as i32);
        let result = ((rn_val * rm_val) as u64).wrapping_add(addend);
        self.write_reg(rd_lo, (result & 0xFFFF_FFFF) as u32);
        self.write_reg(rd_hi, (result >> 32) as u32);
    }

    fn w_smull(&mut self, data: u32, extra: u32) {
        // A7.7.149
        let rn = data & 0xF;
        let rm = data >> 4;
        let rd_lo = extra & 0xF;
        let rd_hi = extra >> 4;

        let rn_val = i64::from(self.read_reg(rn) as i32);
        let rm_val = i64::from(self.read_reg(rm) as i32);
        let result = (rn_val * rm_val) as u64;
        self.write_reg(rd_lo, (result & 0xFFFF_FFFF) as u32);
        self.write_reg(rd_hi, (result >> 32) as u32);
    }

    fn n_stm(&mut self, data: u32) {
        // A7.7.159
        let rn = data >> 8;
        let registers = data;
        let mut address = self.read_reg(rn);
        for i in 0..=7u32 {
            if bitset(registers, i) {
                self.write_word(address, self.read_reg(i));
                address += 4;
            }
        }
        self.write_reg(rn, address);
    }

    fn w_stm(&mut self, data: u32, extra: u32) {
        // A7.7.159
        let rn = data;
        let registers = extra;
        let mut address = self.read_reg(rn);
        for i in 0..=14u32 {
            if bitset(registers, i) {
                self.write_word(address, self.read_reg(i));
                address += 4;
            }
        }
        if bitset(extra, 16) {
            self.write_reg(rn, address);
        }
    }

    fn w_stmdb(&mut self, data: u32, extra: u32) {
        // A7.7.160
        let rn = data;
        let registers = extra & 0xFFFF;
        let wback = bitset(extra, 16);

        let mut address = self.read_reg(rn) - 4 * registers.count_ones();
        for i in 0..=14u32 {
            if bitset(registers, i) {
                self.memory.write_mem_a(address, 4, self.read_reg(i)).unwrap_or_default();
                address += 4;
            }
        }
        if wback {
            self.write_reg(rn, self.read_reg(rn) - 4 * registers.count_ones());
        }
    }

    fn n_str_imm(&mut self, data: u32) {
        // A7.7.161
        let imm32 = (data & 0xFF) << 2;
        let rt = (data >> 8) & 0xF;
        let rn = data >> 12;
        let address = self.read_reg(rn).wrapping_add(imm32);
        self.write_word(address, self.read_reg(rt));
    }

    fn w_str_imm(&mut self, data: u32, extra: u32) {
        // A7.7.161
        let rt = data & 0xF;
        let rn = data >> 4;
        let rn_val = self.read_reg(rn);
        let offset_address = rn_val.wrapping_add(sign_extend(extra, 12));
        let index = bitset(extra, 14);
        let wback = bitset(extra, 13);
        let address = if index { offset_address } else { rn_val };
        self.write_word(address, self.read_reg(rt));
        if wback {
            self.write_reg(rn, offset_address);
        }
    }

    fn n_str_reg(&mut self, data: u32) {
        // A7.7.162
        let rt = data & 0b111;
        let rn = (data >> 3) & 0b111;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        self.write_mem_u(address, 4, self.read_reg(rt));
    }

    fn w_str_reg(&mut self, data: u32, extra: u32) {
        // A7.7.162
        let rt = data & 0xF;
        let rn = data >> 4;
        let rm = extra & 0xF;
        let imm2 = extra >> 4;

        let offset = self.read_reg(rm) << imm2;
        let address = self.read_reg(rn).wrapping_add(offset);
        self.write_mem_u(address, 4, self.read_reg(rt));
    }

    fn n_strb_imm(&mut self, data: u32) {
        // A7.7.163
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let imm5 = data >> 6;
        let address = self.read_reg(rn).wrapping_add(imm5);
        self.write_mem_u(address, 1, self.read_reg(rt));
    }

    fn n_strb_reg(&mut self, data: u32) {
        // A7.7.164
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        self.write_mem_u(address, 1, self.read_reg(rt));
    }

    fn w_strex(&mut self, data: u32, extra: u32) {
        // A7.7.167
        let rt = data & 0xF;
        let rd = data >> 4;
        let imm10 = extra & 0x3FF;
        let rn = extra >> 10;

        let address = self.read_reg(rn).wrapping_add(imm10);
        if self.exclusive_monitors_pass(address,4) {
            self.memory.write_mem_a(address, 4, self.read_reg(rt)).unwrap_or_default();
            self.write_reg(rd, 0);
        } else {
            self.write_reg(rd, 1);
        }
    }

    fn n_strh_imm(&mut self, data: u32) {
        // A7.7.170
        let imm6 = data & 0x3F;
        let rt = (data >> 3) & 0x7;
        let rn = data >> 6;
        let address = self.read_reg(rn).wrapping_add(imm6);
        self.write_mem_u(address, 2, self.read_reg(rt));
    }

    fn n_strh_reg(&mut self, data: u32) {
        // A7.7.171
        let rt = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let address = self.read_reg(rn).wrapping_add(self.read_reg(rm));
        self.write_mem_u(address, 2, self.read_reg(rt));
    }

    fn n_sub_imm(&mut self, data: u32) {
        let imm32 = data & 0xFF;
        let rd = (data >> 8) & 0x7;
        let rn = data >> 11;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !imm32, 1);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_sub_imm(&mut self, data: u32, extra: u32) {
        // A7.7.174
        let imm32 = data << 30 | extra;
        let rd = (data >> 4) & 0xF;
        let rn = (data >> 8) & 0xF;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !imm32, 1);
        self.write_reg(rd, result);
        if bitset(data, 12) {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_sub_reg(&mut self, data: u32) {
        // A7.7.175
        let rd = data & 0x7;
        let rn = (data >> 3) & 0x7;
        let rm = data >> 6;
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !self.read_reg(rm), 1);
        self.write_reg(rd, result);
        if !self.in_it_block() {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn w_sub_reg(&mut self, data: u32, extra: u32) {
        // A7.7.175
        let rd = data & 0xF;
        let rn = (data >> 4) & 0xF;
        let rm = (data >> 8) & 0xF;
        let setflags = bitset(data, 12);

        let shift_t = extra >> 6;
        let shift_n = extra & 0x3F;

        let shifted = self.get_shifted_register(self.read_reg(rm), shift_t, shift_n);
        let (result, carry, overflow) = add_with_carry(self.read_reg(rn), !shifted, 1);
        self.write_reg(rd, result);
        if setflags {
            self.set_flags_nzcv(result, carry, overflow);
        }
    }

    fn n_sub_sp_imm(&mut self, data: u32) {
        // A7.7.176
        let imm9 = data;
        let (result, _, _) = add_with_carry(self.read_sp(), !imm9, 1);
        self.write_sp(result);
    }

    fn n_svc(&mut self, _data: u32) {
        // A7.7.178
        // TODO: CallSupervisor()
    }

    fn n_sxtb(&mut self, data: u32) {
        // A7.7.182
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = sign_extend(self.read_reg(rm), 7);
        self.write_reg(rd, result);
    }

    fn n_sxth(&mut self, data: u32) {
        // A7.7.184
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = sign_extend(self.read_reg(rm), 15);
        self.write_reg(rd, result);
    }

    fn w_tbb(&mut self, data: u32, extra: u32) {
        // A7.7.185
        let rn = data & 0xF;
        let is_tbh = bitset(data, 4);
        let rm = extra;

        let rn_val = self.read_reg(rn);
        let rm_val = self.read_reg(rm);
        let halfwords = if is_tbh {
            self.read_mem_u(rn_val + rm_val << 1, 2)
        } else {
            self.read_mem_u(rn_val + rm_val, 1)
        };
        self.branch_write_pc(self.read_pc().wrapping_add(halfwords * 2));
    }

    fn w_teq_imm(&mut self, data: u32, extra: u32) {
        // A7.7.186
        let imm32 = data << 30 | extra;
        let rn = data >> 4;

        let result = self.read_reg(rn) ^ imm32;
        self.set_flags_nz_alt_c(result, data);
    }

    fn w_teq_reg(&mut self, data: u32, extra: u32) {
        // A7.7.187
        let rn = data & 0xF;
        let rm = data >> 4;
        let shift_t = extra & 0b111;
        let shift_n = extra >> 3;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) ^ shifted;
        self.set_flags_nzc(result, carry);
    }

    fn w_tst_imm(&mut self, data: u32, extra: u32) {
        // A7.7.188
        let imm32 = data << 30 | extra;
        let rn = data >> 4;

        let result = self.read_reg(rn) & imm32;
        self.set_flags_nz_alt_c(result, data);
    }

    fn n_tst_reg(&mut self, data: u32) {
        // A7.7.189
        let rn = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rn) & self.read_reg(rm);
        self.set_flags_nz(result);
    }

    fn w_tst_reg(&mut self, data: u32, extra: u32) {
        // A7.7.189
        let rn = data & 0xF;
        let rm = data >> 4;
        let shift_n = extra >> 3;
        let shift_t = extra & 0b111;

        let (shifted, carry) = self.get_shift_with_carry(self.read_reg(rm), shift_t, shift_n);
        let result = self.read_reg(rn) & shifted;
        self.set_flags_nzc(result, carry);
    }

    fn n_udf(&mut self, _data: u32) {
        // A7.7.194
        println!("Undefined exception");
        self.pending_default_handler.set(true);
    }

    fn w_udf(&mut self, _data: u32, _extra: u32) {
        // A7.7.194
        println!("Undefined exception");
        self.pending_default_handler.set(true);
    }

    fn w_udiv(&mut self, data: u32, extra: u32) {
        // A7.7.195
        let rd = data & 0xF;
        let rn = data >> 4;
        let rm = extra;
        let m = self.read_reg(rm);
        let result = if m == 0 {
            if /*IntegerZeroDivideTrappingEnabled*/ true {
                println!("GenerateIntegerZeroDivide");
                self.pending_default_handler.set(true);
                return;
            } else {
                0
            }
        } else {
            self.read_reg(rn) / m
        };
        self.write_reg(rd, result);
    }

    fn w_umaal(&mut self, data: u32, extra: u32) {
        // A7.7.202
        let rn = data & 0xF;
        let rm = data >> 4;
        let rd_lo = extra & 0xF;
        let rd_hi = extra >> 4;

        let rn_val = u64::from(self.read_reg(rn));
        let rm_val = u64::from(self.read_reg(rm));

        let result = rn_val * rm_val + u64::from(self.read_reg(rd_lo)) + u64::from(self.read_reg(rd_hi));
        let (upper, lower) = bits::split_u64(result);
        self.write_reg(rd_hi, upper);
        self.write_reg(rd_lo, lower);
    }

    fn w_umlal(&mut self, data: u32, extra: u32) {
        // A7.7.203
        let rn = data & 0xF;
        let rm = data >> 4;
        let rd_lo = extra & 0xF;
        let rd_hi = extra >> 4;

        let rd_lo_val = self.read_reg(rd_lo) as u64;
        let rd_hi_val = self.read_reg(rd_hi) as u64;
        let rn_val = self.read_reg(rn) as u64;
        let rm_val = self.read_reg(rm) as u64;
        let addend = rd_hi_val << 32 + rd_lo_val;
        let result = (rn_val * rm_val).wrapping_add(addend);
        let (upper, lower) = bits::split_u64(result);
        self.write_reg(rd_hi, upper);
        self.write_reg(rd_lo, lower);
    }

    fn w_umull(&mut self, data: u32, extra: u32) {
        // A7.7.204
        let rn = data & 0xF;
        let rm = data >> 4;
        let rd_lo = extra & 0xF;
        let rd_hi = extra >> 4;

        let rn_val = self.read_reg(rn) as u64;
        let rm_val = self.read_reg(rm) as u64;
        let result = rn_val * rm_val;
        let (upper, lower) = bits::split_u64(result);
        self.write_reg(rd_hi, upper);
        self.write_reg(rd_lo, lower);
    }

    fn n_uxtb(&mut self, data: u32) {
        // A7.7.221
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rm) & 0xFF;
        self.write_reg(rd, result);
    }

    fn n_uxth(&mut self, data: u32) {
        // A7.7.223
        let rd = data & 0x7;
        let rm = data >> 3;
        let result = self.read_reg(rm) & 0xFFFF;
        self.write_reg(rd, result);
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut registers = String::new();
        let indent = "    ";

        for i in 0..4 {
            let left = self.get_register_display_value(i);
            let right = self.get_register_display_value(i + 8);
            let left_label = format!("r{}", i);
            let right_label = format!("r{}", i + 8);
            registers.push_str(&format!(
                "{}{: >3}: {: <34}  {: >3}: {: <34}\n",
                indent, left_label, left, right_label, right
            ));
        }
        for i in 4..8 {
            let left = self.get_register_display_value(i);
            let right = self.get_register_display_value(i + 8);
            let special = ["r12", "sp", "lr", "pc"];
            let left_label = format!("r{}", i);

            registers.push_str(&format!(
                "{}{: >3}: {: <34}  {: >3}: {: <34}\n",
                indent,
                left_label,
                left,
                special[(i - 4) as usize],
                right
            ));
        }
        registers.push('\n');
        registers.push_str(&format!("{}{}\n{}{}\n", indent, self.cpu.get_apsr_display(), indent, self.cpu.itstate));
        return write!(f, "CPU {{\n{}}}", registers);
    }
}
