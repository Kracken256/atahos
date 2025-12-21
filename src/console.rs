pub struct Console {
    vga_base: *mut [u8; 4000],
    row: usize,
    col: usize,
}

impl Console {
    pub const fn new() -> Self {
        Console {
            vga_base: 0xb8000 as *mut [u8; 4000],
            row: 0,
            col: 0,
        }
    }

    fn scroll(&mut self) {
        if self.row < 25 {
            return;
        }

        let buf = unsafe { &mut *self.vga_base };

        for r in 1..25 {
            for c in 0..80 {
                buf[(r - 1) * 80 * 2 + c * 2] = buf[r * 80 * 2 + c * 2];
                buf[(r - 1) * 80 * 2 + c * 2 + 1] = buf[r * 80 * 2 + c * 2 + 1];
            }
        }

        self.row = 24;
    }

    pub fn write_str(&mut self, s: &str) {
        let buf = unsafe { &mut *self.vga_base };

        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    self.col = 0;
                    self.row += 1;

                    self.scroll();
                }

                byte => {
                    buf[self.row * 80 * 2 + self.col * 2] = byte;
                    buf[self.row * 80 * 2 + self.col * 2 + 1] = 0x0F; // White on black

                    self.col += 1;
                    if self.col >= 80 {
                        self.col = 0;
                        self.row += 1;
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        let buf = unsafe { &mut *self.vga_base };

        for i in 0..4000 {
            buf[i] = 0;
        }

        self.row = 0;
        self.col = 0;
    }
}
