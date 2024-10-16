.global _main
_main:
    mov r0, #target
    mov sp, #500
    add r0, #1
    bx r0;
    nop;
    nop;

target:
    .thumb
    bl destination
    nop;
    nop

destination:
    .thumb
    mov r1, r2

