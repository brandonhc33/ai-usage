# 01 — Visión del producto

## Qué se quiere construir

Un applet nativo para el topbar de COSMIC en Pop!_OS 24 que muestre el estado de uso de IA para:

- Codex / ChatGPT
- Claude

El applet debe mostrar de forma rápida cuánto uso queda o cuánto se ha consumido, pero normalizado como **porcentaje de uso**.

## Comportamiento visual esperado

En el topbar:

```txt
[icono color] 26%
```

Variantes posibles:

```txt
[AI] 26%
[Codex] 100%
[Claude] 13%
```

El porcentaje visible debe ser configurable. Por defecto:

```txt
provider activo: codex
métrica principal: ventana de 5 horas
mostrar: used_percent
```

## Click / panel desplegable

Al hacer click en el applet, mostrar un panel compacto con:

### Codex

- Límite de 5 horas
- Porcentaje usado
- Porcentaje restante
- Hora de reinicio
- Límite semanal
- Porcentaje usado
- Porcentaje restante
- Fecha/hora de reinicio
- Créditos restantes

### Claude

- Sesión actual
- Porcentaje usado
- Porcentaje restante
- Tiempo de reinicio
- Límite semanal / todos los modelos
- Porcentaje usado
- Porcentaje restante
- Tiempo de reinicio
- Rutinas diarias incluidas

## No objetivos

El applet no debe:

- Guardar contraseñas.
- Leer chats o conversaciones.
- Guardar HTML completo.
- Hacer scraping directamente desde el proceso del applet.
- Bloquear el panel si falla la red.
- Depender de que Codex CLI esté abierto.

## UX esperada

Estados rápidos:

| Estado | Topbar | Significado |
|---|---|---|
| Normal | `AI 26%` | datos actualizados |
| Alto uso | `AI 82%` | cerca del límite |
| Límite | `AI 100%` | sin uso restante en ventana principal |
| Login vencido | `AI login` | requiere reautenticar |
| Sin datos | `AI --` | no existe status.json |
| Error | `AI !` | error del collector |

## Filosofía

La applet debe ser tan simple y directa como `visual-mute`: un indicador visible, una acción clara y cero ruido.
