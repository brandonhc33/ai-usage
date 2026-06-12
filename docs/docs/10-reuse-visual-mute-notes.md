# 10 — Notas para reutilizar visual-mute

## Base conocida

`visual-mute` ya es un applet nativo para Pop!_OS 24 / COSMIC. Su README indica que usa integración COSMIC con `X-CosmicApplet=true`, muestra un indicador siempre visible, refresca automáticamente y usa una estructura simple con Rust, `justfile`, recursos e iconos.

## Estructura útil a replicar

```txt
src/
├── main.rs
└── app.rs

resources/
├── icons/
├── app.desktop
└── app.metainfo.xml

justfile
Cargo.toml
README.md
```

## Qué cambiar

### De visual-mute

```txt
pactl get-source-mute @DEFAULT_SOURCE@
pactl set-source-mute @DEFAULT_SOURCE@ toggle
```

### A ai-usage

```txt
leer ~/.cache/ai-usage-status/status.json
mostrar primary.label
click abre popup, no toggle
```

## Desktop id

De:

```txt
com.github.brandonhc33.visual-mute
```

A:

```txt
com.github.brandonhc33.ai-usage
```

## Iconos

De:

```txt
mic-on.svg
mic-muted.svg
```

A:

```txt
ai-orb.svg
codex-orb.svg
claude-orb.svg
```

## Refresh

Visual Mute refresca rápido porque el mute puede cambiar por teclado o apps externas.

Para AI Usage:

```txt
Applet: leer archivo cada 30 s.
Collector: actualizar web cada 180 s.
```

## Click behavior

Visual Mute:

```txt
click = toggle mute
```

AI Usage:

```txt
click = abrir popup con detalle
```

## Código sugerido

Crear módulos:

```txt
src/app.rs
src/status.rs
src/format.rs
src/icons.rs
```

## Dependencias Rust probables

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
```

Mantener lo demás similar a la base de COSMIC/libcosmic que ya funcionó en visual-mute.
