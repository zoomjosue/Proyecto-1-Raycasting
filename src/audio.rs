use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::time::Duration;
use std::{env, fs::File, io::BufReader, path::PathBuf};

/// Gestiona la música ambiental y los efectos de sonido.
pub struct GestorAudio {
    _flujo: OutputStream,
    ambiente: Sink,
}

impl GestorAudio {
    /// Inicia la salida de audio y carga la música disponible.
    pub fn nuevo() -> Option<Self> {
        let flujo = match OutputStreamBuilder::open_default_stream() {
            Ok(flujo) => flujo,
            Err(error) => {
                eprintln!("Audio desactivado: no se pudo abrir la salida de audio: {error}");
                return None;
            }
        };
        let ambiente = Sink::connect_new(flujo.mixer());
        if !agregar_musica_disponible(&ambiente) {
            eprintln!("No se encontró una pista compatible; se usará el ambiente procedural.");
            ambiente.append(SineWave::new(52.0).amplify(0.018).repeat_infinite());
        }
        ambiente.play();
        Some(Self {
            _flujo: flujo,
            ambiente,
        })
    }

    /// Mantiene activa la música ambiental durante el juego.
    pub fn actualizar(&self) {
        if self.ambiente.is_paused() {
            self.ambiente.play();
        }
    }

    /// Reproduce el sonido de golpear una pared.
    pub fn reproducir_golpe_pared(&self) {
        self.reproducir_tono(110.0, 70, 0.06);
    }

    /// Reproduce el sonido de recoger una llave.
    pub fn reproducir_recoleccion(&self) {
        self.reproducir_tono(520.0, 160, 0.09);
    }

    /// Reproduce el sonido de completar el nivel.
    pub fn reproducir_exito(&self) {
        self.reproducir_tono(760.0, 420, 0.12);
    }

    /// Crea un tono corto para un efecto del juego.
    fn reproducir_tono(&self, frecuencia: f32, milisegundos: u64, volumen: f32) {
        let sumidero = Sink::connect_new(self._flujo.mixer());
        sumidero.append(
            SineWave::new(frecuencia)
                .take_duration(Duration::from_millis(milisegundos))
                .amplify(volumen),
        );
        sumidero.detach();
    }
}

/// Busca y agrega la primera pista compatible al sumidero ambiental.
fn agregar_musica_disponible(sumidero: &Sink) -> bool {
    for nombre_archivo in [
        "assets/music/taylor.wav",
        "assets/music/ambient.mp3",
    ] {
        for ruta in rutas_posibles_musica(nombre_archivo) {
            let Ok(archivo) = File::open(ruta) else {
                continue;
            };
            let Ok(fuente) = Decoder::try_from(BufReader::new(archivo)) else {
                continue;
            };
            eprintln!("Música de fondo cargada: {nombre_archivo}");
            sumidero.append(fuente.repeat_infinite().amplify(0.55));
            return true;
        }
    }
    false
}

/// Construye rutas posibles para funcionar desde el proyecto o desde `target`.
fn rutas_posibles_musica(nombre_archivo: &str) -> Vec<PathBuf> {
    let mut rutas = vec![PathBuf::from(nombre_archivo)];
    if let Ok(ejecutable) = env::current_exe() {
        for directorio in ejecutable.ancestors() {
            rutas.push(directorio.join(nombre_archivo));
        }
    }
    rutas
}
