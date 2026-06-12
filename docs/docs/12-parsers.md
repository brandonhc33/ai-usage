# 12 — Parsers

## Regla principal

El collector debe leer texto visible con:

```js
await page.locator('body').innerText()
```

Y después parsear con regex tolerantes a idioma español/inglés.

## Codex — textos esperados en español

```txt
Límite de uso de 5 horas
0 % restante
Se reinicia a las 4:39

Límite de uso semanal
74 % restante
Se reinicia a las 18 jun 2026 15:29

Créditos restantes
128
```

## Codex — normalización

```js
used_percent = 100 - remaining_percent
```

## Codex — regex inicial

```js
const fiveHour = text.match(
  /(Límite de uso de 5 horas|5-hour usage limit)\s+(\d+)\s*%\s*(restante|remaining).*?(Se reinicia a las|Resets)\s+(.+?)(?=Límite de uso semanal|Weekly usage limit|Créditos restantes|Credits remaining|$)/i
);

const weekly = text.match(
  /(Límite de uso semanal|Weekly usage limit)\s+(\d+)\s*%\s*(restante|remaining).*?(Se reinicia a las|Resets)\s+(.+?)(?=Créditos restantes|Credits remaining|$)/i
);

const credits = text.match(
  /(Créditos restantes|Credits remaining)\s+(\d+)/i
);
```

## Claude — textos esperados en español

```txt
Sesión actual
Se restablece en 2 h 33 min
13 % usado

Todos los modelos
Se restablece mié., 11:59 p.m.
1 % usado

Ejecuciones de rutinas diarias incluidas
0 / 5
```

## Claude — normalización

```js
remaining_percent = 100 - used_percent
```

## Claude — regex inicial

```js
const session = text.match(
  /(Sesión actual|Current session).*?(Se restablece en|Resets in)\s+(.+?)\s+(\d+)\s*%\s*(usado|used)/i
);

const weekly = text.match(
  /(Todos los modelos|All models).*?(Se restablece|Resets)\s+(.+?)\s+(\d+)\s*%\s*(usado|used)/i
);

const routines = text.match(
  /(Ejecuciones de rutinas diarias incluidas|Daily routine executions included).*?(\d+)\s*\/\s*(\d+)/i
);
```

## Limpieza de texto

```js
function normalizeText(raw) {
  return raw
    .replace(/\u00a0/g, ' ')
    .replace(/[ \t]+/g, ' ')
    .replace(/\n+/g, '\n')
    .trim()
    .replace(/\n/g, ' ');
}
```

## Tests mínimos

Crear tests unitarios con:

```txt
templates/codex-page-text-sample.txt
templates/claude-page-text-sample.txt
```

El parser debe devolver:

```json
{
  "codex": {
    "five_hour": { "used_percent": 100, "remaining_percent": 0 },
    "weekly": { "used_percent": 26, "remaining_percent": 74 },
    "credits_remaining": 128
  }
}
```

```json
{
  "claude": {
    "session": { "used_percent": 13, "remaining_percent": 87 },
    "weekly": { "used_percent": 1, "remaining_percent": 99 },
    "daily_routines": { "used": 0, "limit": 5 }
  }
}
```
