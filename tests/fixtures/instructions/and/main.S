.syntax unified

.global main
.type main, %function
main:
  mov r0, 0
  mov r1, 0xFFFFFFFF
  mov r2, 0xF0F0F0F0

  @ AND (imm) T1
  and.W r9, r2, 0x0F0F0F0F
  and.W r10, r2, 0xFF00FF00

  @ AND (reg) T1
  ands.N r1, r2

  @ AND (reg) T2
  mov r5, 1
  and.W r5, r2, r0

  mov r0, 0xFFFFFFFF
  mov r1, 0xFFFFFFFF
  mov r6, 1
  mov r7, 1
  mov r8, 1
  mov r9, 1
  and.W r6, r0, r1, LSL 4
  and.W r7, r0, r1, LSR 16
  and.W r8, r0, r1, LSR 32
  and.W r9, r0, r1, ASR 32

  mov r1, 0xF0F0F0F0
  mov r5, 1
  mov r6, 1
  mov r7, 1
  mov r3, 0x80000000
  adds r3, r3 @ set carry

  and.W r5, r0, r1, ROR 4
  and.W r6, r0, r1, RRX

  mov r3, 0
  adds r3, r3 @ clear carry
  and.W r7, r0, r1, RRX


.size main, . - main
