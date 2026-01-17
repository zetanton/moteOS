/* Linker script for ARM64 (AArch64) UEFI boot
 * Targets Raspberry Pi and other ARM64 UEFI systems
 */

ENTRY(efi_main)

/* Memory layout for ARM64 UEFI */
/* UEFI typically loads at 0x80000 (512KB) or higher */
MEMORY
{
    /* Code and read-only data */
    .text : {
        *(.text .text.*)
        *(.rodata .rodata.*)
    }
    
    /* Read-write data */
    .data : {
        *(.data .data.*)
    }
    
    /* Uninitialized data */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
}

SECTIONS
{
    /* Base address - UEFI will load us at a suitable address */
    . = 0x80000;
    
    /* Text section - executable code */
    .text : {
        *(.text .text.*)
        . = ALIGN(8);
    }
    
    /* Read-only data */
    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(8);
    }
    
    /* Read-write data */
    .data : {
        *(.data .data.*)
        . = ALIGN(8);
    }
    
    /* Uninitialized data (BSS) */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
    }
    
    /* Discard other sections */
    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
    
    /* Symbols for debugging */
    _text_start = .;
    _text_end = .;
    _data_start = .;
    _data_end = .;
    _bss_start = .;
    _bss_end = .;
}

/* ARM64 specific requirements */
/* Ensure 4KB page alignment for MMU */
PROVIDE(_stack_top = 0x1000000); /* 16MB - adjust based on system */
