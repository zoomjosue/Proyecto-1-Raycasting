pub struct TextureAtlas {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    door_pixels: Vec<u8>,
    door_width: u32,
    door_height: u32,
}

impl TextureAtlas {
    pub fn load() -> Self {
        let image = image::load_from_memory(include_bytes!("../assets/haunted_wall_atlas.png"))
            .expect("No se pudo cargar assets/haunted_wall_atlas.png")
            .to_rgba8();
        let (width, height) = image.dimensions();
        let door = image::load_from_memory(include_bytes!("../assets/broken_door.png"))
            .expect("No se pudo cargar assets/broken_door.png")
            .to_rgba8();
        let (door_width, door_height) = door.dimensions();
        Self {
            pixels: image.into_raw(),
            width,
            height,
            door_pixels: door.into_raw(),
            door_width,
            door_height,
        }
    }

    pub fn sample(&self, tile: char, wall_u: f32, wall_v: f32) -> u32 {
        let (atlas_x, atlas_y) = match tile {
            'R' => (1.0, 0.0), // ladrillo rojo
            'I' => (0.0, 1.0), // placas de hierro
            'W' => (1.0, 1.0), // madera húmeda
            _ => (0.0, 0.0),   // piedra con musgo
        };
        let u = atlas_x * 0.5 + wall_u.rem_euclid(1.0) * 0.49 + 0.005;
        let v = atlas_y * 0.5 + wall_v.rem_euclid(1.0) * 0.49 + 0.005;
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        let offset = ((y * self.width + x) * 4) as usize;
        ((self.pixels[offset] as u32) << 16)
            | ((self.pixels[offset + 1] as u32) << 8)
            | self.pixels[offset + 2] as u32
    }

    pub fn sample_door(&self, wall_u: f32, wall_v: f32) -> u32 {
        let x = ((wall_u.rem_euclid(1.0) * self.door_width as f32) as u32).min(self.door_width - 1);
        let y =
            ((wall_v.clamp(0.0, 1.0) * self.door_height as f32) as u32).min(self.door_height - 1);
        let offset = ((y * self.door_width + x) * 4) as usize;
        ((self.door_pixels[offset] as u32) << 16)
            | ((self.door_pixels[offset + 1] as u32) << 8)
            | self.door_pixels[offset + 2] as u32
    }
}

#[cfg(test)]
mod tests {
    use super::TextureAtlas;

    #[test]
    fn haunted_atlas_is_available() {
        let atlas = TextureAtlas::load();
        assert!(atlas.width > 0 && atlas.height > 0);
        assert_ne!(atlas.sample('#', 0.25, 0.25), 0);
        assert_ne!(atlas.sample_door(0.5, 0.5), 0);
    }
}
