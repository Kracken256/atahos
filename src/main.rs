#![no_std]
#![no_main]

extern crate alloc;

mod limine;
mod logger;
mod pmm;
mod vmm;

use crate::{
    limine::{FRAMEBUFFER_REQUEST, validate_limine_version},
    logger::initialize_logger,
    pmm::initialize_pmm,
    vmm::initialize_vmm,
};
use alloc::{collections::btree_map::BTreeMap, format, vec::Vec};
use log::{error, info};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    initialize_logger();
    validate_limine_version();
    initialize_interrupts();
    initialize_paging();
    let pmm = initialize_pmm();
    initialize_vmm(pmm);
    splash_screen();

    let mut m: BTreeMap<i32, i32> = BTreeMap::new();
    m.insert(1, 2);
    m.insert(3, 4);
    m.insert(5, 6);
    info!("BTreeMap contents: {:?}", m);
    info!("{}", format!("Formatted string: {}", 42));

    let mut v: Vec<u8> = alloc::vec::Vec::new();
    v.resize(10000, 0);

    loop {}
}

fn initialize_interrupts() {
    info!("Initializing interrupts...");

    // TODO: Setup IDT, PICs, and enable interrupts

    info!("Interrupts initialized.");
}

fn initialize_paging() {
    info!("Initializing paging...");

    // TODO: Setup initial page tables and enable paging

    info!("Paging initialized.");
}

fn splash_screen() {
    const COLOR: u32 = 0x001f1f1f; // Blue color in ARGB format

    let fb = FRAMEBUFFER_REQUEST
        .get_response()
        .unwrap()
        .framebuffers()
        .next()
        .unwrap();

    for y in 0..fb.height() {
        for x in 0..fb.width() {
            let offset = (y * fb.pitch() + x * (fb.bpp() as u64 / 8)) as usize;
            unsafe {
                let pixel_ptr = fb.addr().add(offset);
                *pixel_ptr.add(0) = COLOR.wrapping_shr(0) as u8; // Blue
                *pixel_ptr.add(1) = COLOR.wrapping_shr(8) as u8; // Green
                *pixel_ptr.add(2) = COLOR.wrapping_shr(16) as u8; // Red
                *pixel_ptr.add(3) = COLOR.wrapping_shr(24) as u8; // Alpha
            }
        }
    }
}
