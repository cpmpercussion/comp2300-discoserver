.syntax unified

/**
 * NOTE: This is a "core" instruction that is relied upon to work in other tests.
 *       Do not use other instructions in this test.
 *
 * TODO: Separate "core" tests from general.
 */

.global main
.type main, %function
main:
  @ MOV (imm) T1
  movs.N r0, 0
  movs.N r0, 5

  @ MOV (imm) T2
  movs.W r9, 0b11001010 << 24
  mov.W r9, 0b11001010 << 23

  @ MOV (imm) T3
  mov.W r10, 65535

  @ MOV (imm) T3 (regression)
  mov.W r0, 0x1234

  @ MOV (reg) T1
  mov r8, 0xFF
  mov.N r9, r8

  @ MOV (reg) T2
  mov r0, 0
  movs.N r1, r0

  @ MOV (reg) T3
  mov.W r9, sp

  @ MOV (reg) T1 w. PC
  mov r0, 123
  mov.N pc, r0
.size main, . - main
