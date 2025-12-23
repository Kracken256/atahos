use alloc::sync::Arc;
use log::info;
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::pmm::PhysicalMemoryAllocator;

static mut EARLY_ARENA: [u8; 4096] = [0; 4096];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    // if we're in a hosted environment, the Rust runtime may allocate before
    // main() is called, so we need to initialize the arena automatically
    ClaimOnOom::new(Span::from_array(
        core::ptr::addr_of!(EARLY_ARENA).cast_mut(),
    ))
})
.lock();

pub fn init(_pmm: Arc<spin::Mutex<PhysicalMemoryAllocator>>) {
    info!("Initializing virtual memory manager...");

    // TODO: Setup VMM using physical frames from PMM

    info!("Virtual memory manager initialized.");
}
