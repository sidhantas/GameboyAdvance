b interupt
.org 0x08
mov pc, lr

.org 0xF0
interupt:
swi 0x1234
mov r1, #5
