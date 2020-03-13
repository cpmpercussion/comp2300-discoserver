.syntax unified

@ TODO: Automatically scrape from offline tests

.global main
.type main, %function
main:
  @ ADC (imm) T1
  mov r0, 0
  adds r0, r0 @ clear carry flag

  mov r1, 0xFF << 24
  adc.W r10, r1, 0xAA << 24
  movs r1, 0xFF << 24 @ set carry flag
  adc.W r10, r1, 0xAA << 24

  @ ADC (reg) T1
  movs r0, 0xFF << 24 @ set carry flag
  mov r0, 0xFF << 23
  mov r1, 0xFF << 23
  adcs.N r1, r0

  @ ADC (reg) T2
  movs r8, 0xFF << 24 @ set carry flag
  mov r9, 1
  adc.W r10, r9, r8, ASR 32

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

  mov r0, 0
  mov r1, 0xFFFFFFFF
  mov r2, 0xF0F0F0F0

  @ AND (imm) T1
  and.W r9, r2, 0x0F0F0F0F
  and.W r10, r2, 0xFF00FF00

  @ AND (reg) T1
  ands.N r1, r2

  @ AND (reg) T2
  mov r5, 1
  and.W r5, r2, r0

  mov r0, 0xFFFFFFFF
  mov r1, 0xFFFFFFFF
  mov r6, 1
  mov r7, 1
  mov r8, 1
  mov r9, 1
  and.W r6, r0, r1, LSL 4
  and.W r7, r0, r1, LSR 16
  and.W r8, r0, r1, LSR 32
  and.W r9, r0, r1, ASR 32

  mov r1, 0xF0F0F0F0
  mov r5, 1
  mov r6, 1
  mov r7, 1
  mov r3, 0x80000000
  adds r3, r3 @ set carry

  and.W r5, r0, r1, ROR 4
  and.W r6, r0, r1, RRX

  mov r3, 0
  adds r3, r3 @ clear carry
  and.W r7, r0, r1, RRX

  @ EXCLUSIVE
  ldr r0, =_data_start_ram
  ldr r1, =0xDEADBEE1
  str r1, [r0]
  ldr r0, =(_data_start_ram + 4)
  ldr r1, =0xDEADBEE3
  str r1, [r0]

  ldrex r1, [r0]
  mov r1, 0xFF
  strex r1, r1, [r0]
  strex r1, r1, [r0]

  ldr r0, =_data_start_ram
  ldr r1, =(_data_start_ram + 4)

  ldrex r2, [r0]
  ldrex r4, [r1]
  strex r4, r2, [r0]
  strex r4, r2, [r0]

  ldrex r2, [r0]
  ldr r2, =0xDEADBEE2
  clrex
  strex r3, r2, [r0]

  @ LSL
  @ LSL (imm) T1
  movs r0, 0
  adds r0, 0 @ clear carry
  ldr r0, =0xFFFFFFFF
  lsls.N r1, r0, 0
  lsls.N r1, r0, 31

  ldr r0, =0x0000FFFF
  lsls.N r1, r0, 16

  @ LSL (imm) T2
  movs r0, 0
  adds r0, 0 @ clear carry

  ldr r0, =0x7FFFFFFF
  lsls.W r1, r0, 1
  lsls.W r1, r0, 2

  @ LSL (reg) T1
  ldr r0, =0xFFFFFFFF
  ldr r1, =0xFFFFFFFF
  lsls.N r0, r1

  ldr r0, =0xFFFFFFFF
  ldr r1, =32
  lsls.N r0, r1

  ldr r0, =0xAABECCDD
  ldr r1, =16
  lsls.N r0, r1

  @ LSL (reg) T2
  ldr r10, =0xFFFFFFFF
  ldr r11, =0xFFFFFFFF
  lsls.W r10, r11

  ldr r10, =0xFFFFFFFF
  ldr r11, =32
  lsls.W r10, r11

  ldr r10, =0xAABECCDD
  ldr r11, =16
  lsls.W r10, r11

  @ MOV
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

  @ MUL
  @ MUL T1
  mov r7, 0xFFFFFFFF
  mov r6, 0xFFFFFFFF
  muls.N r7, r6

  @ MUL T2
  mov r12, 0xFFFFFFFF
  mov r11, 0xFFFFFFFF
  mul.W r10, r12, r11

  @ POP
  ldr sp, =_stack_end
  mov r0, 13
  store_loop:
  subs r0, 1
  str r0, [sp, -4]!
  bne store_loop
  mov r5, 1

  @ POP T1
  ldr sp, =(_stack_end - 4 * 3)
  pop.N {r1, r4, r7}

  @ POP T2
  ldr sp, =(_stack_end - 4 * 5)
  pop.W {r10-r12, lr}

  @ POP T3
  ldr sp, =(_stack_end - 4 * 4)
  pop.W {r5}

  @ PUSH
  mov r0, 0
  mov r1, 1
  mov r2, 2
  mov r3, 3
  mov r4, 4
  mov r5, 5
  mov r6, 6
  mov r7, 7
  mov r8, 8
  mov r9, 9
  mov r10, 10
  mov r11, 11
  mov r12, 12
  mov lr, 14

  @ PUSH T1
  ldr sp, =_stack_end
  push.N {r0, r3, r7, lr}

  @ PUSH T2
  ldr sp, =_stack_end
  push.W {r2-r8, r10-r12, lr}

  @ PUSH T3
  ldr sp, =(_stack_end - 1)
  push.W {lr}

  @ SUB
  @ SUB (imm) T1
  mov r7, 5
  subs.N r6, r7, 7

  @ SUB (imm) T2
  ldr r7, =-2
  subs.N r7, 0xFF

  @ SUB (imm) T3
  mov r12, 12
  sub.W r10, r12, 0xFFFFFFFF

  @ SUB (imm) T4
  mov r8, 123
  sub.W r11, r8, 0xFFF

  @ SUB (reg) T1
  mov r6, 2
  mov r3, 7
  subs.N r1, r6, r3

  @ SUB (reg) T2
  mov r10, 8
  mov r11, 16
  sub.W r12, r11, r10

  @ SUB (SP minus imm) T1
  ldr sp, =0xFFFF
  sub.N sp, sp, 508

  @ SUB (SP minus imm) T2
  ldr sp, =0xFFFF
  sub.W sp, sp, (0xFF << 16 | 0xFF)

  @ SUB (SP minus imm) T3
  ldr sp, =0xFFFF
  sub.W sp, sp, 0xFFF


  ldr lr, =0x444F4E45
.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
