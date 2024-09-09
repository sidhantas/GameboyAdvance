.global _main
_main:
mov r1, #200
mov r2, #0x320

strb r2, [r1]
ldr r3, [r1]


.text
test_word: .long 0x123
nop
test_word_2: .long 0x321
test_byte: 
    .byte 0xAB
    .byte 0x11
