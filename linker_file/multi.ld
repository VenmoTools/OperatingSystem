ENTRY(_start)


SECTIONS{

	. = 1M;

	.boot : {
		KEEP(*(.mutiboot_header))
	}
	.bss : {
		*(.bss .bss.*)
	}
	.text : {
	    KEEP(*(.text.init))
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