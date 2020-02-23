.syntax unified

@ NOTE: A bad implementation would stuff up the PC value used when branching. Not sure
@       if it's tested correctly here, but regression tests can be added if any issues are found.
@
@ NOTE: This one is sensitive to changes because it heavily uses relative PC addresses.
@       Make sure to update the corresponding test parts if adding more behaviour.

branch_wide:
  @ Branch T4
  b.W branch_wide_resume
  mov r0, 1
  b branch_wide

.global main
.type main, %function
main:
  @ Branch T1
  movs.N r0, 0 @ non-zero indicates failed branch
  bne.N branch_narrow
  beq.N branch_narrow
  mov r0, 1
  branch_narrow_resume:
  nop

  @ Branch T3
  blt.W branch_wide
  bge.W branch_wide

  branch_wide_resume:
  nop
.size main, . - main

branch_narrow:
  @ Branch T2 (negative jump)
  b.N branch_narrow_resume
  mov r0, 1
