.syntax unified

.global main
.type main, %function
main:

ldr sp, =_stack_end
mov r0, 13
store_loop:
subs r0, 1
str r0, [sp, -4]!
bne store_loop
mov r5, 1

@ POP T1
ldr sp, =(_stack_end - 4 * 3)
pop.N {r1, r4, r7}

@ POP T2
ldr sp, =(_stack_end - 4 * 5)
pop.W {r10-r12, lr}

@ POP T3
ldr sp, =(_stack_end - 4 * 4)
pop.W {r5}

.size main, . - main

.section .isr_vector, "a", %progbits
g_pfnVectors:
.word _stack_end
.word main
