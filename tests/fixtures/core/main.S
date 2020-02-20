/**
 * Instructions that are expected to work in order to set up more complicated
 * tests for other instructions.
 */
.syntax unified

.global main
.type main, %function
main:
  mov r0, 0x0

  bx lr
.size main, . - main
