### Misc
Peripherals and interrupts are not supported. Audio specifically is supported, by detecting the call to `BSP_AUDIO_OUT_Play_Sample`.

### Instructions

| Instruction | Supported | Comments |
|-------------------|-----------|-------------|
| ADC (imm) | ✅ |  |
| ADC (reg) | ✅ |  |
| ADD (imm) | ✅ |  |
| ADD (reg) | ✅ |  |
| ADD (SP plus imm) | ✅ | Only narrow |
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
| CLREX | ❌ |  |
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
| LDMDB, LDMEA |  |  |
| LDR (imm) |  |  |
| LDR (lit) |  |  |
| LDR (reg) |  |  |
| LDRB (imm) |  |  |
| LDRB (lit) |  |  |
| LDRB (reg) |  |  |
| LDRBT |  |  |
| LDRD (imm) |  |  |
| LDRD (lit) |  |  |
| LDREX |  |  |
| LDREXB |  |  |
| LDREXH |  |  |
| LDRH (imm) |  |  |
| LDRH (lit) |  |  |
| LDRH (reg) |  |  |
| LDRHT |  |  |
| LDRSB (imm) |  |  |
| LDRSB (lit) |  |  |
| LDRSB (reg) |  |  |
| LDRSBT |  |  |
| LDRSH (imm) |  |  |
| LDRSH (lit) |  |  |
| LDRSH (reg) |  |  |
| LDRSHT |  |  |
| LDRT |  |  |
| LSL (imm) |  |  |
| LSL (reg) |  |  |
| LSR (imm) |  |  |
| LSR (reg) |  |  |
| MCR, MCR2 |  |  |
| MCRR, MCRR2 |  |  |
| MLA |  |  |
| MLS |  |  |
| MOV (imm) |  |  |
| MOV (reg) |  |  |
| MOV (shifted reg) |  |  |
| MOVT |  |  |
| MRC, MRC2 |  |  |
| MRRC, MRRC2 |  |  |
| MRS |  |  |
| MSR |  |  |
| MUL |  |  |
| MVN (imm) |  |  |
| MVN (reg) |  |  |
| NEG |  |  |
| NOP |  |  |
| ORN (imm) |  |  |
| ORN (reg) |  |  |
| ORR (imm) |  |  |
| ORR (reg) |  |  |
| PKHBT, PKHTB |  |  |
| PLD (imm) |  |  |
| PLD (lit) |  |  |
| PLD (reg) |  |  |
| PLI (imm, lit) |  |  |
