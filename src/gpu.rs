use rand::Rng;

#[derive(Debug)]
pub struct GPU {
    pub mode: GpuGraphicsMode,
    pub ram: crate::memory::Memory,
    pub frame_buffer: Box<[u32; 1920 * 1080]>,
}

impl GPU {
    pub fn init() -> Self {
        let gpu = Self {
            mode: GpuGraphicsMode::Full,
            ram: crate::memory::Memory::empty(0x4000_0000),
            frame_buffer: unsafe { Box::<[u32; 1920 * 1080]>::new_uninit().assume_init() },
        };
        info!("Created GPU");
        return gpu;
    }

    pub fn update(&mut self) -> Result<(), GpuError> {
        self.render();
        Ok(())
    }

    pub fn render(&mut self) {
        for pixel in self.frame_buffer.iter_mut() {
            *pixel = rand::rng().random()
        }
        match self.mode {
            GpuGraphicsMode::Text => {}
            GpuGraphicsMode::Full => {}
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
}

pub fn decode_char_u32(char_word: u32) -> (char, Color) {
    let char_byte = (char_word >> 24) & 0xFF;
    let red_byte = ((char_word >> 16) & 0xFF) as u8;
    let green_byte = ((char_word >> 8) & 0xFF) as u8;
    let blue_byte = (char_word & 0xFF) as u8;

    let char = char::from_u32(char_byte).unwrap();
    let color = Color::from_rgba(red_byte, green_byte, blue_byte, 255);
    return (char, color);
}

pub fn decode_rgba_u32(char_word: u32) -> Color {
    let red_byte = ((char_word >> 24) & 0xFF) as u8;
    let green_byte = ((char_word >> 16) & 0xFF) as u8;
    let blue_byte = ((char_word >> 8) & 0xFF) as u8;
    let alpha_byte = (char_word & 0xFF) as u8;

    let color = Color::from_rgba(red_byte, green_byte, blue_byte, alpha_byte);
    return color;
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
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
