%define ARCHITECTURE 0
%define MAGIC 0xE85250D6
%define HEADER_LENGTH (header_end - header_start)

section .multiboot_header 
header_start:
    ;"magic" bytes (specificerat av multiboot 2 spec.)
    dd MAGIC
    
    ;architecture bytes som definerat av multiboot 2 spec.
    dd ARCHITECTURE

    ;längden av våran header i bytes
    dd header_end - header_start

    ;checksum
    ;för att vara giltig ska (checksum + magic + arch. + header len. = 0)
    ;vi använder inbyggda assemblersymboler för att uppnå detta.
    dd 0x100000000 - (MAGIC + ARCHITECTURE + HEADER_LENGTH)

    ;taggar (i detta fallet bara en slut-tag)
    dw 0
    dw 0 
    dd 8 ;storlek av taggarna i bytes
header_end:
