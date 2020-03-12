.syntax unified

.global main
.type main, %function
main:
  ldr r0, =_data_start_ram
  ldr r1, =0xDEADBEE1
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

.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
