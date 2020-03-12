.syntax unified

.type target2, %function
target2:
  nop
  bx lr
.size target2, . - target2

.word 0xDEADBEEF
.word 0xDEADBEEF
.word 0xDEADBEEF

.global main
.type main, %function
main:
  ldr r0, =target
  bx r0
.size main, . - main

.word 0xDEADBEEF
.word 0xDEADBEEF
.word 0xDEADBEEF

.type target, %function
target:
  ldr lr, =target2
  bx lr
.size target, . - target

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
