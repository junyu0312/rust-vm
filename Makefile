.PHONY: bios

bios: boot/bios.asm
	nasm -f bin boot/bios.asm -o bios.bin


