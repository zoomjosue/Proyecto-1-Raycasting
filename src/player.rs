use crate::map::{MapaJuego, TAMANO_CELDA};
use minifb::{Key, MouseMode, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::TAU;

/// Representa la posición, orientación y colisiones del jugador.
pub struct Jugador {
    pub posicion: Vec2,
    pub angulo: f32,
    pub radio: f32,
    ultimo_x_mouse: Option<f32>,
    ignorar_delta_mouse: bool,
}

impl Jugador {
    /// Crea un jugador en la posición inicial del mapa.
    pub fn nuevo(posicion: Vec2) -> Self {
        Self {
            posicion,
            angulo: 0.0,
            radio: 15.0,
            ultimo_x_mouse: None,
            ignorar_delta_mouse: false,
        }
    }

    /// Reinicia posición, orientación y estado del mouse.
    pub fn reiniciar(&mut self, posicion: Vec2) {
        self.posicion = posicion;
        self.angulo = 0.0;
        self.ultimo_x_mouse = None;
        self.ignorar_delta_mouse = false;
    }

    /// Devuelve el vector unitario hacia donde mira el jugador.
    pub fn direccion_frontal(&self) -> Vec2 {
        Vec2::new(self.angulo.cos(), self.angulo.sin())
    }

    /// Lee teclado y mouse, mueve al jugador y evita atravesar paredes.
    pub fn leer_entrada(&mut self, ventana: &Window, mapa: &MapaJuego, segundos: f32) {
        let velocidad_rotacion = 2.0 * segundos;
        if ventana.is_key_down(Key::A) {
            self.angulo -= velocidad_rotacion;
        }
        if ventana.is_key_down(Key::D) {
            self.angulo += velocidad_rotacion;
        }
        if let Some((x_mouse, _)) = ventana.get_mouse_pos(MouseMode::Clamp) {
            let ancho_ventana = ventana.get_size().0 as f32;
            let x_centro = ancho_ventana * 0.5;
            if self.ignorar_delta_mouse {
                self.ignorar_delta_mouse = false;
            } else if let Some(x_anterior) = self.ultimo_x_mouse {
                let delta_mouse = x_mouse - x_anterior;
                if delta_mouse.abs() < ancho_ventana * 0.5 {
                    self.angulo += delta_mouse * 0.004;
                }
            }
            self.ultimo_x_mouse = Some(x_mouse);

            let margen_borde = 80.0;
            if x_mouse <= margen_borde || x_mouse >= ancho_ventana - margen_borde {
                centrar_cursor(ventana);
                self.ultimo_x_mouse = Some(x_centro);
                self.ignorar_delta_mouse = true;
            }
        }
        let mut direccion_movimiento = 0.0;
        if ventana.is_key_down(Key::W) {
            direccion_movimiento += 1.0;
        }
        if ventana.is_key_down(Key::S) {
            direccion_movimiento -= 1.0;
        }
        if direccion_movimiento != 0.0 {
            self.mover_con_colision(
                self.direccion_frontal() * direccion_movimiento * 145.0 * segundos,
                mapa,
            );
        }
        self.angulo = self.angulo.rem_euclid(TAU);
    }

    /// Aplica un desplazamiento separado por ejes para deslizarse junto a paredes.
    pub fn mover_con_colision(&mut self, desplazamiento: Vec2, mapa: &MapaJuego) {
        let candidato_x = Vec2::new(self.posicion.x + desplazamiento.x, self.posicion.y);
        if !self.colisiona(candidato_x, mapa) {
            self.posicion.x = candidato_x.x;
        }
        let candidato_y = Vec2::new(self.posicion.x, self.posicion.y + desplazamiento.y);
        if !self.colisiona(candidato_y, mapa) {
            self.posicion.y = candidato_y.y;
        }
    }

    /// Comprueba las cuatro esquinas del radio del jugador contra el mapa.
    fn colisiona(&self, posicion: Vec2, mapa: &MapaJuego) -> bool {
        let radio = self.radio;
        let esquinas = [
            (posicion.x - radio, posicion.y - radio),
            (posicion.x + radio, posicion.y - radio),
            (posicion.x - radio, posicion.y + radio),
            (posicion.x + radio, posicion.y + radio),
        ];
        esquinas
            .iter()
            .any(|(x, y)| mapa.es_pared((*x / TAMANO_CELDA) as i32, (*y / TAMANO_CELDA) as i32))
    }
}

/// Centra el cursor usando la posición real de la ventana en Windows.
pub fn centrar_cursor(ventana: &Window) {
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::{
            shared::windef::{HWND, RECT},
            um::winuser::{GetWindowRect, SetCursorPos},
        };

        let identificador = ventana.get_window_handle() as HWND;
        if identificador.is_null() {
            return;
        }
        let mut rectangulo = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(identificador, &mut rectangulo) != 0 {
            let (ancho, alto) = ventana.get_size();
            SetCursorPos(
                rectangulo.left + (ancho / 2) as i32,
                rectangulo.top + (alto / 2) as i32,
            );
        }
    }
}
