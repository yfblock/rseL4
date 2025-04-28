ENTRY(_start)

PROVIDE(KERNEL_OFFSET = 0xffff000000000000);
PROVIDE(START_ADDR = %START_ADDR%);

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = START_ADDR;
    _skernel = .;

    .boot . : AT(ADDR(.boot) - KERNEL_OFFSET)
    {
        *(.boot.text)
        *(.boot.rodata)
        *(.boot.data)
        . = ALIGN(64K);
    }
    . = ALIGN(4K);
    ki_boot_end = .;

    .text . : AT(ADDR(.text) - KERNEL_OFFSET) {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        etext = .;
    }

    .rodata . : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data . : AT(ADDR(.data) - KERNEL_OFFSET)  {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        _edata = .;
    }

    .bss . : AT(ADDR(.bss) - KERNEL_OFFSET)  {
        *(.bss.stack)
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        _ebss = .;
    }

    . = ALIGN(4K);
    ki_end = .;
    PROVIDE(_ekernel = .);
    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}