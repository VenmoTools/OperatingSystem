;************************************************************
;*          FileName    : long_mode                         *
;*          Author      : VenmoSnake                        *
;*          ProjectName : yacc                              *
;*          Time        : 2020/4/25 : 1:16                  *
;************************************************************



global long_mode_entry
extern kmain

section .text
bits 64
long_mode_entry:
	mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
	call kmain
