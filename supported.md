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
