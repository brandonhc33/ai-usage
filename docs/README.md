# AI Usage COSMIC Applet — contexto inicial

Este paquete contiene el contexto para crear un **COSMIC applet para Pop!_OS 24 / COSMIC**, basado en el enfoque usado en `visual-mute`, pero orientado a mostrar el uso real/visual de **Codex/ChatGPT** y **Claude** en el topbar.

## Objetivo

En el topbar debe aparecer un indicador compacto:

```txt
[icono IA/Codex/Claude]  26%
```

Donde el porcentaje visible representa el **porcentaje de uso** más importante del proveedor activo o seleccionado.

Al hacer click, el applet debe abrir un panel con el detalle:

```txt
Codex
5 horas: 100% usado · reinicia 4:39
Semanal: 26% usado · reinicia 18 jun 2026 15:29
Créditos: 128

Claude
Sesión actual: 13% usado · reinicia en 2 h 33 min
Semanal: 1% usado · reinicia mié., 11:59 p.m.
Rutinas: 0 / 5
```

## Decisión técnica

El applet no debe hacer login ni scraping directamente. Debe ser liviano y solo leer:

```txt
~/.cache/ai-usage-status/status.json
```

La obtención de datos se hace con un **collector externo**:

```txt
Playwright → login persistente → scraping de paneles web → JSON limpio → applet
```

## Estructura del paquete

```txt
ai-usage-cosmic-applet-context/
├── README.md
├── docs/
│   ├── 01-product-vision.md
│   ├── 02-architecture.md
│   ├── 03-data-contract-status-json.md
│   ├── 04-login-and-session-storage.md
│   ├── 05-playwright-collector.md
│   ├── 06-cosmic-applet-ui.md
│   ├── 07-installation-systemd.md
│   ├── 08-security-and-privacy.md
│   ├── 09-implementation-plan.md
│   ├── 10-reuse-visual-mute-notes.md
│   ├── 11-error-states.md
│   ├── 12-parsers.md
│   └── 13-testing-checklist.md
├── templates/
│   ├── config.example.json
│   ├── status.example.json
│   ├── codex-page-text-sample.txt
│   ├── claude-page-text-sample.txt
│   └── app.desktop.example
├── scripts/
│   ├── ai-usage-auth.mjs.example
│   ├── ai-usage-collect.mjs.example
│   └── merge-status.mjs.example
├── systemd/
│   ├── ai-usage-collector.service
│   └── ai-usage-collector.timer
├── assets/icons/
│   ├── ai-orb.svg
│   ├── codex-orb.svg
│   └── claude-orb.svg
└── prompts/
    ├── create-applet.prompt.md
    ├── collector.prompt.md
    └── ui.prompt.md
```

## Orden recomendado para empezar

1. Clonar o copiar la base de `visual-mute`.
2. Cambiar nombre, desktop id, metainfo e iconos.
3. Crear el collector Playwright y validar login.
4. Generar `status.json` con datos falsos.
5. Hacer que el applet lea ese JSON.
6. Crear el panel desplegable.
7. Recién después conectar scraping real.

## Nombre sugerido

```txt
ai-usage-applet
com.github.brandonhc33.ai-usage
AI Usage
```
