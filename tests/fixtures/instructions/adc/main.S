.syntax unified

.global main
.type main, %function
main:
  mov r0, 0
  adds r0, r0 @ clear carry flag

  @ ADC (imm) T1
  mov r1, 0xFF << 24
  adc.W r10, r1, 0xAA << 24
  movs r1, 0xFF << 24 @ set carry flag
  adc.W r10, r1, 0xAA << 24

  @ ADC (reg) T1
  movs r0, 0xFF << 24 @ set carry flag
  mov r0, 0xFF << 23
  mov r1, 0xFF << 23
  adcs.N r1, r0

  @ ADC (reg) T2
  movs r8, 0xFF << 24 @ set carry flag
  mov r9, 1
  adc.W r10, r9, r8, ASR 32

.size main, . - main
