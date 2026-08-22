# SAW: El laberinto de hierro

## Ejecutar

```bash
cargo run --release
```

Se necesita Rust estable y una computadora con una ventana gráfica.

## Controles

| Tecla | Acción |
| --- | --- |
| `W` / `S` | Avanzar / retroceder |
| `A` / `D` | Girar con teclado |
| Mouse horizontal | Girar la cámara |
| `M` / `Tab` | Mostrar/ocultar mapa descubierto |
| `E` | Interactuar con la puerta cuando se tienen las 3 llaves |
| `R` | Reiniciar el nivel |
| `Escape` | Salir |

El minimapa permanece oculto hasta pulsar `M`. Cuando se muestra, solo dibuja las celdas visitadas y las paredes cercanas descubiertas, como un mapa que se va revelando. La puerta está marcada con `E`; encuentra las tres llaves (`B`, `P` y `K`) y pulsa `E` cerca de la puerta para ganar.


En el mapa, `R` representa ladrillo rojo, `W` madera, `I` hierro y `D` la puerta de salida. Las marcas `B`, `P` y `K` indican las posiciones de las llaves dorada, plateada y roja.

Video Youtube: https://youtu.be/tl3DmUuF8ik
