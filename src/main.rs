#![no_std]
#![no_main]

extern crate alloc;

mod limine;
mod pmm;
mod vmm;

use crate::{
    limine::{FRAMEBUFFER_REQUEST, MEMORY_MAP_REQUEST, validate_limine_version},
    pmm::PhysicalMemoryAllocator,
};
use ::limine::memory_map::EntryType;
use alloc::{collections::btree_map::BTreeMap, format, sync::Arc};
use core::panic::PanicInfo;
use log::{error, info};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    initialize_logger();
    validate_limine_version();
    let pmm = initialize_pmm();
    initialize_vmm(pmm.clone());
    splash_screen();

    let mut m: BTreeMap<i32, i32> = BTreeMap::new();
    m.insert(1, 2);
    m.insert(3, 4);
    m.insert(5, 6);
    info!("BTreeMap contents: {:?}", m);
    info!("{}", format!("Formatted string: {}", 42));

    loop {}
}

fn initialize_logger() {
    com_logger::builder()
        .filter(log::LevelFilter::Trace)
        .setup();
}

fn memory_region_to_string(ty: EntryType) -> &'static str {
    match ty {
        EntryType::USABLE => "Usable",
        EntryType::RESERVED => "Reserved",
        EntryType::ACPI_RECLAIMABLE => "ACPI Reclaimable",
        EntryType::ACPI_NVS => "ACPI NVS",
        EntryType::BAD_MEMORY => "Bad Memory",
        EntryType::BOOTLOADER_RECLAIMABLE => "Bootloader Reclaimable",
        _ => "Unknown",
    }
}

fn initialize_pmm() -> Arc<PhysicalMemoryAllocator> {
    info!("Supplying memory regions to physical memory manager...");

    let mmap_response = MEMORY_MAP_REQUEST
        .get_response()
        .expect("expected memory map from limine bootloader");

    let mut pmm = PhysicalMemoryAllocator::new();

    for entry in mmap_response.entries() {
        info!(
            "Region: base={:#x}, length={:#x}, type={}",
            entry.base,
            entry.length,
            memory_region_to_string(entry.entry_type)
        );

        match entry.entry_type {
            EntryType::USABLE => {
                info!("Adding region to PMM...");

                let base = entry.base as *mut u8;
                let size = entry.length as usize;

                unsafe { pmm.add_region(base, size) }
            }

            _ => continue,
        }
    }

    let total_mem = pmm.total_memory();
    let total_frames = pmm.available_frames();

    info!(
        "Total memory managed by PMM: {} bytes, {:.2} MiB, {:.2} GiB",
        total_mem,
        total_mem as f32 / (1024.0 * 1024.0),
        total_mem as f32 / (1024.0 * 1024.0 * 1024.0)
    );
    info!("Total available frames: {}", total_frames);
    info!("Physical memory manager initialized.");

    Arc::new(pmm)
}

fn initialize_vmm(_pmm: Arc<PhysicalMemoryAllocator>) {
    info!("Initializing virtual memory manager...");

    // TODO: Implement VMM initialization logic here, using the provided PMM.

    info!("Virtual memory manager initialized.");
}

fn splash_screen() {
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
                *pixel_ptr.add(0) = 0xFF; // Blue
                *pixel_ptr.add(1) = 0x00; // Green
                *pixel_ptr.add(2) = 0x00; // Red
                *pixel_ptr.add(3) = 0xFF; // Alpha
            }
        }
    }
}
