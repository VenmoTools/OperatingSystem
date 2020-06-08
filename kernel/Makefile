TEMP_DIR=boot_temp
source=src/boot
kernel_lib=target/x86_64-unknown-none/debug/librslib.a
isofile=target/iosfiles

.PHONY: kernel dir all iso

all: kernel iso

dir:
	mkdir $(TEMP_DIR)

iosfiles: target/iosfiles/boot/kernel.bin
	grub-mkrescue

kernel.bin:  $(TEMP_DIR)/mutiboot.o $(TEMP_DIR)/boot_start.o  $(TEMP_DIR)/long_mode.o  $(kernel_lib)
	ld --nmagic -T $(source)/linker.ld $^ -o $(TEMP_DIR)/$@

$(TEMP_DIR)/mutiboot.o: $(source)/mutiboot.asm
	nasm -f elf64 $< -o $@

$(TEMP_DIR)/boot_start.o: $(source)/boot_start.asm
	nasm -f elf64 $< -o $@

$(TEMP_DIR)/long_mode.o: $(source)/long_mode.asm
	nasm -f elf64 $< -o $@

iso: kernel.bin
	mkdir -p $(isofile)/boot/grub
	cp $(TEMP_DIR)/$< $(isofile)/boot/$<
	cp $(source)/grub.cfg $(isofile)/boot/grub
	grub-mkrescue -o os.iso $(isofile)
# sudo apt-get install xorriso sudo apt-get install mtools
kernel:
	cargo xbuild

run:
	qemu-system-x86_64 -cdrom os.iso -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -D qemu.log -debugcon file:debug.log