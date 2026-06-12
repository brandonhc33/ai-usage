# 03 — Contrato de datos: status.json

El applet debe depender de un único contrato:

```txt
~/.cache/ai-usage-status/status.json
```

## Ejemplo completo

```json
{
  "schema_version": 1,
  "updated_at": "2026-06-12T05:50:00.000Z",
  "ok": true,
  "primary": {
    "provider": "codex",
    "metric": "five_hour",
    "used_percent": 100,
    "remaining_percent": 0,
    "label": "100%"
  },
  "codex": {
    "ok": true,
    "source": "playwright",
    "updated_at": "2026-06-12T05:50:00.000Z",
    "five_hour": {
      "used_percent": 100,
      "remaining_percent": 0,
      "reset_label": "4:39",
      "reset_epoch": null
    },
    "weekly": {
      "used_percent": 26,
      "remaining_percent": 74,
      "reset_label": "18 jun 2026 15:29",
      "reset_epoch": null
    },
    "credits_remaining": 128
  },
  "claude": {
    "ok": true,
    "source": "playwright",
    "updated_at": "2026-06-12T05:50:00.000Z",
    "session": {
      "used_percent": 13,
      "remaining_percent": 87,
      "reset_label": "en 2 h 33 min",
      "reset_epoch": null
    },
    "weekly": {
      "used_percent": 1,
      "remaining_percent": 99,
      "reset_label": "mié., 11:59 p.m.",
      "reset_epoch": null
    },
    "daily_routines": {
      "used": 0,
      "limit": 5
    }
  },
  "errors": []
}
```

## Regla de normalización

Codex suele mostrar porcentaje restante:

```txt
74% restante
```

Claude suele mostrar porcentaje usado:

```txt
13% usado
```

El collector debe normalizar ambos a:

```json
{
  "used_percent": 26,
  "remaining_percent": 74
}
```

## Campo `primary`

El applet debe renderizar el porcentaje desde:

```json
primary.used_percent
```

Esto evita meter lógica de negocio en Rust.

Ejemplos:

```json
{
  "provider": "codex",
  "metric": "five_hour",
  "used_percent": 100,
  "label": "100%"
}
```

```json
{
  "provider": "claude",
  "metric": "session",
  "used_percent": 13,
  "label": "13%"
}
```

## Estados de error

```json
{
  "schema_version": 1,
  "updated_at": "2026-06-12T06:00:00.000Z",
  "ok": false,
  "primary": {
    "provider": "none",
    "metric": "none",
    "used_percent": null,
    "remaining_percent": null,
    "label": "--"
  },
  "codex": {
    "ok": false,
    "error_code": "login_required",
    "error_message": "Codex requiere iniciar sesión nuevamente."
  },
  "claude": {
    "ok": true,
    "session": {
      "used_percent": 13,
      "remaining_percent": 87,
      "reset_label": "en 2 h 33 min"
    }
  },
  "errors": [
    {
      "provider": "codex",
      "code": "login_required",
      "message": "Codex requiere iniciar sesión nuevamente."
    }
  ]
}
```

## Compatibilidad futura

No borrar campos viejos sin subir `schema_version`.
