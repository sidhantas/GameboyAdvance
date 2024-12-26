#include <stdint.h>
void _start() {
    __asm__("mov sp, #0x6000000");
    uint8_t *i = (uint8_t *)0x4000202;
    *i = (uint8_t)0x0F;
    i[1] = (uint8_t)0xab;

    return;
}
