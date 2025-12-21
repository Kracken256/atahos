#![no_std]
#![no_main]

mod console;
mod multiboot;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut console = console::Console::new();

    console.clear();
    console.write_str("Hello, world!\n");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
