# Prompt — Crear applet COSMIC AI Usage

Necesito crear un COSMIC applet para Pop!_OS 24 basado en la estructura del repositorio `visual-mute`.

Objetivo:

- Mostrar en el topbar un icono de IA/Codex/Claude a color.
- Mostrar al lado solo un porcentaje, por ejemplo `26%`.
- El porcentaje viene de `~/.cache/ai-usage-status/status.json`, campo `primary.label`.
- Al hacer click, abrir un panel con el detalle de Codex y Claude.

Requisitos técnicos:

- Rust + libcosmic, siguiendo el patrón del applet existente.
- Desktop id: `com.github.brandonhc33.ai-usage`.
- Desktop file con `X-CosmicApplet=true` y `X-CosmicHoverPopup=Auto`.
- No ejecutar Playwright desde el applet.
- No hacer login desde el applet.
- Refrescar lectura del JSON cada 30 segundos.
- Manejar archivo inexistente, JSON inválido, login requerido y datos desactualizados.

Contrato JSON:

- Lee `~/.cache/ai-usage-status/status.json`.
- Usa `primary.provider`, `primary.used_percent`, `primary.label`.
- Popup muestra `codex.five_hour`, `codex.weekly`, `codex.credits_remaining`, `claude.session`, `claude.weekly`, `claude.daily_routines`.

Crea:

- `src/status.rs` para tipos serde y lectura segura.
- `src/format.rs` para formatear labels.
- `src/app.rs` para UI.
- Recursos en `resources/icons/`.
- README con instalación.

Usa como contexto los markdown del paquete `ai-usage-cosmic-applet-context`.
