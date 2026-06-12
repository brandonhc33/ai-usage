# 08 — Seguridad y privacidad

## Principios

- No guardar contraseñas.
- No guardar prompts ni conversaciones.
- No guardar HTML completo por defecto.
- No exponer cookies al applet.
- No subir perfiles Chromium a Git.
- No sincronizar perfiles con Drive/Dropbox.

## Carpetas sensibles

```txt
~/.config/ai-usage-status/profiles/chatgpt
~/.config/ai-usage-status/profiles/claude
```

Estas carpetas contienen cookies de sesión. Deben considerarse sensibles.

## Permisos

```bash
chmod 700 ~/.config/ai-usage-status
chmod 700 ~/.config/ai-usage-status/profiles
chmod 700 ~/.config/ai-usage-status/profiles/chatgpt
chmod 700 ~/.config/ai-usage-status/profiles/claude
chmod 700 ~/.cache/ai-usage-status
chmod 600 ~/.cache/ai-usage-status/*.json 2>/dev/null || true
```

## `.gitignore`

```gitignore
.cache/
.config/
profiles/
*.json
!templates/*.json
```

## Debug

Si se necesita guardar texto de debug:

```txt
~/.cache/ai-usage-status/debug-codex-text.txt
```

Debe ser opcional y apagado por defecto porque puede contener textos de la interfaz o datos de cuenta.

## Scraping responsable

Actualizar cada 3 a 5 minutos. Evitar loops rápidos. No intentar saltarse controles de seguridad. Si hay captcha o login, pedir login manual.

## Riesgo principal

La integración por Playwright puede romperse si cambian los textos o layout de los paneles.

Mitigación:

- Parser con español e inglés.
- Tests con textos sample.
- Mostrar error claro.
- Mantener último estado válido.
