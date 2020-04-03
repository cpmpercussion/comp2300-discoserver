# Disco Emulator

### General
Peripherals and interrupts are not supported. Audio specifically is supported, by detecting the call to `BSP_AUDIO_OUT_Play_Sample`.


### Arguments
See the generated `platformio.ini` file for where to add arguments (in the `debug_server` property).

- To check the emulator version, pass `--version`
- To specify the ELF file, pass `-kernel <path>`
- To print a list of sound samples, pass `--samples <start> <end>`
- If you want sound to work, pass `--audio`. Verify by checking the `DEBUG CONSOLE` output for audio related messages. A working audio connection looks something like this
    ```
    Candidate audio format: channels: 1, min: SampleRate(1), max: SampleRate(192000), data: I16
    Spawned audio at freq 48000
    Audio output connected
    ```

    CAUTION: Turn down your volume initially, and raise it to the desired level. The output can be very loud.
- To buffer the audio output, pass `--buffer` and then the number of seconds to buffer for on the next line. E.g.,
    ```
    --buffer
    5
    ```
    for a 5 second buffer (it will gather 5 seconds worth of samples and then play them, then gather the next 5 seconds and play, and so on).

    Try without a buffer first, and only use it if the sound is glitchy (actions such as moving your mouse should make the sound even worse if this is the case).

    Check the debug console for when the buffer is being filled and emptied. You may see `underrun` errors; these can be ignored. The debugger cannot be paused while the buffer is being emptied (i.e., sound is being played).
- A private Piazza post is the preferred way to report any issues. If something is going wrong, please use the `--debug` flag and include the output in `DEBUG CONSOLE` with the report.


### Supported Instructions

The following table shows the expected support of the current emulator. If a box is ticked, then all encodings of that instruction are expected to work as intended. If crossed, then either all or some of the encodings will not execute properly.

| Instruction | Supported | Comments |
|-------------------|-----------|-------------|
| ADC (imm) | ✅ |  |
| ADC (reg) | ✅ |  |
| ADD (imm) | ✅ |  |
| ADD (reg) | ✅ |  |
| ADD (SP plus imm) | ❌ |  |
| ADD (SP plus reg) | ❌ |  |
| ADR | ✅ |  |
| AND (imm) | ✅ |  |
| AND (reg) | ✅ |  |
| ASR (imm) | ✅ |  |
| ASR (reg) | ✅ |  |
| B | ✅ |  |
| BFC | ✅ |  |
| BFI | ✅ |  |
| BIC (imm) | ✅ |  |
| BIC (reg) | ✅ |  |
| BKPT | ✅ | Currently does nothing |
| BL | ✅ |  |
| BLX (reg) | ✅ |  |
| BX | ✅ |  |
| CBNZ, CBZ | ✅ |  |
| CDP, CDP2 | ❌ |  |
| CLREX | ✅ |  |
| CLZ | ✅ |  |
| CMN (imm) | ✅ |  |
| CMN (reg) | ✅ |  |
| CMP (imm) | ✅ |  |
| CMP (reg) | ✅ |  |
| CPS | ❌ |  |
| CPY | ✅ | See MOV (reg) |
| CSDB | ❌ |  |
| DBG | ❌ |  |
| DMB | ❌ |  |
| DSB | ❌ |  |
| EOR (imm) | ✅ |  |
| EOR (reg) | ✅ |  |
| ISB | ❌ |  |
| IT | ✅ |  |
| LDC, LDC2 (imm) | ❌ |  |
| LDC, LDC2 (lit) | ❌ |  |
| LDM, LDMIA, LDMFD | ✅ |  |
| LDMDB, LDMEA | ✅ |  |
| LDR (imm) | ✅ |  |
| LDR (lit) | ✅ |  |
| LDR (reg) | ✅ |  |
| LDRB (imm) | ❌ |  |
| LDRB (lit) | ❌ |  |
| LDRB (reg) | ❌ |  |
| LDRBT | ❌ |  |
| LDRD (imm) | ❌ |  |
| LDRD (lit) | ❌ |  |
| LDREX | ✅ |  |
| LDREXB | ❌ |  |
| LDREXH | ❌ |  |
| LDRH (imm) | ❌ |  |
| LDRH (lit) | ❌ |  |
| LDRH (reg) | ❌ |  |
| LDRHT | ❌ |  |
| LDRSB (imm) | ❌ |  |
| LDRSB (lit) | ❌ |  |
| LDRSB (reg) | ❌ |  |
| LDRSBT | ❌ |  |
| LDRSH (imm) | ❌ |  |
| LDRSH (lit) | ❌ |  |
| LDRSH (reg) | ❌ |  |
| LDRSHT | ❌ |  |
| LDRT | ❌ |  |
| LSL (imm) | ✅ |  |
| LSL (reg) | ✅ |  |
| LSR (imm) | ✅ |  |
| LSR (reg) | ✅ |  |
| MCR, MCR2 | ❌ |  |
| MCRR, MCRR2 | ❌ |  |
| MLA | ✅ |  |
| MLS | ✅ |  |
| MOV (imm) | ✅ |  |
| MOV (reg) | ✅ |  |
| MOV (shifted reg) | ✅ | See LSL, LSR, etc. |
| MOVT | ✅ |  |
| MRC, MRC2 | ❌ |  |
| MRRC, MRRC2 | ❌ |  |
| MRS | ❌ |  |
| MSR | ❌ |  |
| MUL | ✅ |  |
| MVN (imm) | ✅ |  |
| MVN (reg) | ✅ |  |
| NEG | ✅ | See RSB |
| NOP | ✅ |  |
| ORN (imm) | ✅ |  |
| ORN (reg) | ✅ |  |
| ORR (imm) | ✅ |  |
| ORR (reg) | ✅ |  |
| PKHBT, PKHTB | ❌ |  |
| PLD (imm) | ❌ |  |
| PLD (lit) | ❌ |  |
| PLD (reg) | ❌ |  |
| PLI (imm, lit) | ❌ |  |
| PLI (reg) | ❌ |  |
| POP | ✅ |  |
| PSSBB | ❌ |  |
| PUSH | ✅ |  |
| QADD | ✅ |  |
| QADD16 | ❌ |  |
| QADD8 | ❌ |  |
| QASX | ❌ |  |
| QDADD | ❌ |  |
| QDSUB | ❌ |  |
| QSAX | ❌ |  |
| QSUB | ✅ |  |
| QSUB16 | ❌ |  |
| QSUB8 | ❌ |  |
| RBIT | ✅ |  |
| REV | ✅ |  |
| REV16 | ✅ |  |
| REVSH | ✅ |  |
| ROR (imm) | ✅ |  |
| ROR (reg) | ✅ |  |
| RRX | ✅ |  |
| RSB (imm) | ✅ |  |
| RSB (reg) | ✅ |  |
| SADD16 | ❌ |  |
| SADD8 | ❌ |  |
| SASX | ❌ |  |
| SBC (imm) | ✅ |  |
| SBC (reg) | ✅ |  |
| SBFX | ❌ |  |
| SDIV | ✅ |  |
| SEL | ❌ |  |
| SEV | ❌ |  |
| SHADD16 | ❌ |  |
| SHADD8 | ❌ |  |
| SHASX | ❌ |  |
| SHSAX | ❌ |  |
| SHSUB16 | ❌ |  |
| SHSUB8 | ❌ |  |
| SMLABB, SMLABT, SMLATB, SMLATT | ❌ |  |
| SMLAD, SMLADX | ❌ |  |
| SMLAL | ✅ |  |
| SMLALBB, SMLALBT, SMLALTB, SMLALTT | ❌ |  |
| SMLALD, SMLALDX | ❌ |  |
| SMLAWB, SMLAWT | ❌ |  |
| SMLSD, SMLSDX | ❌ |  |
| SMMLA, SMMLAR | ❌ |  |
| SMMLS, SMMLSR | ❌ |  |
| SMMUL, SMMULR | ❌ |  |
| SMUAD, SMUADX | ❌ |  |
| SMULBB, SMULBT, SMULTB, SMULTT | ❌ |  |
| SMULL | ✅ |  |
| SMULWB, SMULWT | ❌ |  |
| SMUSD, SMUSDX | ❌ |  |
| SSAT | ❌ |  |
| SSAT16 | ❌ |  |
| SSAX | ❌ |  |
| SSBB | ❌ |  |
| SSUB16 | ❌ |  |
| SSUBB | ❌ |  |
| STC, STC2 | ❌ |  |
| STM, STMIA, STMEA | ✅ |  |
| STMDB, STMFD | ✅ |  |
| STR (imm) | ✅ |  |
| STR (reg) | ✅ |  |
| STRB (imm) | ❌ |  |
| STRB (reg) | ❌ |  |
| STRBT | ❌ |  |
| STRD (imm) | ❌ |  |
| STREX | ✅ |  |
| STREXB | ❌ |  |
| STREXH | ❌ |  |
| STRH (imm) | ❌ |  |
| STRH (reg) | ❌ |  |
| STRHT | ❌ |  |
| STRT | ❌ |  |
| SUB (imm) | ✅ |  |
| SUB (reg) | ✅ |  |
| SUB (SP minus imm) | ❌ |  |
| SUB (SP minus reg) | ❌ |  |
| SVC | ❌ |  |
| SXTAB | ❌ |  |
| SXTAB16 | ❌ |  |
| SXTAH | ❌ |  |
| SXTB | ❌ |  |
| SXTB16 | ❌ |  |
| SXTH | ❌ |  |
| TBB, TBH | ✅ |  |
| TEQ (imm) | ✅ |  |
| TEQ (reg) | ✅ |  |
| TST (imm) | ✅ |  |
| TST (reg) | ✅ |  |
| UADD16 | ❌ |  |
| UADD8 | ❌ |  |
| UASX | ❌ |  |
| UBFX | ❌ |  |
| UDF | ✅ |  |
| UDIV | ✅ |  |
| UHADD16 | ❌ |  |
| UHADD8 | ❌ |  |
| UHASX | ❌ |  |
| UHSAX | ❌ |  |
| UHSUB16 | ❌ |  |
| UHSUB8 | ❌ |  |
| UMAAL | ✅ |  |
| UMLAL | ✅ |  |
| UMULL | ✅ |  |
| UQADD16 | ❌ |  |
| UQADD8 | ❌ |  |
| UQASX | ❌ |  |
| UQSAX | ❌ |  |
| UQSUB16 | ❌ |  |
| UQSUB8 | ❌ |  |
| USAD8 | ❌ |  |
| USADA8 | ❌ |  |
| USAT | ❌ |  |
| USAT16 | ❌ |  |
| USAX | ❌ |  |
| USUB16 | ❌ |  |
| USUB8 | ❌ |  |
| UXTAB | ❌ |  |
| UXTAB16 | ❌ |  |
| UXTAH | ❌ |  |
| UXTB | ❌ |  |
| UXTB16 | ❌ |  |
| UXTH | ❌ |  |
| VABS | ❌ |  |
| VADD | ❌ |  |
| VCMP, VCMPE | ❌ |  |
| VCVTA, VCVTN, VCVTP, and VCVTM | ❌ |  |
| VCVT, VCVTR (between floating-point and integer) | ❌ |  |
| VCVT (between floating-point and fixed-point) | ❌ |  |
| VCVT (between double-precision and single-precision) | ❌ |  |
| VCVTB, VCVTT | ❌ |  |
| VDIV | ❌ |  |
| VFMA, VFMS | ❌ |  |
| VFNMA, VFNMS | ❌ |  |
| VLDM | ❌ |  |
| VLDR | ❌ |  |
| VMAXNM, VMINNM | ❌ |  |
| VMLA, VMLS | ❌ |  |
| VMOV (imm) | ❌ |  |
| VMOV (reg) | ❌ |  |
| VMOV (ARM core reg to scalar) | ❌ |  |
| VMOV (scalar to ARM core reg) | ❌ |  |
| VMOV (between ARM core reg and single-precision reg) | ❌ |  |
| VMOV (between two ARM core regs and two single-precision regs) | ❌ |  |
| VMOV (between two ARM core regs and a doubleword reg) | ❌ |  |
| VMRS | ❌ |  |
| VMSR | ❌ |  |
| VMUL | ❌ |  |
| VNEG | ❌ |  |
| VNMLA, VNMLS, VNMUL | ❌ |  |
| VPOP | ❌ |  |
| VPUSH | ❌ |  |
| VRINTA, VRINTN, VRINTP, and VRINTM | ❌ |  |
| VRINTX | ❌ |  |
| VRINTZ, VRINTR | ❌ |  |
| VSEL | ❌ |  |
| VSQRT | ❌ |  |
| VSTM | ❌ |  |
| VSTR | ❌ |  |
| VSUB | ❌ |  |
| WFE | ❌ |  |
| WFI | ❌ |  |
| YIELD | ❌ |  |
