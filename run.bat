qemu-system-x86_64 -accel tcg,thread=single -cpu Opteron_G5 -m 1024 -no-reboot -drive format=raw,media=cdrom,file=build/os-gymnasie.iso -serial stdio -smp 1 -vga std