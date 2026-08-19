pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        for py in y.max(0)..(y + height).min(self.height as i32) {
            for px in x.max(0)..(x + width).min(self.width as i32) {
                self.set_pixel(px, py, color);
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        for px in x..x + width {
            self.set_pixel(px, y, color);
            self.set_pixel(px, y + height - 1, color);
        }
        for py in y..y + height {
            self.set_pixel(x, py, color);
            self.set_pixel(x + width - 1, py, color);
        }
    }

    pub fn horizontal_line(&mut self, y: i32, x0: i32, x1: i32, color: u32) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set_pixel(x, y, color);
        }
    }
}
