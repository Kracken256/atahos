#!/bin/bash
set -e

# Copy the compiled binary to iso_root
cp target/i686-unknown-linux-gnu/debug/atahos_core iso_root/boot/atahos_core

# Build the ISO
grub-mkrescue -o target/debug/atahos.iso iso_root

echo "ISO created at target/debug/atahos.iso"
