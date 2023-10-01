ENTRY(start)

SECTIONS {
    . = 1M;
    
    .boot : {
        KEEP(*(.multiboot_header))
        . = ALIGN(4K);
    }
    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }
    .text : ALIGN(4K) {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .data : {
        *(.data .data.*)
        . = ALIGN(4K);
    }
    .bss : {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    .debug_aranges  : ALIGN(4K) {
        *(.debug_aranges)
    }
    .debug_info   : ALIGN(4K) {
    *(.debug_info)
    }
    .debug_abbrev  : ALIGN(4K) {
    *(.debug_abbrev)
    }
    .debug_line    : ALIGN(4K) {
    *(.debug_line)
    }
    .comment   : ALIGN(4K) {
    *(.comment)
    }

}