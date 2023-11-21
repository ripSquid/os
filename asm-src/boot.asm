%define STACKSIZE 8192

global start
extern long_mode_start
extern interrupt_wrapper

section .text
bits 32
start:
    mov esp, stack_top 
    mov edi, ebx                    ;Moves the pointer to the multiboot info structure, not used until WAAAAY later.
    call check_multiboot            ;check if proper magic value was returned by bootloader
    call check_cpuid                ;check if cpu supports cpuid
    call check_longmode             ;check if cpu supports long mode

    call page_table_setup           ;create identity page table
    call enable_paging              ;enable paging
    lgdt [gdt64.pointer]            ;load 10 byte size+address pointer into gdt register
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

;Sets up the first gigabyte of memory to be identity mapped
page_table_setup:

    mov eax, p3_table   ;insert address to p3_table into eax
    or eax, 0b11        ;present + writable flags
    mov [p4_table], eax ;assign to the 0th entry of p4_table 
    
    mov eax, p2_table   ;insert address of p2_table to eax
    or eax, 0b11        ;present + writable
    mov [p3_table], eax ;assign to the 0th entry of p3_table

    mov ecx, 0          ;reset ecx (will use in a second)  

.map_p2_table:

    ;construct page table entry
    mov eax, 0x200000               ;the size of 2MiB in bytes (1024*1024*2 in hex)
    mul ecx                         ;multiply with ecx (which is our page counter, think of it as i in a for loop)
    or eax, 0b10000011              ;same as above + huge flag (This tells PAE that our page is 2MiB and negates the need for a p1_table)

    mov [p2_table + ecx * 8], eax   ;assign the page table entry we just created to the ecx'th entry in the p2_table
    inc ecx                         ;increase ecx in preparation for next iteration
    cmp ecx, 512                    ;compare, are we on iteration 512? (finished)
    jne .map_p2_table               ;if not, jump back to .map_p2_table

    ;map entry 511 of p4 table to itself, making a recursive page table.
    mov eax, p4_table               ;insert address to p4_table into eax
    or eax, 0b11                    ;present + writable
    mov [p4_table + (511 * 8)], eax ;assign to the 511th entry of p4_table

    ;return
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



;static read only data
section .rodata
gdt64:
    dq 0                                        ; Empty beginner segment into the GDT
.code: equ $ - gdt64                            ; kernel code segment in GDT (long mode)
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.k_data: equ $ - gdt64                          ; kernel data segment in GDT (long mode + writable)
    dq (1<<44) | (1<<47) | (1<<53) | (1<<41)
.pointer:                                       ; pointer to GDT
    dw $ - gdt64 - 1
    dq gdt64


;statically allocated data
section .bss

align 4096

;space for memory paging tables
p4_table:
    resb 4096
p3_table: 
    resb 4096
p2_table:
    resb 4096

;space for CPU stack
stack_bottom:
    resb STACKSIZE
stack_top: 
