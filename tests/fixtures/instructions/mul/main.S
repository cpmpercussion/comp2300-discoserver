.syntax unified

.global main
.type main, %function
main:
  @ MUL T1
  mov r7, 0xFFFFFFFF
  mov r6, 0xFFFFFFFF
  muls.N r7, r6

  @ MUL T2
  mov r12, 0xFFFFFFFF
  mov r11, 0xFFFFFFFF
  mul.W r10, r12, r11

.size main, . - main
