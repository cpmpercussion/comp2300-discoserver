.syntax unified

.global main
.type main, %function
main:
  mov r0, 0
  loop:
  add r0, 1
  b loop
.size main, . - main
