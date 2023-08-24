%define STACKSIZE 64

global start

section .text
bits 32
start:
    mov esp, stack_top 
    call check_multiboot
    call check_cpuid
    call check_longmode
    
    mov dword [0xb8000], 0x2f4b2f4f
    hlt




error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

check_cpuid:

    ;put flags into eax
    pushfd              
    pop eax         

    ;make a copy to restore later
    mov ecx, eax        ;copy into ecx
    xor eax, 1 << 21    ;flip bit 21 (ID)

    ;put eax into flags
    push eax            
    popfd               

    ;put flags into eax
    pushfd              
    pop eax

    ;return flags to original state
    push ecx
    popfd

    ;compare new to old
    cmp eax, ecx
    je .cpuid_fail
    ret

.cpuid_fail:
    mov al, "1"
    jmp error

check_longmode:

    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .longmode_fail

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29 
    jz .longmode_fail
    ret

.longmode_fail
    mov al, "2"
    jmp error

check_multiboot:
    cmp eax, 0x36d76289 
    jne .multiboot_fail
    ret 
.multiboot_fail:
    mov al, "0"
    jmp error


section .bss
stack_bottom:
    resb STACKSIZE
stack_top: