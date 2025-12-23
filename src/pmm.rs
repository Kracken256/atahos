const PAGE_SIZE: usize = 4096;

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
