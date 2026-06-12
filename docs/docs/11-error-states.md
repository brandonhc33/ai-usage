# 11 — Estados de error

## Errores del collector

| Código | Causa | Applet |
|---|---|---|
| `login_required` | Sesión vencida | `AI login` |
| `page_changed` | Parser no encontró textos | `AI !` |
| `network_error` | Sin internet o timeout | `AI !` con último dato si existe |
| `profile_missing` | No se ejecutó auth | `AI login` |
| `provider_disabled` | Proveedor apagado en config | no mostrar provider |

## Errores del applet

| Error | Topbar | Popup |
|---|---|---|
| status.json no existe | `AI --` | “Ejecuta ai-usage-collect” |
| JSON inválido | `AI !` | “status.json inválido” |
| Archivo viejo | `AI 26%` con alerta | “Datos desactualizados” |
| Sin provider primario | `AI --` | “Configura provider primario” |

## Stale data

Si `updated_at` tiene más de 15 minutos:

```txt
mostrar porcentaje, pero con indicador de alerta
```

Ejemplo:

```txt
AI 26% !
```

Popup:

```txt
Última actualización: hace 28 min
El collector podría estar detenido.
```

## Login vencido

Popup:

```txt
Codex requiere iniciar sesión nuevamente.
Ejecuta:

ai-usage-auth codex
```

## Parser roto

Popup:

```txt
No pude encontrar los textos esperados en la página.
Puede que el layout o idioma haya cambiado.
Ejecuta el collector con debug para revisar el texto visible.
```
