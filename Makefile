ELF=rppico-spi-to-uart

PLATFORM=thumbv6m-none-eabi

SRCS:=$(wildcard src/*.rs)

target/$(PLATFORM)/release/$(ELF).uf2: target/$(PLATFORM)/release/$(ELF)
	elf2uf2-rs target/$(PLATFORM)/release/$(ELF) target/$(PLATFORM)/release/$(ELF).uf2

toolchain:
	rustup install stable
	rustup default stable
	rustup target add $(PLATFORM)
	rustup component add llvm-tools-preview

target/$(PLATFORM)/release/$(ELF): $(SRCS)
	cargo build --release --target=$(PLATFORM) --bin=$(ELF)
	cargo size --release --target=$(PLATFORM) --bin=$(ELF) -- -A

clean:
	cargo clean

download: target/$(PLATFORM)/release/$(ELF).uf2
	# Upload to device and execute
	sudo picotool load ./target/$(PLATFORM)/release/$(ELF).uf2 -x -f
	# File is now on the raspberry pico.
	# To repeat, you might need to reboot while the reset button is pressed
