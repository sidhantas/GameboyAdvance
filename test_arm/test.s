.global _main
_main:
mov r1, #200
mov r2, #-5
mov r3, #500
    
str r2, [r1]
swpb r4, r3, [r1]
stmda r5!, {r6, r7}


.text
test_word: .long 0x123
nop
test_word_2: .long 0x321
test_byte: 
    .byte 0xAB
    .byte 0x11
