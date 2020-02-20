.syntax unified

.global main
.type main, %function
main:
  mov r0, 0

  @ ADD (imm) T1
  adds.N r1, r0, 7

  @ ADD (imm) T2
  mov r7, 0xFFFFFFFF
  adds.N r7, r7, 0xFF

  @ ADD (imm) T3
  mov r10, 0
  add.W r11, r10, 0xFF << 24
  adds.W r7, r10, 0xFF << 24

  @ ADD (imm) T4
  add.W r11, r0, 0xFFF
.size main, . - main
