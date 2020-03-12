.syntax unified

.global main
.type main, %function
main:
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

.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
