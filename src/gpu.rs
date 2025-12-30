use rand::Rng;

#[derive(Debug)]
pub struct GPU {
    mode: GpuGraphicsMode,
    ram: crate::memory::Memory,
    grid_buffer: Box<[[u32; 96]; 35]>,
    frame_buffer: Box<[[u32; 1920]; 1080]>
}

impl GPU {
    pub fn init() -> Self {
        let mut gpu = Self {
            mode: GpuGraphicsMode::Text,
            ram: crate::memory::Memory::empty(0x4000_0000),
            grid_buffer: Box::new([[b' '.into(); 96]; 35]),
            frame_buffer: unsafe { Box::<[[u32; 1920]; 1080]>::new_uninit().assume_init()}
        };
        for y in 0..gpu.frame_buffer.len() {
            for x in 0..gpu.frame_buffer[0].len() {
                gpu.frame_buffer[y][x] = (rand::rng().random_range(0..=255) as u32) << 24
                    | (rand::rng().random_range(0..=255) as u32) << 16
                    | (rand::rng().random_range(0..=255) as u32) << 8
                    | rand::rng().random_range(0..=255) as u32;
            }
        }
        for y in 0..gpu.grid_buffer.len() {
            for x in 0..gpu.grid_buffer[0].len() {
                gpu.grid_buffer[y][x] = ((rand::rng().sample(rand::distr::Alphanumeric) as u32) << 24)
                    | (rand::rng().random_range(0..=255) as u32) << 16
                    | (rand::rng().random_range(0..=255) as u32) << 8
                    | rand::rng().random_range(0..=255) as u32;
            }
        }
        info!("Created GPU");
        return gpu;
    }

    pub fn update(&mut self) -> Result<(), GpuError> {
        self.render();
        Ok(())
    }

    pub fn render(&mut self) {
        match self.mode {
            GpuGraphicsMode::Text => {
                for (y, row) in self.grid_buffer.iter().enumerate() {
                    for (x, word) in row.iter().enumerate() {
                        let (character, color) = decode_char_u32(*word);
                        macroquad::text::draw_text(
                            &character.to_string(),
                            (x * 15) as f32,
                            (y * 23 + 19) as f32,
                            29.0,
                            color,
                        );
                    }
                }
            }
            GpuGraphicsMode::Full => {
                for (y, row) in self.frame_buffer.iter().enumerate() {
                    for (x, word) in row.iter().enumerate() {
                        macroquad::shapes::draw_rectangle(x as f32, y as f32, 1.0, 1.0, decode_rgba_u32(*word));
                    }
                }
            }
        }
    }

    pub fn handle_errors(&self, error: Result<(), GpuError>) {}

    pub async fn run(mut self) {
        loop {
            let result = self.update();
            macroquad::window::next_frame().await;

            if result.is_err() {
                self.handle_errors(result);
            }
        }
    }
}

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Rusty-vm".to_string(),
        window_width: (1920.0 * 0.75) as i32,
        window_height: (1080.0 * 0.75) as i32,
        window_resizable: false,
        fullscreen: false,
        sample_count: 8,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
pub async fn main() {
    GPU::init().run().await;
}

pub fn decode_char_u32(char_word: u32) -> (char, macroquad::color::Color) {
    let char_byte = (char_word >> 24) & 0xFF;
    let red_byte = ((char_word >> 16) & 0xFF) as u8;
    let green_byte = ((char_word >> 8) & 0xFF) as u8;
    let blue_byte = (char_word & 0xFF) as u8;

    let char = char::from_u32(char_byte).unwrap();
    let color = macroquad::color::Color::from_rgba(red_byte, green_byte, blue_byte, 255);
    return (char, color);
}

pub fn decode_rgba_u32(char_word: u32) -> macroquad::color::Color {
    let red_byte = ((char_word >> 24) & 0xFF) as u8;
    let green_byte = ((char_word >> 16) & 0xFF) as u8;
    let blue_byte = ((char_word >> 8) & 0xFF) as u8;
    let alpha_byte = (char_word & 0xFF) as u8;

    let color = macroquad::color::Color::from_rgba(red_byte, green_byte, blue_byte, alpha_byte);
    return color;
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
