.global _main
_main:
add r1, r1, #1;
add r2, r1, #2;
_branch_addr:
add r0, r1, r2;
add r0, r1, r2;
add r0, r1, r2;
add r0, r1, r2;
nop;
nop;
nop;
nop;
nop;
nop;

BL _branch_addr;
nop;
nop;
