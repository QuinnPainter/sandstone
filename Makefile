ROM_NAME := enginetest
ARM9_ARCH := thumbv5te-none-eabi
ARM7_ARCH := thumbv4t-none-eabi

dev:
	cd arm9; cargo build
	cd arm7; cargo build
	ndstool -c $(ROM_NAME).nds -9 arm9/target/$(ARM9_ARCH)/debug/arm9 -7 arm7/target/$(ARM7_ARCH)/debug/arm7

release:
	cd arm9; cargo build --release
	cd arm7; cargo build --release
	ndstool -c $(ROM_NAME).nds -9 arm9/target/$(ARM9_ARCH)/release/arm9 -7 arm7/target/$(ARM7_ARCH)/release/arm7

# this is supposed to give debug symbols that work in NO$GBA, but it doesn't work. 
#debug:
#	cargo build
#	arm-none-eabi-objcopy --only-keep-debug target/thumbv5te-none-eabi/debug/nds-redux-test test.nef
#	arm-none-eabi-objcopy --strip-all target/thumbv5te-none-eabi/debug/nds-redux-test test.elf
#	ndstool -c test.nds -9 test.elf

asm:
	cd arm9; cargo rustc -- --emit asm
	cd arm7; cargo rustc -- --emit asm

clean:
	cd arm9; cargo clean
	cd arm7; cargo clean

.PHONY: dev release asm clean
