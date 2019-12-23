
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

debug: boot
	qemu-system-x86_64 -drive format=raw,file=$(IMAGE) -append "root=/dev/ram console=ttyS0 rdinit=/sbin/init mem=1024M" -S -s

test:
	$(GO) $(TEST_FLAGS)