use crate::{
    framebuffer::BufferPantalla,
    map::{MapaJuego, TAMANO_CELDA},
    player::Jugador,
    texture::AtlasTexturas,
};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

pub const CAMPO_VISION: f32 = PI / 3.0;

/// Renderiza cielo, suelo y paredes, y devuelve el buffer de profundidad.
pub fn renderizar_mundo(
    buffer: &mut BufferPantalla,
    mapa: &MapaJuego,
    jugador: &Jugador,
    texturas: &AtlasTexturas,
) -> Vec<f32> {
    let horizonte = buffer.alto as i32 / 2;
    for fila in 0..horizonte {
        let intensidad = (18 + fila * 10 / horizonte) as u32;
        buffer.linea_horizontal(
            fila,
            0,
            buffer.ancho as i32 - 1,
            (intensidad << 16) | (intensidad << 8) | (intensidad + 8),
        );
    }
    for fila in horizonte..buffer.alto as i32 {
        let intensidad = (28 + (fila - horizonte) * 22 / horizonte) as u32;
        buffer.linea_horizontal(
            fila,
            0,
            buffer.ancho as i32 - 1,
            (intensidad << 16) | ((intensidad / 2) << 8) | (intensidad / 2),
        );
    }
    let direccion = jugador.direccion_frontal();
    let plano = Vec2::new(-direccion.y, direccion.x) * (CAMPO_VISION / 2.0).tan();
    let mut buffer_profundidad = vec![f32::MAX; buffer.ancho];
    for (x_pantalla, profundidad) in buffer_profundidad.iter_mut().enumerate() {
        let x_camara = 2.0 * x_pantalla as f32 / buffer.ancho as f32 - 1.0;
        let rayo = direccion + plano * x_camara;
        let impacto = lanzar_rayo(mapa, jugador.posicion, rayo);
        let distancia_corregida =
            (impacto.distancia * (rayo.x * direccion.x + rayo.y * direccion.y)).max(0.1);
        *profundidad = distancia_corregida;
        let alto_pared = (TAMANO_CELDA * buffer.alto as f32 / distancia_corregida) as i32;
        // Se usa la proyección completa para evitar estirar la textura al acercarse.
        let parte_superior_proyectada = buffer.alto as i32 / 2 - alto_pared / 2;
        let parte_inferior_proyectada = buffer.alto as i32 / 2 + alto_pared / 2;
        let parte_superior = parte_superior_proyectada.max(0);
        let parte_inferior = parte_inferior_proyectada.min(buffer.alto as i32 - 1);
        let luz = (1.0 / (1.0 + distancia_corregida * 0.005)).max(0.25);
        let luz_lateral = if impacto.lado == 1 { 0.72 } else { 1.0 };
        for fila in parte_superior..=parte_inferior {
            let u_pared = impacto.x_pared / TAMANO_CELDA;
            let v_pared = if alto_pared > 0 {
                (fila - parte_superior_proyectada) as f32 / alto_pared as f32
            } else {
                0.0
            };
            let color_textura = if impacto.baldosa == 'D' {
                texturas.muestrear_puerta(u_pared, v_pared)
            } else {
                texturas.muestrear(impacto.baldosa, u_pared, v_pared)
            };
            buffer.establecer_pixel(
                x_pantalla as i32,
                fila,
                oscurecer(color_textura, luz * luz_lateral),
            );
        }
    }
    buffer_profundidad
}

/// Contiene los datos de una pared alcanzada por un rayo.
pub struct ImpactoRayo {
    pub distancia: f32,
    pub baldosa: char,
    pub lado: i32,
    pub x_pared: f32,
}

/// Recorre el mapa con DDA hasta encontrar una pared.
pub fn lanzar_rayo(mapa: &MapaJuego, origen: Vec2, rayo: Vec2) -> ImpactoRayo {
    let mut mapa_x = (origen.x / TAMANO_CELDA) as i32;
    let mut mapa_y = (origen.y / TAMANO_CELDA) as i32;
    let delta_x = if rayo.x.abs() < 0.0001 {
        1e30
    } else {
        (TAMANO_CELDA / rayo.x).abs()
    };
    let delta_y = if rayo.y.abs() < 0.0001 {
        1e30
    } else {
        (TAMANO_CELDA / rayo.y).abs()
    };
    let (paso_x, mut lado_x) = if rayo.x < 0.0 {
        (-1, (origen.x / TAMANO_CELDA - mapa_x as f32) * delta_x)
    } else {
        (1, (mapa_x as f32 + 1.0 - origen.x / TAMANO_CELDA) * delta_x)
    };
    let (paso_y, mut lado_y) = if rayo.y < 0.0 {
        (-1, (origen.y / TAMANO_CELDA - mapa_y as f32) * delta_y)
    } else {
        (1, (mapa_y as f32 + 1.0 - origen.y / TAMANO_CELDA) * delta_y)
    };
    for _ in 0..256 {
        let lado = if lado_x < lado_y {
            lado_x += delta_x;
            mapa_x += paso_x;
            0
        } else {
            lado_y += delta_y;
            mapa_y += paso_y;
            1
        };
        if mapa.es_pared(mapa_x, mapa_y) {
            let distancia = if lado == 0 {
                lado_x - delta_x
            } else {
                lado_y - delta_y
            };
            let posicion_impacto = origen + rayo * distancia;
            let x_pared = if lado == 0 {
                posicion_impacto.y
            } else {
                posicion_impacto.x
            };
            return ImpactoRayo {
                distancia: distancia.max(0.1),
                baldosa: mapa.baldosa_en(mapa_x, mapa_y),
                lado,
                x_pared,
            };
        }
    }
    ImpactoRayo {
        distancia: 1.0,
        baldosa: '#',
        lado: 0,
        x_pared: 0.0,
    }
}

/// Reduce la intensidad de un color sin modificar sus canales de forma desigual.
fn oscurecer(color: u32, intensidad: f32) -> u32 {
    let rojo = (((color >> 16) & 255) as f32 * intensidad).clamp(0.0, 255.0) as u32;
    let verde = (((color >> 8) & 255) as f32 * intensidad).clamp(0.0, 255.0) as u32;
    let azul = ((color & 255) as f32 * intensidad).clamp(0.0, 255.0) as u32;
    (rojo << 16) | (verde << 8) | azul
}
