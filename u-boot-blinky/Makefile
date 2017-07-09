all:
	arm-linux-gnueabihf-as -o blink.o blink.s	
	arm-linux-gnueabihf-ld -o blink.elf -T blink.ld blink.o
	arm-linux-gnueabihf-objcopy -O binary blink.elf blink.bin
