%define STACKSIZE 64

global start
extern long_mode_start

section .text
bits 32
start:
    mov esp, stack_top 
    call check_multiboot
    call check_cpuid
    call check_longmode

    call page_table_setup
    call enable_paging
    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

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

.longmode_fail:
    mov al, "2"
    jmp error

check_multiboot:
    cmp eax, 0x36d76289 
    jne .multiboot_fail
    ret 
.multiboot_fail:
    mov al, "0"
    jmp error


page_table_setup:

    mov eax, p3_table
    or eax, 0b11
    mov [p4_table], eax
    
    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    mov ecx, 0

.map_p2_table:

    mov eax, 0x200000 
    mul ecx
    or eax, 0b10000011
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ret

enable_paging:

    mov eax, p4_table
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr


    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    
    ret




section .rodata
gdt64:
    dq 0
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.pointer:
    dw $ - gdt64 - 1
    dq gdt64


;statically allocated data
section .bss

align 4096

;memory paging tables
p4_table:
    resb 4096
p3_table: 
    resb 4096
p2_table:
    resb 4096



stack_bottom:
    resb STACKSIZE
stack_top: