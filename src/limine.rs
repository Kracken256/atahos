use limine::{
    BaseRevision,
    request::{FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker},
};
use log::info;

#[used]
pub static KERNEL_BASE: usize = 0xffffffff80000000;

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(0);

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

pub fn validate_version() {
    if !BASE_REVISION.is_supported() {
        panic!("Limine bootloader version not supported");
    } else {
        info!(
            "AtahOS Kernel Initialized - Limine Revision: {}",
            BASE_REVISION.loaded_revision().unwrap()
        )
    }
}
