.syntax unified

.global main
.type main, %function
main:
  @ STR (imm) T4
  ldr r0, =_data_start_ram
  ldr r1, =_stack_end
  ldr r2, =0x32A7F092
  mov r5, 0

  main_loop:
  str r2, [r0], 4
  cmp r0, r1
  bne main_loop
  mov r5, 1
  mov r5, 0

  @ STR (imm) T1
  ldr r0, =_data_start_ram
  ldr r1, =0xDEADBEEF
  

.size main, . - main
