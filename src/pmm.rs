const PAGE_SIZE: usize = 4096;

pub type Frame = [u8; PAGE_SIZE];

#[repr(C)]
struct FreeListFrame {
    next: *mut FreeListFrame,
}

pub struct PhysicalMemoryAllocator {
    free_list: *mut FreeListFrame,
}

impl PhysicalMemoryAllocator {
    pub const fn new() -> Self {
        Self {
            free_list: core::ptr::null_mut(),
        }
    }

    pub fn allocate_frame(&mut self) -> Option<*mut Frame> {
        if self.free_list.is_null() {
            return None;
        }

        let frame = self.free_list;
        self.free_list = unsafe { (*frame).next };
        Some(frame as *mut Frame)
    }

    pub unsafe fn deallocate_frame(&mut self, frame: *mut Frame) {
        let frame_ptr = frame as *mut FreeListFrame;
        unsafe { (*frame_ptr).next = self.free_list }
        self.free_list = frame_ptr;
    }

    pub unsafe fn add_region(&mut self, base: *mut u8, size: usize) {
        let mut current = base as usize;
        let end = current + size;

        while current + PAGE_SIZE <= end {
            unsafe { self.deallocate_frame(current as *mut Frame) }
            current += PAGE_SIZE;
        }
    }

    pub fn available_frames(&self) -> usize {
        let mut count = 0;
        let mut current = self.free_list;

        while !current.is_null() {
            count += 1;
            unsafe {
                current = (*current).next;
            }
        }

        count
    }

    pub fn total_memory(&self) -> usize {
        self.available_frames() * PAGE_SIZE
    }
}

unsafe impl Send for PhysicalMemoryAllocator {}
unsafe impl Sync for PhysicalMemoryAllocator {}

pub static PMM: spin::Mutex<PhysicalMemoryAllocator> =
    spin::Mutex::new(PhysicalMemoryAllocator::new());
