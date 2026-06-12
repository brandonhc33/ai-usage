# Prompt — Crear collector Playwright

Necesito crear un collector Node.js con Playwright para alimentar un COSMIC applet.

Objetivo:

- Login manual con perfiles Chromium persistentes.
- Leer panel de Codex Usage.
- Leer panel de Claude Usage.
- Parsear porcentaje usado/restante y reinicios.
- Escribir `~/.cache/ai-usage-status/status.json`.

Comandos requeridos:

```bash
ai-usage-auth codex
ai-usage-auth claude
ai-usage-collect
```

Ubicaciones:

```txt
~/.config/ai-usage-status/config.json
~/.config/ai-usage-status/profiles/chatgpt
~/.config/ai-usage-status/profiles/claude
~/.cache/ai-usage-status/status.json
```

Reglas:

- No guardar contraseñas.
- No guardar HTML completo por defecto.
- Escritura atómica de JSON.
- Soportar español e inglés cuando sea posible.
- Si hay login vencido, escribir error `login_required`.
- Si el parser falla, escribir error `page_changed`.

Codex:

- URL: `https://chatgpt.com/codex/cloud/settings/analytics#usage`.
- Extraer límite 5 horas, semanal y créditos restantes.
- Codex suele mostrar porcentaje restante, convertir a usado.

Claude:

- URL base: `https://claude.ai/settings/usage`.
- Extraer sesión actual, semanal/todos los modelos y rutinas diarias.
- Claude suele mostrar porcentaje usado, calcular restante.

Usa el contrato de `docs/03-data-contract-status-json.md`.
