.syntax unified

.global main
.type main, %function
main:
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

.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
