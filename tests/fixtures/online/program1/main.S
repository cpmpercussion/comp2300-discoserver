.syntax unified

.global main
.type main, %function
main:
  mov r0, 0
  loop:
  add r0, 1
  b loop
.size main, . - main


.section .isr_vector, "a", %progbits
.type	g_pfnVectors, %object

g_pfnVectors:
.word _stack_end
.word main

.size	g_pfnVectors, . - g_pfnVectors
