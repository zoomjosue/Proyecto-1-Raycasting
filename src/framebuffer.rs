/// Búfer de píxeles que representa la imagen completa del juego.
pub struct BufferPantalla {
    pub ancho: usize,
    pub alto: usize,
    pub pixeles: Vec<u32>,
}

impl BufferPantalla {
    /// Crea un búfer vacío con las dimensiones indicadas.
    pub fn nuevo(ancho: usize, alto: usize) -> Self {
        Self {
            ancho,
            alto,
            pixeles: vec![0; ancho * alto],
        }
    }

    /// Rellena todos los píxeles con un color RGB.
    pub fn limpiar(&mut self, color: u32) {
        self.pixeles.fill(color);
    }

    /// Establece un píxel si se encuentra dentro de la pantalla.
    pub fn establecer_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.ancho && (y as usize) < self.alto {
            self.pixeles[y as usize * self.ancho + x as usize] = color;
        }
    }

    /// Dibuja un rectángulo relleno, recortándolo a la pantalla.
    pub fn rellenar_rectangulo(&mut self, x: i32, y: i32, ancho: i32, alto: i32, color: u32) {
        for py in y.max(0)..(y + alto).min(self.alto as i32) {
            for px in x.max(0)..(x + ancho).min(self.ancho as i32) {
                self.establecer_pixel(px, py, color);
            }
        }
    }

    /// Dibuja únicamente el borde de un rectángulo.
    pub fn dibujar_rectangulo(&mut self, x: i32, y: i32, ancho: i32, alto: i32, color: u32) {
        for px in x..x + ancho {
            self.establecer_pixel(px, y, color);
            self.establecer_pixel(px, y + alto - 1, color);
        }
        for py in y..y + alto {
            self.establecer_pixel(x, py, color);
            self.establecer_pixel(x + ancho - 1, py, color);
        }
    }

    /// Dibuja una línea horizontal entre dos coordenadas.
    pub fn linea_horizontal(&mut self, y: i32, x_inicial: i32, x_final: i32, color: u32) {
        for x in x_inicial.min(x_final)..=x_inicial.max(x_final) {
            self.establecer_pixel(x, y, color);
        }
    }
}
