.global _main
_main:
b  b_dest


.org 0x9c2
main_2:
nop;
nop;
nop;
add r2, r3, #1

.org 0x1930
b_dest:

ldr r2, =bx_dest
add r2, r2, #1
bx r2
bx_dest:
.thumb
    add r1, r1, r1
    bl main_2;
    nop;
