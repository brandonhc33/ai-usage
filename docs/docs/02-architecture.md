# 02 — Arquitectura

## Arquitectura general

```txt
┌─────────────────────────────┐
│ Login manual Playwright      │
│ ai-usage-auth codex|claude   │
└──────────────┬──────────────┘
               ↓
┌─────────────────────────────┐
│ Perfil Chromium persistente  │
│ ~/.config/ai-usage-status/   │
└──────────────┬──────────────┘
               ↓
┌─────────────────────────────┐
│ Collector Playwright         │
│ ai-usage-collect             │
└──────────────┬──────────────┘
               ↓
┌─────────────────────────────┐
│ JSON sanitizado              │
│ ~/.cache/ai-usage-status/    │
└──────────────┬──────────────┘
               ↓
┌─────────────────────────────┐
│ COSMIC applet                │
│ lee status.json              │
└─────────────────────────────┘
```

## Separación de responsabilidades

### Applet COSMIC

Responsabilidades:

- Leer `~/.cache/ai-usage-status/status.json`.
- Renderizar icono + porcentaje.
- Mostrar popup con detalle.
- Mostrar errores de forma simple.
- Refrescar cada 30 a 60 segundos leyendo archivo local.

No debe:

- Ejecutar Playwright.
- Abrir navegador.
- Hacer login.
- Leer cookies.
- Guardar sesiones.

### Collector

Responsabilidades:

- Abrir páginas web de Codex y Claude con Playwright.
- Usar perfiles persistentes ya logueados.
- Leer texto visible de los paneles de uso.
- Parsear porcentajes, reinicios y créditos.
- Escribir JSON limpio.

### Auth helper

Responsabilidades:

- Abrir Chromium visible.
- Permitir login manual.
- Validar que la pantalla de uso cargó.
- Cerrar guardando cookies en el perfil persistente.

## Ubicaciones

```txt
~/.config/ai-usage-status/
├── config.json
└── profiles/
    ├── chatgpt/
    └── claude/

~/.cache/ai-usage-status/
├── status.json
├── codex.json
├── claude.json
└── last-error.json
```

## Fuentes de datos

### Codex

Fuente principal:

```txt
https://chatgpt.com/codex/cloud/settings/analytics#usage
```

Datos esperados:

- Límite de uso de 5 horas.
- Límite de uso semanal.
- Créditos restantes.

### Claude

Fuente principal por uniformidad:

```txt
https://claude.ai/settings/usage
```

Alternativa/fallback recomendado:

```txt
Claude Code statusLine
```

Claude Code puede entregar JSON oficial con `rate_limits.five_hour` y `rate_limits.seven_day`, pero para mantener la misma línea que Codex se deja Playwright como primera etapa del proyecto.

## Flujo de actualización

```txt
systemd --user timer cada 3 min
        ↓
ai-usage-collect
        ↓
lee Codex + Claude
        ↓
escribe status.json
        ↓
applet refresca lectura local
```

## Por qué no usar CLI como base única

- Claude Code tiene una vía buena mediante `statusLine`.
- Codex CLI puede mostrar `/status`, pero no es una fuente universal y estable para extraer el mismo dashboard web.
- El dashboard web de Codex es la fuente más cercana a los datos visuales deseados.
