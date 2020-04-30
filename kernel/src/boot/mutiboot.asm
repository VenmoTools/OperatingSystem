;************************************************************
;*          FileName    : mutiboot                          *
;*          Author      : VenmoSnake                        *
;*          ProjectName : yacc                              *
;*          Time        : 2020/4/25 : 12:24                 *
;************************************************************

section .mutiboot_header

%macro muti_header 2
	dd 0xe85250d6 ; magic number
	dd 0
	dd %1   ; header length
	dd %2   ; check sum
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
%endmacro

header_begin:
muti_header  header_end - header_begin, 0x100000000 - (0xe85250d6 + 0 + (header_end - header_begin))
header_end:





