.global _main
_main:

msr CPSR_f, #0xd0000000
bx r3;
mov ip, #223
mov r1, #0xF
mov r13, #0xFF
orr r2, ip, r1, LSL#28 
mrs r2, CPSR



bx_dest:
    nop;
