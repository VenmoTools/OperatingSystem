;************************************************************
;*          FileName    : long_mode                         *
;*          Author      : VenmoSnake                        *
;*          ProjectName : boot                              *
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
	call kmain
