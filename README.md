# osdev
Writing an "OS" as Gymnasiearbete

## Installation Instructions
Packages needed for compilation into an iso file
`nasm grub-common xorriso make`

You also need to install rust which can be done through https://rustup.rs and also run this command to add rust src

`rustup component add rust-src --toolchain nightly-2023-08-23-x86_64-unknown-linux-gnu`

To compile it then you run `make clean iso` and the output is available at 

Running `make` will also launch qemu if you have it installed with the iso file attached
