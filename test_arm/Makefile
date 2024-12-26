CC=arm-none-eabi
TEST=test

all: $(TEST).s $(TEST).c
	$(CC)-as -mcpu=arm7tdmi $(TEST).s -o $(TEST).o
	$(CC)-objcopy -O binary $(TEST).o $(TEST).bin

	$(CC)-gcc -O0 -ffreestanding -nostdlib -fomit-frame-pointer -c test.c -o test_c_program.o
	$(CC)-objcopy -O binary test_c_program.o test_c_program.bin


dump: all
	$(CC)-objdump -D $(TEST).o
