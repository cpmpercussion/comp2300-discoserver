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
| ASR (imm) |  |  |
| ASR (reg) |  |  |
| B |  |  |
| BFC |  |  |
| BFI |  |  |
| BIC (imm) |  |  |
