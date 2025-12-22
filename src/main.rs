#![no_std]
#![no_main]

mod multiboot;

use core::panic::PanicInfo;
use log::error;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    /* Initialize the COM port loggger */
    com_logger::builder()
        .filter(log::LevelFilter::Trace)
        .setup();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
