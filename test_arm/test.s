.global _main
_main:
    subs r1, r15, #8
    nop;
    nop;
    nop;
    add r15, r1, #0
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

