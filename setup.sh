#!/bin/bash

rm -rf limine
rm -rf iso_root_temp

git clone https://codeberg.org/Limine/Limine.git limine --branch=v10.x-binary --depth=1
make -C limine

cp limine/limine-bios-cd.bin limine/limine-bios.sys limine/limine-uefi-cd.bin \
   iso_root/boot/limine/

cp limine/*.EFI iso_root/EFI/BOOT/
