# 06 — UI del COSMIC applet

## Topbar

El topbar debe mostrar:

```txt
[icono] 26%
```

## Iconos

Estados posibles:

| Provider primario | Icono sugerido |
|---|---|
| `codex` | `codex-orb.svg` |
| `claude` | `claude-orb.svg` |
| `none` | `ai-orb.svg` |

Usar iconos propios para evitar depender de marcas oficiales. Si se usan logos oficiales, revisar permisos y licencias.

## Colores sugeridos

```txt
Normal: color del provider
Advertencia: amarillo/naranjo si >= 75% usado
Crítico: rojo si >= 95% usado
Sin datos: gris
Login requerido: azul/gris con texto "login"
```

## Label

El applet debe leer:

```json
primary.label
```

Ejemplos:

```txt
26%
100%
--
login
!
```

## Tooltip

Tooltip sugerido:

```txt
AI Usage — Codex 5h: 100% usado, semanal: 26% usado
```

## Popup / panel al click

Contenido recomendado:

```txt
AI Usage
Actualizado hace 2 min

Codex
5 horas    100% usado    reinicia 4:39
Semanal     26% usado    reinicia 18 jun 2026 15:29
Créditos    128

Claude
Sesión      13% usado     reinicia en 2 h 33 min
Semanal      1% usado     reinicia mié., 11:59 p.m.
Rutinas      0 / 5

[Actualizar ahora] [Abrir configuración]
```

## Diseño visual del popup

Layout vertical simple:

```txt
Header
Cards pequeñas por proveedor
Footer con acciones
```

Cada card:

```txt
┌────────────────────────────┐
│ Codex                 100% │
│ 5 horas · reinicia 4:39    │
│ Semanal 26% · 74% restante │
│ Créditos 128              │
└────────────────────────────┘
```

## Acciones al click

Opcional:

- Click principal: abre popup.
- Botón `Actualizar ahora`: ejecuta `ai-usage-collect` en background.
- Botón `Login Codex`: ejecuta `ai-usage-auth codex` con terminal o muestra comando.
- Botón `Abrir panel Codex`: abre navegador con URL.
- Botón `Abrir panel Claude`: abre navegador con URL.

Para la primera versión, basta popup y lectura pasiva.

## Refresh del applet

El applet puede refrescar leyendo archivo cada:

```txt
30 segundos
```

No necesita coincidir con el collector.

## Estados visuales mínimos

| Condición | Topbar |
|---|---|
| `status.json` no existe | `AI --` |
| `ok=true` | `icon 26%` |
| `ok=false` pero hay datos | `icon 26%!` |
| login requerido | `AI login` |
| JSON inválido | `AI !` |
