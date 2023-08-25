%define ARCHITECTURE 0
section .multiboot_header 
header_start:
    ;magic (required by multiboot 2 spec)
    dd 0xE85250D6
    
    ;architecture 
    dd ARCHITECTURE

    ;length
    dd header_end - header_start

    ;checksum    
    ;checksum + magic + arch. + len. = 0
    dd 0x100000000 - (0xe85250d6 + ARCHITECTURE + (header_end - header_start))

    ;tag
    dw 0
    dw 0
    dd 8 ;size of tag including ourselves
header_end:
