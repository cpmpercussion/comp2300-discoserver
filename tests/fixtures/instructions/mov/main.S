.syntax unified

.global main
.type main, %function
main:
  @ MOV (imm) T1
  movs.N r0, 0
  movs.N r0, 5

  @ MOV (imm) T2
  movs.W r1, 0b11001010 << 24
  mov.W r1, 0b11001010 << 23

  @ MOV (imm) T3
  mov.W r2, 65535
.size main, . - main
