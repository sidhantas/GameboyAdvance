.global _main
_main:
ldr r1, _branch_addr;
str r1, _branch_addr;
sub r1, #1
BX r1;
nop;
nop;
nop;
nop;
nop;
nop;

_branch_addr:
    nop;
