CC=arm-none-eabi
TEST=test

all: $(TEST).s $(TEST).c
	$(CC)-as -mcpu=arm7tdmi $(TEST).s -o $(TEST).o
	$(CC)-objcopy -O binary $(TEST).o $(TEST).bin


dump: all
	$(CC)-objdump -D $(TEST).o
