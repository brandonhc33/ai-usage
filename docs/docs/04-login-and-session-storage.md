# 04 — Login y guardado de sesión

## Principio principal

El login se hace **una vez, manualmente**, en una ventana Chromium de Playwright.

El applet no maneja login.

## Comandos deseados

```bash
ai-usage-auth codex
ai-usage-auth claude
```

## Perfiles persistentes

Codex:

```txt
~/.config/ai-usage-status/profiles/chatgpt
```

Claude:

```txt
~/.config/ai-usage-status/profiles/claude
```

Playwright debe usar `launchPersistentContext`, no `browser.newContext()`.

## Flujo de auth

```txt
1. El usuario ejecuta ai-usage-auth codex.
2. Se abre Chromium visible.
3. El usuario inicia sesión.
4. El usuario navega o espera hasta el panel de uso.
5. Vuelve a la terminal y presiona Enter.
6. El script valida que aparezcan textos esperados.
7. Cierra Chromium.
8. Las cookies quedan guardadas en el perfil persistente.
```

## Validación mínima para Codex

Buscar alguno de estos textos:

```txt
Límite de uso
usage limit
Créditos restantes
remaining credits
```

## Validación mínima para Claude

Buscar alguno de estos textos:

```txt
Sesión actual
Current session
Todos los modelos
All models
Usage
```

## Qué se guarda

Sí se guarda:

- Cookies de sesión.
- LocalStorage/IndexedDB si el sitio lo usa.
- Perfil Chromium local.

No se guarda:

- Contraseña.
- HTML completo.
- Capturas de pantalla.
- Conversaciones.
- Prompts.

## Permisos recomendados

```bash
mkdir -p ~/.config/ai-usage-status/profiles
chmod 700 ~/.config/ai-usage-status
chmod 700 ~/.config/ai-usage-status/profiles
chmod 700 ~/.config/ai-usage-status/profiles/chatgpt
chmod 700 ~/.config/ai-usage-status/profiles/claude

mkdir -p ~/.cache/ai-usage-status
chmod 700 ~/.cache/ai-usage-status
```

## Expiración de sesión

Si el collector detecta login vencido, debe escribir:

```json
{
  "provider": "codex",
  "code": "login_required",
  "message": "Ejecuta ai-usage-auth codex"
}
```

El applet debe mostrar:

```txt
AI login
```

## Consideración sobre headless

Puede que algunas páginas se comporten distinto en headless. Se recomienda:

- Auth siempre `headless: false`.
- Collector primero probar con `headless: true`.
- Si falla, permitir `headless: false` desde config.
