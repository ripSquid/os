global long_mode_start


section .text
bits 64
long_mode_start:
    ;empty registers associated with moving memory around
    mov ax, 0
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov ds, ax
    mov es, ax

    ;jump into our rust entry point (rust_start in lib.rs)
    extern rust_start
    call rust_start
    
    mov dword [0xb8000], 0x2f4b2f4f
    hlt