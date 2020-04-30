;************************************************************
;*          FileName    : boot_start                        *
;*          Author      : VenmoSnake                        *
;*          ProjectName : yacc                              *
;*          Time        : 2020/4/25 : 12:34                 *
;************************************************************

;==========================================
;               macro here
;==========================================


;==========================================
;               const here
;==========================================
MAGIC_CHECK_FAILED  equ     'M'
CPUID_CHECK_FAILED  equ     'C'
NO_LONG_MODE        equ     'L'
STACK_SIZE          equ     0x1000 * 4

;==========================================
;               entry point
;==========================================
global _start
extern long_mode_entry


;==========================================
;               bss section
;==========================================
section .bss
bits 32
align 0x1000
p4_table:
    resb 0x1000
p3_table:
    resb 0x1000
p2_table:
	resb 0x1000
stack_end:
	resb STACK_SIZE
stack_start:

;==========================================
;               read only section
;==========================================
section .rodata
gdt:
	dq 0
.code_segment: equ $ - gdt
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; temporary code segment
.pointer:
    dw $ - gdt - 1
    dq gdt

;==========================================
;               code section
;==========================================
section .text
bits 32
_start:
	mov esp,stack_start
	mov edi,ebx

	; check magic
	call check_multiboot
	call check_cpuid
	call check_long_mode
	call set_up_page_tables
	call enable_paging
	lgdt [gdt.pointer]
	jmp gdt.code_segment:long_mode_entry ; far jump
    hlt

check_multiboot:
    cmp eax, 0x36d76289
    jne .check_failed
    ret
.check_failed:
    mov al,MAGIC_CHECK_FAILED
    jmp error

set_up_page_tables:
    ; map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    ; map first P3 entry to P2 table
    mov eax, p2_table
    or eax, 0b11 ; present + writable
    mov [p3_table], eax

    ; map each P2 entry to a huge 2MiB page
    mov ecx, 0         ; counter variable

.map_p2_table:
    ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page
    or eax, 0b10000011 ; present + writable + huge
    mov [p2_table + ecx * 8], eax ; map ecx-th entry

    inc ecx            ; increase counter
    cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
    jne .map_p2_table  ; else map the next entry
    ret

enable_paging:
    ; load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret


; @param eax: check status code
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt


check_cpuid:
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ; in the FLAGS register. If we can flip it, CPUID is available.

    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al,CPUID_CHECK_FAILED
    jmp error

check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
	mov al,NO_LONG_MODE
    jmp error
