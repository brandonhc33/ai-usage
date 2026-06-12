# 05 — Collector Playwright

## Responsabilidad

El collector debe abrir los paneles web autenticados, leer texto visible, parsear datos y escribir JSON limpio.

## Comando deseado

```bash
ai-usage-collect
```

## Flujo interno

```txt
1. Cargar config.json.
2. Abrir perfil ChatGPT/Codex.
3. Ir a Codex usage page.
4. Leer body.innerText().
5. Parsear Codex.
6. Cerrar perfil.
7. Abrir perfil Claude.
8. Ir a Claude usage page.
9. Leer body.innerText().
10. Parsear Claude.
11. Fusionar resultados.
12. Escribir status.json de forma atómica.
```

## Escritura atómica

Nunca escribir directamente encima de `status.json`.

```txt
status.json.tmp → rename → status.json
```

Así el applet nunca lee un archivo cortado.

## Configuración sugerida

```json
{
  "refresh_seconds": 180,
  "primary": {
    "provider": "codex",
    "metric": "five_hour"
  },
  "providers": {
    "codex": {
      "enabled": true,
      "url": "https://chatgpt.com/codex/cloud/settings/analytics#usage",
      "profile_dir": "~/.config/ai-usage-status/profiles/chatgpt",
      "headless": true,
      "wait_ms": 8000
    },
    "claude": {
      "enabled": true,
      "url": "https://claude.ai/settings/usage",
      "profile_dir": "~/.config/ai-usage-status/profiles/claude",
      "headless": true,
      "wait_ms": 8000
    }
  }
}
```

## Requisitos

```bash
sudo apt install nodejs npm jq
npm install playwright
npx playwright install chromium
```

En proyectos reales, usar `pnpm` o `npm` según prefieras.

## Frecuencia de actualización

Recomendado:

```txt
cada 3 a 5 minutos
```

No es necesario actualizar cada pocos segundos porque los límites no cambian visualmente tan rápido y se evita cargar páginas constantemente.

## Datos raw

Durante desarrollo puedes guardar texto de depuración en:

```txt
~/.cache/ai-usage-status/debug-codex-text.txt
~/.cache/ai-usage-status/debug-claude-text.txt
```

Pero en versión final debe estar desactivado por defecto.

## Fallback con último estado válido

Si el collector falla, puede mantener `last_good_status.json`.

Regla:

```txt
si falla scraping pero existe último estado válido con menos de 30 min:
  status.json.ok = false
  status.json.stale = true
  status.json.primary = último primary válido
```

Así el topbar sigue mostrando algo útil, pero el panel indica que está desactualizado.
