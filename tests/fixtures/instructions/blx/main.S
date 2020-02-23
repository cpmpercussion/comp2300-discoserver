.syntax unified

.type target2, %function
target2:
  mov r0, 1
  nop
.size target2, . - target2

.global main
.type main, %function
main:
  ldr r0, =target
  blx r0
.size main, . - main

.word 0xDEADBEEF
.word 0xDEADBEEF
.word 0xDEADBEEF

.type target, %function
target:
  mov r0, 0
  ldr r1, =target2
  blx r1
.size target, . - target
