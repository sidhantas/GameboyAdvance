.global _main
_main:
    mov r0, #target
    add r0, #1
    bx r0;
    nop;
    nop;

target:
    .thumb
    lsr r0, r1, #32
