pub struct AtlasTexturas {
    pixeles: Vec<u8>,
    ancho: u32,
    alto: u32,
    pixeles_puerta: Vec<u8>,
    ancho_puerta: u32,
    alto_puerta: u32,
}

impl AtlasTexturas {
    /// Carga el atlas de paredes y la textura independiente de la puerta.
    pub fn cargar() -> Self {
        let imagen = image::load_from_memory(include_bytes!("../assets/haunted_wall_atlas.png"))
            .expect("No se pudo cargar assets/haunted_wall_atlas.png")
            .to_rgba8();
        let (ancho, alto) = imagen.dimensions();
        let puerta = image::load_from_memory(include_bytes!("../assets/broken_door.png"))
            .expect("No se pudo cargar assets/broken_door.png")
            .to_rgba8();
        let (ancho_puerta, alto_puerta) = puerta.dimensions();
        Self {
            pixeles: imagen.into_raw(),
            ancho,
            alto,
            pixeles_puerta: puerta.into_raw(),
            ancho_puerta,
            alto_puerta,
        }
    }

    /// Muestrea una textura del atlas según el tipo de pared y sus coordenadas.
    pub fn muestrear(&self, baldosa: char, u_pared: f32, v_pared: f32) -> u32 {
        let (coordenada_atlas_x, coordenada_atlas_y) = match baldosa {
            'R' => (1.0, 0.0), // ladrillo rojo
            'I' => (0.0, 1.0), // placas de hierro
            'W' => (1.0, 1.0), // madera húmeda
            _ => (0.0, 0.0),   // piedra con musgo
        };
        let u = coordenada_atlas_x * 0.5 + u_pared.rem_euclid(1.0) * 0.49 + 0.005;
        let v = coordenada_atlas_y * 0.5 + v_pared.rem_euclid(1.0) * 0.49 + 0.005;
        let x = ((u * self.ancho as f32) as u32).min(self.ancho - 1);
        let y = ((v * self.alto as f32) as u32).min(self.alto - 1);
        let desplazamiento = ((y * self.ancho + x) * 4) as usize;
        ((self.pixeles[desplazamiento] as u32) << 16)
            | ((self.pixeles[desplazamiento + 1] as u32) << 8)
            | self.pixeles[desplazamiento + 2] as u32
    }

    /// Muestrea la textura de la puerta usando coordenadas normalizadas.
    pub fn muestrear_puerta(&self, u_pared: f32, v_pared: f32) -> u32 {
        let x = ((u_pared.rem_euclid(1.0) * self.ancho_puerta as f32) as u32)
            .min(self.ancho_puerta - 1);
        let y =
            ((v_pared.clamp(0.0, 1.0) * self.alto_puerta as f32) as u32).min(self.alto_puerta - 1);
        let desplazamiento = ((y * self.ancho_puerta + x) * 4) as usize;
        ((self.pixeles_puerta[desplazamiento] as u32) << 16)
            | ((self.pixeles_puerta[desplazamiento + 1] as u32) << 8)
            | self.pixeles_puerta[desplazamiento + 2] as u32
    }
}

#[cfg(test)]
mod tests {
    use super::AtlasTexturas;

    #[test]
    fn atlas_del_castillo_esta_disponible() {
        let atlas = AtlasTexturas::cargar();
        assert!(atlas.ancho > 0 && atlas.alto > 0);
        assert_ne!(atlas.muestrear('#', 0.25, 0.25), 0);
        assert_ne!(atlas.muestrear_puerta(0.5, 0.5), 0);
    }
}
