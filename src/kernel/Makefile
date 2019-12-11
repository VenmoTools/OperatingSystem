
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

test:
	$(GO) $(TEST_FLAGS)