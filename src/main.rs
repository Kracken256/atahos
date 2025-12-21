#![no_std]
#![no_main]

use core::panic::PanicInfo;

// The Multiboot header (Must be at the very start of the binary)
#[unsafe(link_section = ".multiboot")]
#[unsafe(no_mangle)]
pub static MULTIBOOT_HEADER: [u32; 3] = [
    0x1BADB002,                           // Magic number
    0x00,                                 // Flags
    0u32.wrapping_sub(0x1BADB002 + 0x00), // Checksum
];

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in b"Hello from Rust!".iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0B; // Light cyan color
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
