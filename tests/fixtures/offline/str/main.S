.syntax unified

.global main
.type main, %function
main:
  @ STR (imm) T4
  ldr r0, =_data_start_ram
  ldr r1, =_stack_end
  ldr r2, =0x32A7F092
  mov r5, 0

  main_loop:
  str r2, [r0], 4
  cmp r0, r1
  bne main_loop
  mov r5, 1
  mov r5, 0

  @ STR (imm) T1
  ldr r0, =_data_start_ram
  ldr r1, =0xDEADBEE1
  str.N r1, [r0, 124]

  @ STR (imm) T2
  ldr sp, =_data_start_ram
  ldr r1, =0xDEADBEE2
  str.N r1, [sp, 1020]

  @ STR (imm) T3
  ldr r10, =_data_start_ram
  ldr r1, =0xDEADBEE3
  str.W r1, [r10, 4095]

  @ STR (imm) T4
  ldr r10, =(_data_start_ram + 256)
  ldr r1, =0xDEADBEE4
  str.W r1, [r10, -255]!

  @ STR (reg) T1
  ldr r0, =_data_start_ram
  mov r1, 12
  ldr r2, =0xDEADBEE5
  str.N r2, [r0, r1]

  @ STR (reg) T2
  ldr r0, =_data_start_ram
  mov r1, 12
  ldr r2, =0xDEADBEE6
  str.W r2, [r0, r1, LSL 3]
.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
