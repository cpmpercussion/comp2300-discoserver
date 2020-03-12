.syntax unified

.global main
.type main, %function
main:
  mov r0, 0

  @ ADD (imm) T1
  adds.N r1, r0, 7

  @ ADD (imm) T2
  mov r7, 0xFFFFFFFF
  adds.N r7, r7, 0xFF

  @ ADD (imm) T3
  mov r10, 0
  add.W r11, r10, 0xFF << 24
  adds.W r7, r10, 0xFF << 24

  @ ADD (imm) T4
  add.W r11, r0, 0xFFF

  @ ADD (reg) T1
  mov r6, 0xFF << 24
  mov r5, 0xEE << 24
  adds.N r7, r6, r5

  @ ADD (reg) T2
  movs r8, 0
  mov r11, 23
  add.N r11, r8

  @ ADD (reg) T3
  movs r0, 0
  mov r1, -1
  adds.W r9, r1, r0
  add.W r8, r0, r0

  @ ADD (reg) T2 w. PC
  mov r10, 4
  add.N pc, r10
  movs.N r4, 13
  movs.N r4, 26
  movs.N r4, 54
  movs.N r4, 76 @ Jumps here
  movs.N r4, 98

.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
