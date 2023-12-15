.PHONY: all clean run iso

run-qemu:
	qemu-system-x86_64                             \
  -accel tcg,thread=single                       \
  -cpu Opteron_G5                                \
  -m 1024                                        \
  -no-reboot                                     \
  -no-shutdown \
  -drive format=raw,media=cdrom,file=build/os-gymnasie.iso    \
  -serial stdio                                  \
  -smp 1                                         \
  -vga std \



name ?= gymnasie
kernel_binary := build/kernel-$(name).bin
iso := build/os-$(name).iso

rust_target ?= x86_64-os
rust_optimisation ?= release
rust_bin := target/$(rust_target)/$(rust_optimisation)/libos.a


linker_script := asm-src/linker.ld
grub_cfg := asm-src/grub.cfg
assembly_source_files := $(wildcard asm-src/*.asm)
assembly_object_files := $(patsubst asm-src/%.asm, \
	build/asm/%.o, $(assembly_source_files))



all: $(kernel_binary)

clean:
	@-rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel_binary) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel_binary) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 
	

$(kernel_binary): $(assembly_object_files) $(linker_script)
	@cargo build --target $(rust_target).json --$(rust_optimisation)
	@ld -n -T $(linker_script) -o $(kernel_binary) $(assembly_object_files) $(rust_bin)

# compile assembly files
build/asm/%.o: asm-src/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -g -felf64 $< -o $@
