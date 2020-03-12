.syntax unified

.type target2, %function
target2:
  mov r0, 1
  nop
.size target2, . - target2

.global main
.type main, %function
main:
  mov r0, 1
  bl target
.size main, . - main

.word 0xDEADBEEF
.word 0xDEADBEEF
.word 0xDEADBEEF

.type target, %function
target:
  mov r0, 0
  bl target2
.size target, . - target

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
