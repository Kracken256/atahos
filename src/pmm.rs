use alloc::sync::Arc;
use limine::memory_map::EntryType;
use log::info;

use crate::limine::MEMORY_MAP_REQUEST;

pub const PAGE_SIZE: usize = 4096;

pub type Frame = [u8; PAGE_SIZE];

#[repr(C)]
struct FreeListFrame {
    next: *mut FreeListFrame,
}

pub struct PhysicalMemoryAllocator {
    free_list: *mut FreeListFrame,
    available_frames: usize,
}

impl PhysicalMemoryAllocator {
    pub const fn new() -> Self {
        Self {
            free_list: core::ptr::null_mut(),
            available_frames: 0,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<*mut Frame> {
        if self.free_list.is_null() {
            return None;
        }

        let frame = self.free_list;
        self.free_list = unsafe { (*frame).next };
        self.available_frames -= 1;
        Some(frame as *mut Frame)
    }

    pub unsafe fn deallocate_frame(&mut self, frame: *mut Frame) {
        let frame_ptr = frame as *mut FreeListFrame;
        unsafe { (*frame_ptr).next = self.free_list }
        self.free_list = frame_ptr;
        self.available_frames += 1;
    }

    pub unsafe fn add_region(&mut self, base: *mut u8, size: usize) {
        let mut current = base as usize;
        let end = current + size;

        current = current.next_multiple_of(PAGE_SIZE);

        while current + PAGE_SIZE <= end {
            unsafe { self.deallocate_frame(current as *mut Frame) }
            current += PAGE_SIZE;
        }
    }

    pub fn available_frames(&self) -> usize {
        self.available_frames
    }

    pub fn total_memory(&self) -> usize {
        self.available_frames() * PAGE_SIZE
    }
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

pub fn init() -> Arc<spin::Mutex<PhysicalMemoryAllocator>> {
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

    Arc::new(spin::Mutex::new(pmm))
}
