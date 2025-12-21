# Atah OS – AI working notes

- **What this is:** Minimal Multiboot v1 kernel that writes a greeting to the VGA text buffer; fully `#![no_std]`/`#![no_main]` with manual entry point and panic loop. Core logic in [src/main.rs](src/main.rs#L1-L35).
- **Target/abi:** Builds for `i686-unknown-linux-gnu` with a bare-metal-style link; configured via [.cargo/config.toml](.cargo/config.toml#L1-L9). Do not use `std` or host syscalls; stick to `core`.
- **Link/layout:** Custom linker script [src/linker.ld](src/linker.ld#L1-L8) places `.text` at 1MiB and keeps the Multiboot header first. Keep `_start`, `MULTIBOOT_HEADER`, and the panic/eh symbols present when editing entry code.
- **Build command:** `cargo build` (target is fixed in config). Profiles force `panic = "abort"` per [Cargo.toml](Cargo.toml#L1-L8).
- **ISO pipeline:** Build triggers [build.rs](build.rs#L5-L49) which rewrites `build-iso.sh` for the active profile; script copies `target/i686-unknown-linux-gnu/<profile>/atahos_core` into `iso_root/boot/` and runs `grub-mkrescue`. After a successful build, run `./build-iso.sh` to emit `target/<profile>/atahos.iso`.
- **Prereqs:** Host needs `grub-mkrescue` (and its deps like `xorriso`) installed; ensure the Rust target is added (`rustup target add i686-unknown-linux-gnu`).
- **Boot media layout:** GRUB menu at [iso_root/boot/grub/grub.cfg](iso_root/boot/grub/grub.cfg#L1-L6) loads `/boot/atahos_core` via `multiboot`. Keep the binary name/path stable or update both the copy step and menu.
- **Editor setup:** VS Code sets `rust-analyzer.cargo.target` to `i686-unknown-none` in [.vscode/settings.json](.vscode/settings.json#L1-L3) to avoid host std; cargo actually builds for `i686-unknown-linux-gnu`. If diagnostics look odd, align these targets.
- **Modifying entry:** `_start` writes bytes directly to VGA memory at `0xb8000` with attribute `0x0B`. Adjust text or color in [src/main.rs](src/main.rs#L16-L27); keep the infinite loop to avoid falling off entry.
- **Adding code:** Stay `no_std`; avoid dynamic allocations and RTTI. If introducing new sections (data/bss), extend [src/linker.ld](src/linker.ld#L1-L8) accordingly and preserve `.multiboot` at the start.
- **Rebuild triggers:** build script reruns when `src/` or `iso_root/` change; no need to touch `build.rs` manually when editing GRUB config or kernel code.

Use these notes to keep the kernel bootable, the ISO reproducible, and cargo/GRUB expectations aligned.
