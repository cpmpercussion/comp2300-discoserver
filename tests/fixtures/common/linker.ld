/* We skip the normal startup script and start directly at main. This means .data and the sp are not initialised */
ENTRY(main)

_stack_end = 0x20018000;

MEMORY {
  RAM (xrw) : ORIGIN = 0x20000000, LENGTH = 96K
  RAM2 (xrw) : ORIGIN = 0x10000000, LENGTH = 32K
  FLASH (rx) : ORIGIN = 0x8000000, LENGTH = 1024K
}

SECTIONS {
  .isr_vector : {
    . = ALIGN(4);
    KEEP(*(.isr_vector))
    . = ALIGN(4);
  } > FLASH

  .text : {
    . = ALIGN(4);
    _text_start = .;
    *(.text)
    . = ALIGN(4);
    _text_end = .;
  } > FLASH

  _data_start_flash = LOADADDR(.data);

  .data : {
    . = ALIGN(4);
    _data_start_ram = .;
    KEEP(*(.data)) /* NOTE: KEEP is helpful to prevent it discarding unused labels when --gc-sections is passed */
    . = ALIGN(4);
    _data_end_ram = .;
  } > RAM AT > FLASH


  .bss : {
    . = ALIGN(4);
    _bss_start = .;
    *(.bss)
    . = ALIGN(4);
    _bss_end = .;
  } > RAM

  .ARM.attributes 0 : { *(.ARM.attributes) }
}
