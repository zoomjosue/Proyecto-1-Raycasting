use crate::{framebuffer::BufferPantalla, ProyeccionSprite};
use std::collections::VecDeque;

/// Agrupa las imágenes 2D que aparecen dentro del mundo.
pub struct RecursosSprites {
    llaves: ImagenSprite,
}

struct ImagenSprite {
    pixeles: Vec<u8>,
    ancho: u32,
    alto: u32,
}

impl RecursosSprites {
    /// Carga el atlas de las tres llaves desde los recursos incluidos.
    pub fn cargar() -> Self {
        Self {
            llaves: ImagenSprite::desde_bytes(include_bytes!("../assets/colored_keys.png")),
        }
    }

    /// Dibuja la llave correspondiente a su índice en el atlas.
    pub fn dibujar_llave(
        &self,
        buffer: &mut BufferPantalla,
        proyeccion: ProyeccionSprite,
        indice_llave: usize,
    ) {
        let ancho_llave = self.llaves.ancho / 3;
        let inicio = ancho_llave * indice_llave.min(2) as u32;
        self.llaves.dibujar(
            buffer,
            proyeccion.centro_x,
            proyeccion.superior,
            proyeccion.inferior,
            inicio,
            inicio + ancho_llave,
        );
    }
}

impl ImagenSprite {
    /// Convierte bytes PNG a píxeles y elimina el fondo de cuadricula.
    fn desde_bytes(bytes: &[u8]) -> Self {
        let imagen = image::load_from_memory(bytes)
            .expect("No se pudo cargar un sprite PNG")
            .to_rgba8();
        let (ancho, alto) = imagen.dimensions();
        let mut pixeles = imagen.into_raw();
        eliminar_fondo_cuadricula(&mut pixeles, ancho, alto);
        Self {
            pixeles,
            ancho,
            alto,
        }
    }

    /// Proyecta una sección del atlas dentro del búfer de pantalla.
    fn dibujar(
        &self,
        buffer: &mut BufferPantalla,
        centro_x: i32,
        superior: i32,
        inferior: i32,
        fuente_x_inicial: u32,
        fuente_x_final: u32,
    ) {
        let alto_sprite = (inferior - superior).max(1);
        let ancho_fuente = (fuente_x_final - fuente_x_inicial).max(1);
        let ancho_sprite = (alto_sprite as f32 * ancho_fuente as f32 / self.alto as f32) as i32;
        let izquierda = centro_x - ancho_sprite / 2;
        for y_pantalla in superior.max(0)..=inferior.min(buffer.alto as i32 - 1) {
            let y_fuente =
                (((y_pantalla - superior) as f32 / alto_sprite as f32) * self.alto as f32) as u32;
            for x_pantalla in
                izquierda.max(0)..=(izquierda + ancho_sprite).min(buffer.ancho as i32 - 1)
            {
                let x_fuente = (fuente_x_inicial as f32
                    + ((x_pantalla - izquierda) as f32 / ancho_sprite.max(1) as f32)
                        * ancho_fuente as f32) as u32;
                let x_fuente = x_fuente.min(self.ancho - 1);
                let y_fuente = y_fuente.min(self.alto - 1);
                let desplazamiento = ((y_fuente * self.ancho + x_fuente) * 4) as usize;
                let alpha = self.pixeles[desplazamiento + 3];
                if alpha < 16 {
                    continue;
                }
                let color = ((self.pixeles[desplazamiento] as u32) << 16)
                    | ((self.pixeles[desplazamiento + 1] as u32) << 8)
                    | self.pixeles[desplazamiento + 2] as u32;
                buffer.establecer_pixel(x_pantalla, y_pantalla, color);
            }
        }
    }
}

/// Convierte el fondo gris de los bordes del atlas en transparencia.
fn eliminar_fondo_cuadricula(pixeles: &mut [u8], ancho: u32, alto: u32) {
    let mut visitados = vec![false; (ancho * alto) as usize];
    let mut cola = VecDeque::new();
    for x in 0..ancho {
        cola.push_back((x, 0));
        cola.push_back((x, alto - 1));
    }
    for y in 0..alto {
        cola.push_back((0, y));
        cola.push_back((ancho - 1, y));
    }
    while let Some((x, y)) = cola.pop_front() {
        let indice = (y * ancho + x) as usize;
        if visitados[indice] {
            continue;
        }
        visitados[indice] = true;
        let desplazamiento = indice * 4;
        let rojo = pixeles[desplazamiento];
        let verde = pixeles[desplazamiento + 1];
        let azul = pixeles[desplazamiento + 2];
        let neutro = rojo.max(verde).max(azul) - rojo.min(verde).min(azul) < 12;
        if rojo < 180 || !neutro {
            continue;
        }
        pixeles[desplazamiento + 3] = 0;
        if x > 0 {
            cola.push_back((x - 1, y));
        }
        if x + 1 < ancho {
            cola.push_back((x + 1, y));
        }
        if y > 0 {
            cola.push_back((x, y - 1));
        }
        if y + 1 < alto {
            cola.push_back((x, y + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecursosSprites;

    #[test]
    fn sprites_cargan_con_transparencia() {
        let recursos = RecursosSprites::cargar();
        assert!(recursos
            .llaves
            .pixeles
            .chunks_exact(4)
            .any(|pixel| pixel[3] < 255));
    }
}
