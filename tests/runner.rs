#![allow(dead_code)]

mod common;
use common::{load_program, load_and_step};

#[test]
fn adc() {
    let mut board = load_and_step("adc", 2).unwrap();
    assert_eq!(board.cpu.get_flags().c, false);
    assert_eq!(board.cpu.get_flags().z, true);

    // ADC (imm) T1
    board.step().unwrap();
    assert_eq!(board.cpu.get_flags().c, false);
    board.step().unwrap();
    assert_eq!(board.read_reg(10u32), 0xA9 << 24);

    board.step().unwrap();
    assert_eq!(board.cpu.get_flags().c, true);
    board.step().unwrap();
    assert_eq!(board.read_reg(10u32), (0xA9 << 24) + 1);

    // ADC (reg) T1
    board.step().unwrap();
    assert_eq!(board.cpu.get_flags().c, true);
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), (0xFF << 24) + 1);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, false);
    assert_eq!(flags.v, true);

    // ADC (reg) T2
    board.step().unwrap();
    assert_eq!(board.cpu.get_flags().c, true);
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(10u32), 1);
}

#[test]
fn add() {
    let mut board = load_and_step("add", 1).unwrap();
    assert_eq!(board.read_reg(0u32), 0);

    // ADD (imm) T1
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 7);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, false);
    assert_eq!(flags.v, false);

    // ADD (imm) T2
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(7u32), 0xFE);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);
    assert_eq!(flags.v, false);

    // ADD (imm) T3
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(11u32), 0xFF << 24);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);
    assert_eq!(flags.v, false);

    board.step().unwrap();
    assert_eq!(board.read_reg(7u32), 0xFF << 24);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, false);
    assert_eq!(flags.v, false);

    // ADD (imm) T4
    board.step().unwrap();
    assert_eq!(board.read_reg(11u32), 0xFFF);

    // ADD (reg) T1
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(7u32), 0xED << 24);

    // ADD (reg) T2
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(11u32), 23);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, true);

    // ADD (reg) T3
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(9u32), u32::max_value());
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, false);

    board.step().unwrap();
    assert_eq!(board.read_reg(8u32), 0);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, false);

    // ADD (reg) T2 w. PC
    assert_ne!(board.read_reg(4u32), 76);
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(4u32), 76);

    board.step().unwrap();
    assert_eq!(board.read_reg(4u32), 98);
}

#[test]
fn mov() {
    let mut board = load_program("mov").unwrap();
    let orig_flags = board.cpu.get_flags();

    // MOV (imm) T1
    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 0);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, true);
    assert_eq!(flags.c, orig_flags.c);

    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 5);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, orig_flags.c);

    // MOV (imm) T2
    board.step().unwrap();
    assert_eq!(board.read_reg(9u32), 0b11001010 << 24);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);

    board.step().unwrap();
    assert_eq!(board.read_reg(9u32), 0b11001010 << 23);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);

    // MOV (imm) T3
    board.step().unwrap();
    assert_eq!(board.read_reg(10u32), 65535);

    // MOV (reg) T1
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(8u32), 0xFF);
    assert_eq!(board.read_reg(9u32), 0xFF);

    // MOV (reg) T2
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 0);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, true);

    // MOV (reg) T3
    board.step().unwrap();
    assert_eq!(board.read_reg(9u32), board.read_sp());

    // MOV (reg) T1 w. PC
    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 123);

    board.step().unwrap();
    assert_eq!(board.cpu.read_instruction_pc(), 122); // Branching 0's the least significant bit
}

#[test]
fn str() {
    let mut board = load_program("str").unwrap();

    // STR (imm) T4
    let mut i = 0;
    while board.read_reg(5u32) != 1 {
        i += 1;
        if i > 100_000 {
            panic!("Expected iterations to finish");
        };
        board.step().unwrap();
    }
    board.step().unwrap();

    let test_val = 0x32A7F092;
    for i in (0x2000_0000..0x2001_8000).step_by(4) {
        match board.memory.read_mem_u(i, 4) {
            Ok(v) => {
                if v != test_val {
                    println!("incorrect memory value: expected {}, got {}", test_val, v);
                    assert!(false);
                }
            },
            Err(e) => {
                println!("Failed to read word at {}: {}", i, e);
                assert!(false);
            }
        };
    }

    // STR (imm) T1
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0000 + 124, 4) {
        assert_eq!(v, 0xDEADBEE1);
    } else {
        assert!(false);
    }

    // STR (imm) T2
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0000 + 1020, 4) {
        assert_eq!(v, 0xDEADBEE2);
    } else {
        assert!(false);
    }

    // STR (imm) T3
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0000 + 4095, 4) {
        assert_eq!(v, 0xDEADBEE3);
    } else {
        assert!(false);
    }

    // STR (imm) T4
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0001, 4) {
        assert_eq!(v, 0xDEADBEE4);
    } else {
        assert!(false);
    }
    assert_eq!(board.read_reg(10u32), 0x2000_0001);

    // STR (reg) T1
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0000 + 12, 4) {
        assert_eq!(v, 0xDEADBEE5);
    } else {
        assert!(false);
    }

    // STR (reg) T2
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    if let Ok(v) = board.memory.read_mem_u(0x2000_0000 + (12 << 3), 4) {
        assert_eq!(v, 0xDEADBEE6);
    } else {
        assert!(false);
    }
}

#[test]
fn ldr() {
    let mut board = load_program("ldr").unwrap();

    let mut i = 0;
    while board.read_reg(5u32) != 1 {
        i += 1;
        if i > 100_000 {
            panic!("Expected iterations to finish");
        };
        board.step().unwrap();
    }
    board.step().unwrap();

    for i in (0x2000_0000..0x2001_8000).step_by(4) {
        match board.memory.read_mem_u(i, 4) {
            Ok(v) => {
                if v != i - 0x2000_0000 {
                    println!("incorrect memory value: expected {}, got {}", i - 0x2000_0000, v);
                    assert!(false);
                }
            },
            Err(e) => {
                println!("Failed to read word at {}: {}", i, e);
                assert!(false);
            }
        };
    }

    // LDR (imm) T1
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 124);

    // LDR (imm) T2
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 1020);

    // LDR (imm) T3
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 4092);
    assert_eq!(board.read_reg(2u32), 0x0010_0000); // verified against real board w/ offset 4095

    // LDR (imm) T4
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 0);
    assert_eq!(board.read_reg(0u32), 0x2000_0000);

    // LDR (reg) T1
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(2u32), 0x1777C);

    // LDR (reg) T2
    board.step().unwrap();
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(2u32), 16);

    // LDR (lit) T1
    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 0xDEADBEE1);

    // LDR (lit) T2
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 0xDEADBEE1);
    assert_eq!(board.read_reg(1u32), 0xDEADBEE2);
}

#[test]
fn push() {
    let mut board = load_and_step("push", 14).unwrap();

    // PUSH T1
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 4, 4).unwrap(), 14);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 8, 4).unwrap(), 7);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 12, 4).unwrap(), 3);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 16, 4).unwrap(), 0);

    // PUSH T2
    board.step().unwrap();
    board.step().unwrap();
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 4, 4).unwrap(), 14);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 8, 4).unwrap(), 12);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 12, 4).unwrap(), 11);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 16, 4).unwrap(), 10);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 20, 4).unwrap(), 8);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 24, 4).unwrap(), 7);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 28, 4).unwrap(), 6);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 32, 4).unwrap(), 5);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 36, 4).unwrap(), 4);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 40, 4).unwrap(), 3);
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 44, 4).unwrap(), 2);

    // PUSH T3
    board.step().unwrap();
    assert_eq!(board.read_sp(), 0x2001_7FFC); // The stack pointer lower bits are cleared
    board.step().unwrap();
    assert_eq!(board.memory.read_mem_u(0x2001_8000 - 8, 4).unwrap(), 14);
}
