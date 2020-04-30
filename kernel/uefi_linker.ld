ENTRY(kmain);

SECTIONS{

    . = 2M;
    .bss : {
        *(.bss .bss.*)
    }
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data.rel.ro : {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }
}