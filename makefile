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

linker_script := asm-src/linker.ld
grub_cfg := asm-src/grub.cfg
assembly_source_files := $(wildcard asm-src/*.asm)
assembly_object_files := $(patsubst asm-src/%.asm, \
	asm-src/%.o, $(assembly_source_files))



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
	@mkdir build
	@ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
