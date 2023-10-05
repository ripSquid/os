global long_mode_start

%macro pushaq 0
  push rax
  push rcx
  push rdx
  push rbx
  push rbp
  push rsi
  push rdi
  push r8
  push r9
  push r10
  push r11

%endmacro

%macro popaq 0
  pop r11
  pop r10
  pop r9
  pop r8
  pop rdi
  pop rsi
  pop rbp
  pop rbx
  pop rdx
  pop rcx
  pop rax
%endmacro

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
    
    ;give address of interrupt_wrapper to rust

    mov rdi, interrupt_wrapper

    ;jump into our rust entry point (rust_start in lib.rs)
    extern rust_start
    call rust_start
    

    hlt
