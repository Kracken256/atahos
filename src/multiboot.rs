// The Multiboot header (Must be at the very start of the binary)
#[unsafe(link_section = ".multiboot")]
#[unsafe(no_mangle)]
pub static MULTIBOOT_HEADER: [u32; 3] = [
    0x1BADB002,                           // Magic number
    0x00,                                 // Flags
    0u32.wrapping_sub(0x1BADB002 + 0x00), // Checksum
];
