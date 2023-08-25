.PHONY: all clean run iso

run-qemu:
	qemu-system-x86_64                             \
  -accel tcg,thread=single                       \
  -cpu Opteron_G5                                \
  -m 1024                                        \
  -no-reboot                                     \
  -drive format=raw,media=cdrom,file=build/os-gymnasie.iso    \
  -serial stdio                                  \
  -smp 1                                         \
  -vga std

arch ?= gymnasie
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

target ?= x86_64-os
rust_os := os/target/$(target)/release/libos.a


linker_script := asm-src/linker.ld
grub_cfg := asm-src/grub.cfg
assembly_source_files := $(wildcard asm-src/*.asm)
assembly_object_files := $(patsubst asm-src/%.asm, \
	build/asm/%.o, $(assembly_source_files))



all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 
	

$(kernel): $(assembly_object_files) $(linker_script)
	@cd os; cargo build --target $(target).json --release
	@ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

# compile assembly files
build/asm/%.o: asm-src/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
