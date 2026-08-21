mod audio;
mod font;
mod framebuffer;
mod map;
mod player;
mod raycaster;
mod sprites;
mod texture;

use audio::GestorAudio;
use font::{ancho_texto, dibujar_texto};
use framebuffer::BufferPantalla;
use map::{MapaJuego, TAMANO_CELDA};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::Vec2;
use player::{centrar_cursor, Jugador};
use raycaster::renderizar_mundo;
use sprites::RecursosSprites;
use std::time::{Duration, Instant};
use texture::AtlasTexturas;

const ANCHO: usize = 1920;
const ALTO: usize = 1080;
const DURACION_OBJETIVO: Duration = Duration::from_millis(33);

#[derive(PartialEq)]
enum Pantalla {
    Bienvenida,
    Jugando,
    Exito,
}

struct Juego {
    pantalla: Pantalla,
    mapa: MapaJuego,
    jugador: Jugador,
    buffer: BufferPantalla,
    mostrar_mapa: bool,
    recogidas: Vec<bool>,
    tiempo_animacion: f32,
    cuadros_por_segundo: f32,
    audio: Option<GestorAudio>,
    texturas: AtlasTexturas,
    sprites: RecursosSprites,
    e_presionada: bool,
    r_presionada: bool,
}

impl Juego {
    /// Construye el juego cargando mapa, texturas, sprites y audio.
    fn nuevo() -> Self {
        let mapa = MapaJuego::cargar();
        let recogidas = vec![false; mapa.coleccionables.len()];
        Self {
            jugador: Jugador::nuevo(mapa.inicio),
            mapa,
            buffer: BufferPantalla::nuevo(ANCHO, ALTO),
            pantalla: Pantalla::Bienvenida,
            mostrar_mapa: false,
            recogidas,
            tiempo_animacion: 0.0,
            cuadros_por_segundo: 30.0,
            audio: GestorAudio::nuevo(),
            texturas: AtlasTexturas::cargar(),
            sprites: RecursosSprites::cargar(),
            e_presionada: false,
            r_presionada: false,
        }
    }

    /// Reinicia la posición, llaves, mapa descubierto y pantalla del nivel.
    fn reiniciar_nivel(&mut self) {
        self.jugador.reiniciar(self.mapa.inicio);
        self.mapa
            .descubiertas
            .iter_mut()
            .for_each(|fila| fila.fill(false));
        self.recogidas.fill(false);
        self.mostrar_mapa = false;
        self.pantalla = Pantalla::Jugando;
    }

    /// Actualiza controles, exploración, coleccionables y condición de victoria.
    fn actualizar(&mut self, ventana: &Window, segundos: f32) {
        self.tiempo_animacion += segundos;
        let teclas_presionadas = ventana.get_keys_pressed(KeyRepeat::No);
        let mapa_presionado =
            teclas_presionadas.contains(&Key::M) || teclas_presionadas.contains(&Key::Tab);
        let interactuar = tecla_presionada_una_vez(ventana, Key::E, &mut self.e_presionada);
        let reiniciar = tecla_presionada_una_vez(ventana, Key::R, &mut self.r_presionada);
        match self.pantalla {
            Pantalla::Bienvenida => {
                if mapa_presionado {
                    self.reiniciar_nivel();
                    self.mostrar_mapa = true;
                } else if ventana.is_key_pressed(Key::Enter, KeyRepeat::No)
                    || ventana.is_key_pressed(Key::Space, KeyRepeat::No)
                {
                    self.reiniciar_nivel();
                }
            }
            Pantalla::Jugando => {
                if mapa_presionado {
                    self.mostrar_mapa = !self.mostrar_mapa;
                }
                if reiniciar {
                    self.reiniciar_nivel();
                    return;
                }
                let posicion_anterior = self.jugador.posicion;
                self.jugador.leer_entrada(ventana, &self.mapa, segundos);
                if posicion_anterior == self.jugador.posicion
                    && (ventana.is_key_down(Key::W) || ventana.is_key_down(Key::S))
                {
                    if let Some(audio) = &self.audio {
                        audio.reproducir_golpe_pared();
                    }
                }
                self.mapa.revelar_cerca(self.jugador.posicion, 3);
                self.recoger_objetos();
                let cerca_salida = (self.jugador.posicion - self.mapa.salida.posicion).magnitude()
                    < TAMANO_CELDA * 0.9;
                if cerca_salida
                    && self.recogidas.iter().all(|encontrada| *encontrada)
                    && interactuar
                {
                    self.pantalla = Pantalla::Exito;
                    if let Some(audio) = &self.audio {
                        audio.reproducir_exito();
                    }
                }
                if let Some(audio) = &self.audio {
                    audio.actualizar();
                }
            }
            Pantalla::Exito => {
                if ventana.is_key_pressed(Key::Enter, KeyRepeat::No) || reiniciar {
                    self.reiniciar_nivel();
                }
            }
        }
    }

    /// Marca las llaves cercanas como recogidas y reproduce su efecto.
    fn recoger_objetos(&mut self) {
        for (indice, objeto) in self.mapa.coleccionables.iter().enumerate() {
            if !self.recogidas[indice]
                && (self.jugador.posicion - objeto.posicion).magnitude() < TAMANO_CELDA * 0.42
            {
                self.recogidas[indice] = true;
                if let Some(audio) = &self.audio {
                    audio.reproducir_recoleccion();
                }
            }
        }
    }

    /// Renderiza la pantalla correspondiente al estado actual.
    fn renderizar(&mut self) {
        match self.pantalla {
            Pantalla::Bienvenida => self.renderizar_bienvenida(),
            Pantalla::Jugando => self.renderizar_jugando(),
            Pantalla::Exito => self.renderizar_exito(),
        }
    }

    /// Dibuja la pantalla de bienvenida con estética de terror.
    fn renderizar_bienvenida(&mut self) {
        self.buffer.limpiar(0x08070B);
        let parpadeo = (self.tiempo_animacion * 7.0).sin().abs();
        for fila in 0..ALTO as i32 {
            let rojo = (8.0 + fila as f32 * 18.0 / ALTO as f32 + parpadeo * 5.0) as u32;
            self.buffer.linea_horizontal(
                fila,
                0,
                ANCHO as i32 - 1,
                (rojo << 16) | ((rojo / 4) << 8) | 7,
            );
        }
        self.dibujar_decoracion_menu();
        self.dibujar_vineta();
        texto_centrado(&mut self.buffer, "EXPEDIENTE 09", 96, 0x807A75);
        texto_centrado(&mut self.buffer, "SIERRA", 145, 0xE2B34C);
        texto_centrado(&mut self.buffer, "EL LABERINTO DE HIERRO", 202, 0xC7C0B6);
        texto_centrado(&mut self.buffer, "HAS SIDO ELEGIDO", 296, 0x9D2735);
        texto_centrado(&mut self.buffer, "NO HAY SALIDA", 336, 0x7E202C);
        self.buffer.dibujar_rectangulo(610, 425, 700, 105, 0x9D2735);
        self.buffer.dibujar_rectangulo(620, 435, 680, 85, 0x4B1A22);
        if parpadeo > 0.18 {
            texto_centrado(&mut self.buffer, "PULSA ENTER", 460, 0xE2B34C);
        }
        texto_centrado(
            &mut self.buffer,
            "ENCUENTRA 3 LLAVES   LLEGA A LA PUERTA   PULSA E",
            610,
            0x807A75,
        );
        texto_centrado(
            &mut self.buffer,
            "WASD MOVER   RATÓN GIRAR   M / TAB MAPA",
            650,
            0x807A75,
        );
        texto_centrado(&mut self.buffer, "UNA TRAMPA PARA LOS VIVOS", 920, 0x5E5550);
    }

    /// Dibuja barras, marcas y el ojo animado del menú.
    fn dibujar_decoracion_menu(&mut self) {
        let pulso = if (self.tiempo_animacion * 4.0).sin() > 0.0 {
            0x9D2735
        } else {
            0x4A1720
        };
        self.buffer.rellenar_rectangulo(70, 120, 8, 760, 0x21181D);
        self.buffer
            .rellenar_rectangulo(ANCHO as i32 - 78, 120, 8, 760, 0x21181D);
        for x in [110, 1810] {
            self.buffer.rellenar_rectangulo(x, 180, 3, 590, 0x3A2428);
            for y in (200..760).step_by(55) {
                self.buffer.rellenar_rectangulo(x - 9, y, 21, 3, 0x574147);
            }
        }
        self.buffer.rellenar_rectangulo(180, 820, 1560, 5, pulso);
        self.buffer.rellenar_rectangulo(180, 827, 1560, 2, 0x3A171E);
        for i in 0..12 {
            let x = 210 + i * 135;
            self.buffer.rellenar_rectangulo(
                x,
                835,
                72,
                8,
                if i % 2 == 0 { 0x8D2636 } else { 0x271317 },
            );
        }
        let eye_x = ANCHO as i32 / 2;
        let eye_y = 780;
        self.buffer
            .rellenar_rectangulo(eye_x - 48, eye_y - 3, 96, 6, 0x32151B);
        self.buffer
            .rellenar_rectangulo(eye_x - 12, eye_y - 12, 24, 24, pulso);
        self.buffer
            .rellenar_rectangulo(eye_x - 4, eye_y - 4, 8, 8, 0xE2B34C);
        for y in (40..ALTO as i32).step_by(17) {
            if (y + self.tiempo_animacion as i32) % 7 == 0 {
                self.buffer
                    .linea_horizontal(y, 100, ANCHO as i32 - 100, 0x251419);
            }
        }
    }

    /// Renderiza el mundo, los sprites, el HUD y el minimapa.
    fn renderizar_jugando(&mut self) {
        let buffer_profundidad =
            renderizar_mundo(&mut self.buffer, &self.mapa, &self.jugador, &self.texturas);
        self.dibujar_sprites_mundo(&buffer_profundidad);
        self.dibujar_hud();
        if self.mostrar_mapa {
            self.dibujar_minimapa();
        }
    }

    /// Renderiza la pantalla de éxito después de abrir la puerta.
    fn renderizar_exito(&mut self) {
        self.buffer.limpiar(0x100A0D);
        for fila in 0..ALTO as i32 {
            self.buffer.linea_horizontal(
                fila,
                0,
                ANCHO as i32 - 1,
                0x100A0D + ((fila as u32 / 12) << 16),
            );
        }
        self.dibujar_vineta();
        texto_centrado(&mut self.buffer, "TRAMPA DESACTIVADA", 190, 0xE2B34C);
        texto_centrado(&mut self.buffer, "SOBREVIVISTE AL LABERINTO", 265, 0xE8E0D2);
        texto_centrado(&mut self.buffer, "ENTER O R PARA REINICIAR", 430, 0xC7C0B6);
    }

    /// Dibuja FPS, estado del mapa, llaves y objetivo actual.
    fn dibujar_hud(&mut self) {
        self.buffer
            .rellenar_rectangulo(0, 0, ANCHO as i32, 38, 0x100D12);
        self.buffer
            .linea_horizontal(37, 0, ANCHO as i32 - 1, 0x762432);
        dibujar_texto(
            &mut self.buffer,
            &format!("FPS: {}", self.cuadros_por_segundo.round() as i32),
            18,
            12,
            2,
            0xD7CBB8,
        );
        dibujar_texto(
            &mut self.buffer,
            if self.mostrar_mapa {
                "MAPA: SÍ"
            } else {
                "MAPA: NO"
            },
            150,
            12,
            2,
            0xD7CBB8,
        );
        let encontradas = self.recogidas.iter().filter(|objeto| **objeto).count();
        dibujar_texto(
            &mut self.buffer,
            &format!("LLAVES: {}/{}", encontradas, self.recogidas.len()),
            285,
            12,
            2,
            0xE2B34C,
        );
        if encontradas == self.recogidas.len() {
            dibujar_texto(&mut self.buffer, "SALIDA: PULSA E", 470, 12, 2, 0xE2B34C);
        } else {
            dibujar_texto(
                &mut self.buffer,
                "ENCUENTRA LAS 3 LLAVES",
                470,
                12,
                2,
                0xD7CBB8,
            );
        }
    }

    /// Proyecta y dibuja en pantalla las llaves todavía no recogidas.
    fn dibujar_sprites_mundo(&mut self, buffer_profundidad: &[f32]) {
        for indice in 0..self.mapa.coleccionables.len() {
            let objeto = self.mapa.coleccionables[indice];
            if !self.recogidas[indice] {
                if let Some(proyeccion) =
                    self.proyectar_sprite(objeto.posicion, buffer_profundidad, 0.72, indice as f32)
                {
                    self.sprites
                        .dibujar_llave(&mut self.buffer, proyeccion, indice);
                }
            }
        }
    }

    /// Calcula la posición y tamaño de un sprite visto desde la cámara.
    fn proyectar_sprite(
        &self,
        posicion: Vec2,
        buffer_profundidad: &[f32],
        tamano: f32,
        fase_animacion: f32,
    ) -> Option<ProyeccionSprite> {
        let relativo = posicion - self.jugador.posicion;
        let direccion_frontal = self.jugador.direccion_frontal();
        let derecha = Vec2::new(-direccion_frontal.y, direccion_frontal.x);
        let distancia = relativo.dot(&direccion_frontal);
        if distancia <= 8.0 {
            return None;
        }
        let distancia_focal = ANCHO as f32 / (2.0 * (raycaster::CAMPO_VISION / 2.0).tan());
        let x_pantalla = ANCHO as f32 / 2.0 + relativo.dot(&derecha) / distancia * distancia_focal;
        let pulso = 1.0 + (self.tiempo_animacion * 4.0 + fase_animacion).sin() * 0.06;
        let alto =
            (TAMANO_CELDA * ALTO as f32 / distancia * tamano * pulso).clamp(10.0, 360.0) as i32;
        let ancho = (alto as f32 * 0.62) as i32;
        let centro_x = x_pantalla as i32;
        if centro_x + ancho / 2 < 0 || centro_x - ancho / 2 >= ANCHO as i32 {
            return None;
        }
        if centro_x >= 0
            && centro_x < ANCHO as i32
            && distancia > buffer_profundidad[centro_x as usize] + 8.0
        {
            return None;
        }
        let desplazamiento_suelo = (TAMANO_CELDA * ALTO as f32 / (2.0 * distancia)) as i32;
        let inferior = ALTO as i32 / 2 + desplazamiento_suelo;
        Some(ProyeccionSprite {
            centro_x,
            superior: inferior - alto,
            inferior,
            alto,
        })
    }

    /// Dibuja el minimapa con paredes y zonas descubiertas por el jugador.
    fn dibujar_minimapa(&mut self) {
        let tamano_celda = 10;
        let ancho_mapa = self.mapa.ancho() as i32 * tamano_celda;
        let origen_x = 30;
        let origen_y = 70;
        self.buffer.rellenar_rectangulo(
            origen_x - 8,
            origen_y - 8,
            ancho_mapa + 16,
            self.mapa.alto() as i32 * tamano_celda + 16,
            0x120E12,
        );
        self.buffer.dibujar_rectangulo(
            origen_x - 8,
            origen_y - 8,
            ancho_mapa + 16,
            self.mapa.alto() as i32 * tamano_celda + 16,
            0xB68A4A,
        );
        dibujar_texto(
            &mut self.buffer,
            "MAPA ABIERTO",
            origen_x,
            origen_y - 26,
            2,
            0xE2B34C,
        );
        for fila in 0..self.mapa.alto() {
            for columna in 0..self.mapa.ancho() {
                if !self.mapa.descubiertas[fila][columna] {
                    continue;
                }
                let color = if self.mapa.es_pared(columna as i32, fila as i32) {
                    MapaJuego::color_pared(self.mapa.baldosa_en(columna as i32, fila as i32))
                } else {
                    0x30272A
                };
                self.buffer.rellenar_rectangulo(
                    origen_x + columna as i32 * tamano_celda,
                    origen_y + fila as i32 * tamano_celda,
                    tamano_celda - 1,
                    tamano_celda - 1,
                    color,
                );
            }
        }
        let (columna_salida, fila_salida) = MapaJuego::mundo_a_celda(self.mapa.salida.posicion);
        if fila_salida >= 0
            && columna_salida >= 0
            && (fila_salida as usize) < self.mapa.alto()
            && (columna_salida as usize) < self.mapa.ancho()
            && self.mapa.descubiertas[fila_salida as usize][columna_salida as usize]
        {
            self.buffer.rellenar_rectangulo(
                origen_x + columna_salida * tamano_celda,
                origen_y + fila_salida * tamano_celda,
                tamano_celda - 1,
                tamano_celda - 1,
                0xE2B34C,
            );
            dibujar_texto(
                &mut self.buffer,
                "E",
                origen_x + columna_salida * tamano_celda + 1,
                origen_y + fila_salida * tamano_celda,
                1,
                0x170E10,
            );
        }
        for (indice, objeto) in self.mapa.coleccionables.iter().enumerate() {
            let (columna_objeto, fila_objeto) = MapaJuego::mundo_a_celda(objeto.posicion);
            if fila_objeto >= 0
                && columna_objeto >= 0
                && (fila_objeto as usize) < self.mapa.alto()
                && (columna_objeto as usize) < self.mapa.ancho()
                && self.mapa.descubiertas[fila_objeto as usize][columna_objeto as usize]
                && !self.recogidas[indice]
            {
                self.buffer.rellenar_rectangulo(
                    origen_x + columna_objeto * tamano_celda + 2,
                    origen_y + fila_objeto * tamano_celda + 2,
                    tamano_celda - 4,
                    tamano_celda - 4,
                    0xD94F56,
                );
            }
        }
        let (columna_jugador, fila_jugador) = MapaJuego::mundo_a_celda(self.jugador.posicion);
        self.buffer.rellenar_rectangulo(
            origen_x + columna_jugador * tamano_celda + 1,
            origen_y + fila_jugador * tamano_celda + 1,
            tamano_celda - 2,
            tamano_celda - 2,
            0xE8D36C,
        );
        dibujar_texto(
            &mut self.buffer,
            "DESCUBIERTO",
            origen_x,
            origen_y + self.mapa.alto() as i32 * tamano_celda + 10,
            1,
            0xB68A4A,
        );
    }

    /// Dibuja bordes oscuros para crear una viñeta cinematográfica.
    fn dibujar_vineta(&mut self) {
        for indice in 0..40 {
            self.buffer.dibujar_rectangulo(
                indice,
                indice,
                ANCHO as i32 - 2 * indice,
                ALTO as i32 - 2 * indice,
                0x190C10,
            );
        }
    }
}

#[derive(Clone, Copy)]
pub struct ProyeccionSprite {
    pub centro_x: i32,
    pub superior: i32,
    pub inferior: i32,
    pub alto: i32,
}

/// Dibuja un texto centrado horizontalmente en la pantalla.
fn texto_centrado(buffer: &mut BufferPantalla, texto: &str, y: i32, color: u32) {
    let escala = 4;
    dibujar_texto(
        buffer,
        texto,
        (ANCHO as i32 - ancho_texto(texto, escala)) / 2,
        y,
        escala,
        color,
    );
}

/// Detecta una pulsación única aunque la tecla permanezca presionada.
fn tecla_presionada_una_vez(ventana: &Window, tecla: Key, estaba_presionada: &mut bool) -> bool {
    let esta_presionada = ventana.is_key_down(tecla);
    let nueva_pulsacion = esta_presionada && !*estaba_presionada;
    *estaba_presionada = esta_presionada;
    nueva_pulsacion
}

#[cfg(target_os = "windows")]
/// Activa el escalado DPI consciente para alinear la ventana con el mouse.
fn configurar_dpi_windows() {
    unsafe {
        winapi::um::winuser::SetProcessDPIAware();
    }
}

#[cfg(not(target_os = "windows"))]
/// No necesita configuración DPI fuera de Windows.
fn configurar_dpi_windows() {}

/// Crea la ventana y ejecuta el ciclo principal del juego.
fn main() {
    configurar_dpi_windows();
    let mut ventana = Window::new(
        "SIERRA - El laberinto de hierro | Ray caster en Rust",
        ANCHO,
        ALTO,
        WindowOptions {
            borderless: true,
            title: false,
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear la ventana");
    ventana.set_cursor_visibility(false);
    centrar_cursor(&ventana);
    ventana.set_target_fps(0);
    let mut juego = Juego::nuevo();
    let mut marco_anterior = Instant::now();
    while ventana.is_open() && !ventana.is_key_down(Key::Escape) {
        let inicio_marco = Instant::now();
        let segundos = marco_anterior.elapsed().as_secs_f32().clamp(0.001, 0.1);
        marco_anterior = inicio_marco;
        juego.actualizar(&ventana, segundos);
        juego.renderizar();
        if let Err(error) = ventana.update_with_buffer(&juego.buffer.pixeles, ANCHO, ALTO) {
            eprintln!("Error de render: {error}");
            break;
        }
        juego.cuadros_por_segundo = 1.0 / segundos;
        let transcurrido = inicio_marco.elapsed();
        if transcurrido < DURACION_OBJETIVO {
            std::thread::sleep(DURACION_OBJETIVO - transcurrido);
        }
    }
}
