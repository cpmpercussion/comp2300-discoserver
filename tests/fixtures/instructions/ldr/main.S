.syntax unified

.align 4
lit_back:
.word 0xDEADBEE2

.global main
.type main, %function
main:
  @ Setup (success is verified in test)
  ldr r0, =_data_start_ram
  ldr r1, =_stack_end
  mov r2, 0
  mov r5, 0

  main_loop:
  str r2, [r0], 4
  add r2, 4
  cmp r0, r1
  bne main_loop
  mov r5, 1
  mov r5, 0

  @ LDR (imm) T1
  ldr r0, =_data_start_ram
  ldr.N r1, [r0, 124]

  @ LDR (imm) T2
  ldr sp, =_data_start_ram
  ldr.N r1, [sp, 1020]

  @ LDR (imm) T3
  ldr r0, =_data_start_ram
  ldr.W r1, [r0, 4092]
  ldr.W r2, [r0, 4095]

  @ LDR (imm) T4
  ldr r0, =(_data_start_ram + 255)
  ldr.W r1, [r0, -255]!

  @ LDR (reg) T1
  ldr r0, =_data_start_ram
  ldr r1, =0x1777C
  ldr.N r2, [r0, r1]

  @ LDR (reg) T2
  ldr r0, =_data_start_ram
  mov r1, 2
  ldr.W r2, [r0, r1, LSL 3]

  @ LDR (lit) T1
  ldr.N r0, lit_forward

  @ LDR (lit) T2
  ldr.W r0, lit_forward
  ldr.W r1, lit_back

  bx lr
.size main, . - main

.align 4
lit_forward:
.word 0xDEADBEE1
