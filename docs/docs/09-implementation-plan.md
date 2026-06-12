# 09 — Plan de implementación

## Fase 1 — Base del applet

1. Copiar estructura de `visual-mute`.
2. Renombrar crate, desktop id y metainfo.
3. Reemplazar iconos por `ai-orb.svg`, `codex-orb.svg`, `claude-orb.svg`.
4. Reemplazar lectura de `pactl` por lectura de `status.json`.
5. Mostrar label estático desde `templates/status.example.json`.

Resultado esperado:

```txt
[AI] 26%
```

## Fase 2 — Lectura de JSON real

1. Crear módulo Rust `status.rs`.
2. Parsear `status.json` con `serde`.
3. Manejar estados:
   - archivo no existe
   - JSON inválido
   - ok=false
   - login_required
4. Refrescar cada 30 segundos.

## Fase 3 — Popup

1. Al click abrir panel.
2. Mostrar cards de Codex y Claude.
3. Mostrar última actualización.
4. Mostrar errores si existen.

## Fase 4 — Collector fake

1. Crear `scripts/merge-status.mjs`.
2. Generar `status.json` con datos falsos.
3. Probar applet sin Playwright.

## Fase 5 — Auth Playwright

1. Implementar `ai-usage-auth.mjs`.
2. Login manual para Codex.
3. Login manual para Claude.
4. Validar perfiles persistentes.

## Fase 6 — Collector Playwright

1. Implementar lectura de Codex.
2. Guardar `codex.json`.
3. Implementar lectura de Claude.
4. Guardar `claude.json`.
5. Fusionar a `status.json`.

## Fase 7 — systemd

1. Instalar servicio user.
2. Timer cada 3 minutos.
3. Logs con journalctl.

## Fase 8 — Pulido

1. Botón “Actualizar ahora”.
2. Botones “abrir panel Codex/Claude”.
3. Config para provider primario.
4. Thresholds visuales.
5. README final.

## MVP mínimo

```txt
Applet lee status.json fake → muestra icono + porcentaje → popup con detalle.
```

Después conectar Playwright.
