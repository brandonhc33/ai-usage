# AI Usage — applet COSMIC

Applet para la barra superior de COSMIC (Pop!_OS 24) que muestra el porcentaje
de uso de Codex/ChatGPT y Claude, leyendo el archivo
`~/.cache/ai-usage-status/status.json`. Ese archivo lo escribe un colector
externo en Node.js (Playwright) que **el applet nunca ejecuta directamente**
salvo cuando el usuario pulsa "Actualizar ahora" (lo lanza como proceso aparte).

El diseño completo está documentado en [`docs/`](docs/). Este README resume
cómo compilar, instalar y probar el resultado.

## Estructura del proyecto

```
src/            Código del applet (Rust / libcosmic / iced)
resources/      Iconos, .desktop y metainfo para la instalación
scripts/        Colector Node.js + Playwright (ai-usage-auth, ai-usage-collect)
systemd/        Unidades de usuario para correr el colector periódicamente
templates/      Ejemplo de configuración del colector
docs/           Especificación y contexto original del proyecto
```

## Requisitos

- Pop!_OS 24 / COSMIC con `cosmic-panel`.
- Rust estable (`rustup`), `cargo`.
- Dependencias de sistema de libcosmic (las habituales para compilar apps
  COSMIC: `just`, `pkg-config`, librerías de Wayland/GTK que ya trae Pop!_OS 24).
- Node.js 18+ y npm (solo para el colector, no para el applet).

## Instalación automática (desde 0)

`install.sh` hace todo lo de las secciones 1-5 de una sola vez: compila e
instala el applet, crea `~/.cache/ai-usage-status` y
`~/.config/ai-usage-status` (con perfiles `chatgpt`/`claude` y permisos
700), instala las dependencias del colector, publica `ai-usage-collect`,
`ai-usage-auth` y `node` en `~/.local/bin`, integra el applet en el panel de
COSMIC y habilita el timer de systemd. Es seguro volver a correrlo.

```bash
./install.sh
```

Al final del script quedan dos pasos manuales que no se pueden automatizar:

1. Iniciar sesión en Codex y Claude desde el popup del applet (botones
   "Iniciar sesión Codex/Claude"), porque requiere completar el login en una
   ventana de Chrome.
2. Opcional: `sudo apt install xvfb` para que las actualizaciones en segundo
   plano no abran una ventana de Chrome visible (ver
   [Cloudflare y modo headless](#cloudflare-y-modo-headless)).

El resto de este README documenta esos mismos pasos en detalle, por si
preferís hacerlos a mano o algo falla.

## 1. Compilar el applet

```bash
cargo check          # verificación rápida
just                  # equivalente a `just build-release` (cargo build --release)
```

El binario queda en `target/release/ai-usage-applet`.

## 2. Probar en modo desarrollo

Antes de instalar, podés generar un `status.json` falso para ver el applet
con datos sin necesidad del colector real:

```bash
mkdir -p ~/.cache/ai-usage-status
cd scripts
npm install
npm run fake-status   # escribe ~/.cache/ai-usage-status/status.json de ejemplo
cd ..
```

Luego corré el applet:

```bash
just run
```

> Nota: como applet de panel, `just run` abre una ventana standalone con el
> contenido; para verlo integrado en la barra superior real hace falta
> instalarlo (paso 3) y agregarlo desde la configuración del panel COSMIC.

## 3. Instalar en el panel COSMIC (usuario actual)

```bash
just build-release
just install-user
```

Esto instala:

- `~/.local/bin/ai-usage-applet`
- `~/.local/share/applications/com.github.brandonhc33.ai-usage.desktop`
- `~/.local/share/metainfo/com.github.brandonhc33.ai-usage.metainfo.xml`
- `~/.local/share/icons/hicolor/scalable/apps/com.github.brandonhc33.ai-usage.svg`

Reiniciá el panel para que lo detecte:

```bash
pkill cosmic-panel
```

Luego, en **Configuración del sistema → Panel de escritorio (Desktop) →
Configurar aplicaciones del panel**, agregá "AI Usage" a la zona deseada.

### Desinstalar

```bash
just uninstall-user
```

## 4. Configurar el colector (Codex/Claude)

El colector vive en `scripts/` y es un proceso Node/Playwright separado que
el applet lanza como subproceso (nunca scrapea desde el binario Rust).

```bash
cd scripts
npm install
npx playwright install chromium
```

### Publicar los scripts en `PATH`

El applet invoca `ai-usage-auth` y `ai-usage-collect` por nombre, así que
deben estar en el `PATH` del usuario:

```bash
mkdir -p ~/.local/bin
ln -sf "$(pwd)/ai-usage-collect.mjs" ~/.local/bin/ai-usage-collect
ln -sf "$(pwd)/ai-usage-auth.mjs" ~/.local/bin/ai-usage-auth
```

(Asegurate de que `~/.local/bin` esté en tu `PATH`.)

Estos scripts usan `#!/usr/bin/env node`. El panel de COSMIC lanza el applet
con un `PATH` reducido que normalmente **no incluye** el `node` instalado vía
nvm, así que `env` no lo encuentra y "Actualizar ahora" falla en silencio. Si
usás nvm, enlazá también `node` dentro de `~/.local/bin`:

```bash
ln -sf "$(command -v node)" ~/.local/bin/node
```

### Autenticación inicial (una vez por proveedor, desde el applet)

En el popup del applet, los botones **"Iniciar sesión Codex"** e
**"Iniciar sesión Claude"** abren un Chromium visible apuntando al panel de
uso correspondiente. Iniciá sesión normalmente; el script detecta
automáticamente cuando el panel de uso terminó de cargar, guarda la sesión en
`~/.config/ai-usage-status/profiles/` y cierra la ventana solo. No hace falta
volver a una terminal ni presionar nada más.

También se puede hacer desde la línea de comandos:

```bash
node scripts/ai-usage-auth.mjs codex
node scripts/ai-usage-auth.mjs claude
```

### Ejecutar el colector

Después de iniciar sesión, presioná **"Actualizar ahora"** en el popup (o
manualmente con `node scripts/ai-usage-collect.mjs`). Esto scrapea ambos
paneles con la sesión guardada y escribe
`~/.cache/ai-usage-status/status.json`, que el applet vuelve a leer a los
pocos segundos.

### Cloudflare y modo headless

Tanto `ai-usage-auth` como `ai-usage-collect` usan el Chrome real del sistema
(`channel: 'chrome'`) en vez del Chromium de Playwright, porque Cloudflare
bloquea con un desafío "Verify you are human" al Chromium automatizado y
también al Chrome **headless** (incluso con los flags anti-automatización).
Headed funciona sin problemas.

- `ai-usage-auth` siempre corre headed (necesitás ver la ventana para iniciar
  sesión).
- `ai-usage-collect` (botón "Actualizar ahora" y el timer systemd) corre
  headless por defecto, lo cual **falla con Cloudflare**. Para que funcione
  sin abrir una ventana visible en cada actualización, instalá `xvfb`:

  ```bash
  sudo apt install xvfb
  ```

  Si `Xvfb` está disponible, el colector lo detecta automáticamente y corre
  Chrome en modo headed dentro de un display virtual (invisible para el
  usuario, pero "headed" para Cloudflare). Sin `xvfb`, podés alternativamente
  poner `"headless": false` en `config.json` para el proveedor afectado, pero
  eso abre una ventana de Chrome visible en cada actualización.

### Configuración del colector

Copiá la plantilla y ajustala si querés cambiar el proveedor/métrica
principal, umbrales o intervalos:

```bash
mkdir -p ~/.config/ai-usage-status
cp templates/config.example.json ~/.config/ai-usage-status/config.json
```

## 5. Ejecutar el colector periódicamente (systemd)

```bash
mkdir -p ~/.config/systemd/user
cp systemd/ai-usage-collector.service systemd/ai-usage-collector.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ai-usage-collector.timer
```

Verificar:

```bash
systemctl --user list-timers | grep ai-usage
journalctl --user -u ai-usage-collector.service
```

Por defecto corre 30s después del login y luego cada 2 minutos
(`systemd/ai-usage-collector.timer`).

## Permisos y privacidad

Los perfiles de Chromium contienen cookies de sesión y deben tratarse como
sensibles:

```bash
chmod 700 ~/.config/ai-usage-status
chmod 700 ~/.config/ai-usage-status/profiles
chmod 700 ~/.config/ai-usage-status/profiles/chatgpt
chmod 700 ~/.config/ai-usage-status/profiles/claude
chmod 700 ~/.cache/ai-usage-status
chmod 600 ~/.cache/ai-usage-status/*.json 2>/dev/null || true
```

Nada de esto se versiona: `.gitignore` excluye `status.json`, perfiles,
archivos de debug y `node_modules`. Más detalle en
[`docs/docs/08-security-and-privacy.md`](docs/docs/08-security-and-privacy.md).

## Estados del applet

- **`AI --`**: no existe `status.json` (colector nunca corrió).
- **`!`**: `status.json` existe pero no se puede parsear.
- Icono + porcentaje: color según uso (verde 0-50% / naranja 51-75% /
  rojo ≥76%).
- Sufijo `!`: datos desactualizados (>15 min) o con errores.
- `login`: el proveedor principal requiere volver a iniciar sesión (usá el
  botón "Iniciar sesión Codex/Claude" del popup).

El popup (click en el applet) muestra el detalle de Codex y Claude, errores
por proveedor con sugerencias, y botones para:

- **Actualizar ahora**: corre el colector con la sesión guardada.
- **Iniciar sesión Codex / Claude**: abre Chromium para (re)loguearse.
- **Panel Codex / Claude**: abre el panel de uso real en el navegador.

## Checklist de pruebas

Ver [`docs/docs/13-testing-checklist.md`](docs/docs/13-testing-checklist.md)
para la lista completa (colector, applet, systemd e instalación COSMIC).

## Desarrollo

```bash
just check        # cargo clippy --all-features -W clippy::pedantic
cargo check        # verificación rápida de tipos
just build-debug   # build de desarrollo
```

## Licencia

MIT (ver `Cargo.toml` / encabezados `SPDX-License-Identifier: MIT`).
