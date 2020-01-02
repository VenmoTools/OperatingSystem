
GO= cargo
TEST_FLAGS=xtest
RUST_BUILD=$(GO) xbuild
RUST_FLAGS=--target x86-64.json
IMAGE=target/x86-64/debug/bootimage-kernel.bin

build: src/main.rs
	$(RUST_BUILD) $(RUST_FLAGS)

boot:
	$(GO) bootimage

run: boot
	qemu-system-x86_64 -drive format=raw,file=$(IMAGE)

# target remote localhost:1234 GDB
# LLDB gdb-remote 1234
# 0x20940c
debug: boot
	qemu-system-x86_64 -s -S -m 512 -hda $(IMAGE)

test:
	$(GO) $(TEST_FLAGS)