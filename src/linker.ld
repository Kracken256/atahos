ENTRY(_start)
SECTIONS {
    . = 1M;
    .text : {
        KEEP(*(.multiboot))
        *(.text)
    }
}