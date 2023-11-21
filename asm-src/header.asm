%define ARCHITECTURE 0
section .multiboot_header 
header_start:
    ;"magic" bytes (specified by multiboot 2 spec)
    ;The primary purpose for this is for grub to be sure this isn't here by accident.
    dd 0xE85250D6
    
    ;architecture bytes as defined by the multiboot 2 spec
    dd ARCHITECTURE

    ;length of header
    dd header_end - header_start

    ;checksum
    ;as per the specification (checksum + magic + arch. + len. = 0)
    dd 0x100000000 - (0xe85250d6 + ARCHITECTURE + (header_end - header_start))

    ;tag
    dw 0
    dw 0
    dd 8 ;size of tag including ourselves
header_end:
