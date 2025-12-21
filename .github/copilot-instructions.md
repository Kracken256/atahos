# Atah OS - AI Coding Agent Instructions

## Project Overview

Atah OS is a UEFI-based operating system written in Rust, targeting bare-metal x86_64 architecture. This is a low-level systems project with no standard library (`#![no_std]`).

## Architecture

### Workspace Structure

This is a Cargo workspace with four primary components:

- **`boot/`** - UEFI bootloader (builds to `.efi` executable)
- **`core/`** - Core OS components (currently stub)
- **`home/`** - Home/user-space components (currently stub)
- **`libs/`** - Shared libraries for OS components (currently stub)
- **`image/`** - EFI boot image structure with directories for deployment

### Build Targets

- The bootloader (`boot/`) targets **`x86_64-unknown-uefi`** exclusively
- Build configuration is in `boot/.cargo/config.toml` with default target set
- Toolchain specified in `boot/rust-toolchain.toml`

## Critical Build Workflow

### Building the Bootloader

```bash
cd boot
cargo build --target x86_64-unknown-uefi
```

The output `.efi` file goes to `target/x86_64-unknown-uefi/debug/atahos_boot.efi` (or `release/`).

### Deployment

The bootloader must be placed at `image/EFI/EFI/BOOT/BOOTx64.EFI` for UEFI systems to recognize it.

### Debugging

Use `objdump -d` to inspect the compiled `.efi` binary (as seen in terminal history).

## Code Conventions

### No Standard Library

All code uses `#![no_std]` and `#![no_main]` attributes. Use `uefi` crate APIs instead of std.

### Entry Points

- Bootloader entry: `#[entry]` macro from `uefi` crate (see `boot/src/main.rs`)
- Must initialize UEFI helpers: `uefi::helpers::init().unwrap()`

### Dependencies

- **`uefi = "0.36"`** with features `["panic_handler", "logger"]` for bootloader
- **`log = "0.4"`** for logging (outputs via UEFI console)

### Rust Edition

Uses **`edition = "2024"`** (latest edition) across all crates.

## Common Patterns

### UEFI API Usage

```rust
use uefi::prelude::*;
use log::info;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Log messages go to UEFI console");
    boot::stall(Duration::from_secs(10));  // Use UEFI boot services
    Status::SUCCESS
}
```

### Image Structure

- `image/EFI/EFI/BOOT/` - Standard UEFI boot path
- `image/EFI/CORE/` - Core OS modules
- `image/HOME/` - User-space files
- `image/MODS/` - Kernel modules/extensions

## Important Notes

- **No tests**: The bootloader cannot run standard Rust tests (incompatible with `#![no_std]` UEFI target)
- **Separate target directories**: Each workspace member has its own target directory (see `.gitignore`)
- **UEFI limitations**: No heap allocation, threading, or file I/O without explicit UEFI protocol usage
- **Status returns**: Functions return `uefi::Status` instead of `Result`

## When Adding Features

1. Keep bootloader minimal - only boot logic in `boot/`
2. OS functionality goes in `core/` (kernel) or `home/` (userspace)
3. Shared utilities go in `libs/`
4. Always use UEFI-safe APIs - check `uefi` crate documentation
5. Remember: no panicking, no unwinding - use UEFI status codes
