use nalgebra_glm::Vec2;

pub const TAMANO_CELDA: f32 = 64.0;
pub type Cuadricula = Vec<Vec<char>>;

#[derive(Clone, Copy)]
pub struct Salida {
    pub posicion: Vec2,
}

#[derive(Clone, Copy)]
pub struct Coleccionable {
    pub posicion: Vec2,
}

pub struct MapaJuego {
    pub celdas: Cuadricula,
    pub inicio: Vec2,
    pub salida: Salida,
    pub coleccionables: Vec<Coleccionable>,
    pub descubiertas: Vec<Vec<bool>>,
}

impl MapaJuego {
    /// Carga el mapa ASCII y convierte sus marcas especiales en objetos del juego.
    pub fn cargar() -> Self {
        let mut celdas = Vec::new();
        let mut inicio = Vec2::new(TAMANO_CELDA * 1.5, TAMANO_CELDA * 1.5);
        let mut salida = Salida {
            posicion: Vec2::new(TAMANO_CELDA * 22.5, TAMANO_CELDA * 19.5),
        };
        let mut coleccionables = Vec::new();
        for (fila, linea) in include_str!("../map.txt").lines().enumerate() {
            let mut fila_parseada = Vec::new();
            for (columna, baldosa) in linea.chars().enumerate() {
                let posicion = Vec2::new(
                    (columna as f32 + 0.5) * TAMANO_CELDA,
                    (fila as f32 + 0.5) * TAMANO_CELDA,
                );
                match baldosa {
                    'S' => {
                        inicio = posicion;
                        fila_parseada.push(' ');
                    }
                    'D' => {
                        salida = Salida { posicion };
                        fila_parseada.push('D');
                    }
                    'B' | 'P' | 'K' => {
                        coleccionables.push(Coleccionable { posicion });
                        fila_parseada.push(' ');
                    }
                    _ => fila_parseada.push(baldosa),
                }
            }
            celdas.push(fila_parseada);
        }
        let descubiertas = vec![vec![false; celdas.first().map_or(0, Vec::len)]; celdas.len()];
        Self {
            celdas,
            inicio,
            salida,
            coleccionables,
            descubiertas,
        }
    }

    /// Devuelve el número de columnas del mapa.
    pub fn ancho(&self) -> usize {
        self.celdas.first().map_or(0, Vec::len)
    }

    /// Devuelve el número de filas del mapa.
    pub fn alto(&self) -> usize {
        self.celdas.len()
    }

    /// Indica si una celda es una pared o está fuera del mapa.
    pub fn es_pared(&self, columna: i32, fila: i32) -> bool {
        if fila < 0 || columna < 0 {
            return true;
        }
        self.celdas
            .get(fila as usize)
            .and_then(|fila| fila.get(columna as usize))
            .is_none_or(|baldosa| *baldosa != ' ')
    }

    /// Obtiene la baldosa de una celda, usando pared si está fuera del mapa.
    pub fn baldosa_en(&self, columna: i32, fila: i32) -> char {
        self.celdas
            .get(fila as usize)
            .and_then(|fila| fila.get(columna as usize))
            .copied()
            .unwrap_or('#')
    }

    /// Convierte una posición del mundo a coordenadas de celda.
    pub fn mundo_a_celda(posicion: Vec2) -> (i32, i32) {
        (
            (posicion.x / TAMANO_CELDA) as i32,
            (posicion.y / TAMANO_CELDA) as i32,
        )
    }
    /// Marca como descubiertas las celdas cercanas al jugador.
    pub fn revelar_cerca(&mut self, posicion: Vec2, radio: i32) {
        let (celda_x, celda_y) = Self::mundo_a_celda(posicion);
        for fila in celda_y - radio..=celda_y + radio {
            for columna in celda_x - radio..=celda_x + radio {
                if fila >= 0
                    && columna >= 0
                    && (fila as usize) < self.alto()
                    && (columna as usize) < self.ancho()
                {
                    self.descubiertas[fila as usize][columna as usize] = true;
                }
            }
        }
    }

    /// Devuelve el color usado para representar una baldosa en el minimapa.
    pub fn color_pared(baldosa: char) -> u32 {
        match baldosa {
            '#' => 0x7D2935,
            'R' => 0xA84436,
            'W' => 0xA26A3A,
            'I' => 0x2D6B7D,
            'D' => 0x4E3027,
            'B' => 0x9A4C34,
            'P' => 0x6A3D86,
            _ => 0x63303B,
        }
    }
}
