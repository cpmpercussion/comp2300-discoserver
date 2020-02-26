A few programs that should be tested to ensure correct PC treatment.


Note: Everything tested so far is consistent with
- Isolated "true" instruction pointer
- Address of current instruction + 4 available in "normal" register bank
- Branch instructions affect the "true" instruction pointer
- Non specific branch instructions affect "normal" PC

### PC value
- ADC (imm) with rn=PC is Unpredictable, but will use PC+4 (no word align)
- ADC (reg) is same as ADC (imm); PC is not word aligned
- ADD (imm) is PC + 4, word aligned
- ADD (imm) writing to PC does nothing
- ADD (sp + imm) writes same as ADD (imm)
- LSL (imm) into PC does nothing

```
mov r0, PC @ 2byte
sub r0, 6  @ 4byte
mov PC, r0 @ 2byte
```


### SP value
- POP with SP is unpredictable, and does not get recognised (instruction executes as if SP bit is false)


### Compile steps:

- arm-none-eabi-as -mthumb -mcpu=cortex-m4 -o main.o main.s
- arm-none-eabi-ld -T ../../common/linker.ld -nostartfiles -o firmware.elf main.o


- GDB register names
```
["r0","r1","r2","r3","r4","r5","r6","r7","r8","r9","r10","r11","r12","sp","lr","pc","","","","","","","","","","xPSR","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","d0","d1","d2","d3","d4","d5","d6","d7","d8","d9","d10","d11","d12","d13","d14","d15","","","","","","","","","","","","","","","","","fpscr","msp","psp","primask","basepri","faultmask","control","s0","s1","s2","s3","s4","s5","s6","s7","s8","s9","s10","s11","s12","s13","s14","s15","s16","s17","s18","s19","s20","s21","s22","s23","s24","s25","s26","s27","s28","s29","s30","s31"]
```
