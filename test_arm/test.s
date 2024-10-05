.global _main
_main:
    mov r0, #target
    add r0, #1
    bx r0;
    nop;
    nop;

target:
    .thumb
    ldr r5, [pc, #12]
