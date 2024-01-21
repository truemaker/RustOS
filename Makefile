run: build
	qemu-system-x86_64 target/x86_64-rustos/debug/bootimage-rustos.bin
build:
	cargo bootimage
