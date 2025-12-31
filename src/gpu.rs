use rand::Rng;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_HEIGHT: usize = 720;

#[derive(Debug)]
pub struct GPU {
    pub mode: GpuGraphicsMode,
    pub ram: crate::memory::Memory,
    pub frame_buffer: Box<[u32; 1280 * 720]>,
    pub registers: [u32; 10], // fb_pointer, pixeldata, update-enable
    pub map_base: u32,
}

impl GPU {
    pub fn init(map_base: u32) -> Self {
        let gpu = Self {
            mode: GpuGraphicsMode::Full,
            ram: crate::memory::Memory::empty(0x4000_0000),
            frame_buffer: unsafe { Box::<[u32; 1280 * 720]>::new_uninit().assume_init() },
            registers: [0u32; 10],
            map_base,
        };
        info!("Created GPU");
        return gpu;
    }

    pub fn update(&mut self) -> Result<(), GpuError> {
        self.render();
        Ok(())
    }

    pub fn render(&mut self) {
        if self.registers[2] == 0 {
            self.show_life();
        } else if self.registers[2] >= 1 {
            self.frame_buffer[self.registers[0] as usize] = self.registers[1];
        }
    }

    pub fn handle_errors(&self, error: Result<(), GpuError>) {}

    pub fn run(mut self) {
        loop {
            let result = self.update();

            if result.is_err() {
                self.handle_errors(result);
            }
        }
    }

    pub fn draw_letter(&mut self, char: char, pos_x: u32, pos_y: u32) {}

    pub fn blit_pixel(&mut self, pos_x: usize, pos_y: usize, color: Color) {
        self.frame_buffer[pos_y * SCREEN_WIDTH + pos_x] = color.to_argb_u32()
    }

    pub fn show_life(&mut self) {
        for pixel in self.frame_buffer.iter_mut() {
            *pixel = Color::from_u32(rand::rng().random()).to_argb_u32();
        }
        let size = 400; // Triangle side length in pixels (adjust to fit your window)
        let cx: i32 = (SCREEN_WIDTH / 2) as i32;  // Center X
        let cy: i32 = (SCREEN_HEIGHT / 2 + 50) as i32; // Center Y (shift down a bit for visibility)

        let h = (size as f32 * (3f32.sqrt() / 2.0)) as i32; // Height of equilateral triangle

        // Triangle vertices (screen coordinates)
        let vx0 = cx - size / 2; // Red (bottom-left)
        let vy0 = cy + h / 3;
        let vx1 = cx;           // Green (top)
        let vy1 = cy - (2 * h / 3);
        let vx2 = cx + size / 2; // Blue (bottom-right)
        let vy2 = cy + h / 3;

        // Bounding box for faster looping
        let min_x = vx0.min(vx1).min(vx2);
        let max_x = vx0.max(vx1).max(vx2);
        let min_y = vy0.min(vy1).min(vy2);
        let max_y = vy0.max(vy1).max(vy2);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Barycentric coordinates
                let denom = (vy1 - vy2) as f32 * (vx0 - vx2) as f32 + (vx2 - vx1) as f32 * (vy0 - vy2) as f32;
                let a = ((vy1 - vy2) as f32 * (x - vx2) as f32 + (vx2 - vx1) as f32 * (y - vy2) as f32) / denom;
                let b = ((vy2 - vy0) as f32 * (x - vx2) as f32 + (vx0 - vx2) as f32 * (y - vy2) as f32) / denom;
                let c = 1.0 - a - b;

                // Inside triangle if all coords >= 0 (and <=1 implicitly)
                if a >= 0.0 && b >= 0.0 && c >= 0.0 {
                    // Scale to 0-255 (u8) and pack into u32 color (0xRRGGBB)
                    let r = (a * 255.0) as u8;
                    let g = (b * 255.0) as u8;
                    let bl = (c * 255.0) as u8; // 'bl' to avoid keyword conflict

                    let color: u32 = ((r as u32) << 16) | ((g as u32) << 8) | (bl as u32);

                    self.blit_pixel(x as usize, y as usize, Color::from_u32(color));
                }
            }
        }
        for y in 0..255 {
            for x in 0..255 {
                self.blit_pixel(x, y, Color::from_argb(255, x as u8, x as u8, x as u8));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

impl crate::mmio::AddressSpace for GPU {
    fn read8(&self, addr_offset: u32) -> u8 {
        0
    }
    fn write8(&mut self, addr_offset: u32, value: u8) {
        if addr_offset >= 0x10 {
            error!("Address offset out of bounds!");
            return;
        }
        self.registers[addr_offset as usize] = value as u32;
        info!("Received value {} at address {}", value, addr_offset)
    }
    fn write32(&mut self, addr_offset: u32, value: u32) {
        if addr_offset >= 0x10 {
            error!("Address offset out of bounds!");
            return;
        }

        self.registers[addr_offset as usize] = value;
        info!("Received value {} at address {}", value, self.map_base - addr_offset)
    }
}

pub struct Coordinates {
    x: usize,
    y: usize,
}

impl Coordinates {
    pub fn from_index(idx: usize, res_width: usize, res_height: usize) -> Self {
        Self {
            x: idx % res_width,
            y: idx / res_width
        }
    }
}

pub fn decode_char_u32(char_word: u32) -> (char, Color) {
    let char_byte = (char_word >> 24) & 0xFF;
    let red_byte = ((char_word >> 16) & 0xFF) as u8;
    let green_byte = ((char_word >> 8) & 0xFF) as u8;
    let blue_byte = (char_word & 0xFF) as u8;

    let char = char::from_u32(char_byte).unwrap();
    let color = Color::from_argb(255, red_byte, green_byte, blue_byte);
    return (char, color);
}

pub fn decode_rgba_u32(char_word: u32) -> Color {
    let red_byte = ((char_word >> 24) & 0xFF) as u8;
    let green_byte = ((char_word >> 16) & 0xFF) as u8;
    let blue_byte = ((char_word >> 8) & 0xFF) as u8;
    let alpha_byte = (char_word & 0xFF) as u8;

    let color = Color::from_argb(alpha_byte, red_byte, green_byte, blue_byte);
    return color;
}

pub struct Color {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn from_u32(color: u32) -> Self {
        Self {
            a: ((color >> 24) & 0xFF) as u8,
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }

    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    pub fn to_argb_u32(&self) -> u32 {
        return (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32;
    }
}

#[derive(Debug)]
pub enum GpuGraphicsMode {
    Text,
    Full,
}

#[derive(Debug, Display)]
pub enum GpuError {
    Error,
}
