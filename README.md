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

El colector vive en `scripts/` y es independiente del applet.

```bash
cd scripts
npm install
npx playwright install chromium
```

### Autenticación inicial (una vez por proveedor)

Abre un Chromium visible para iniciar sesión; la sesión queda guardada en un
perfil persistente dentro de `~/.config/ai-usage-status/profiles/`.

```bash
node ai-usage-auth.mjs codex
node ai-usage-auth.mjs claude
```

### Ejecutar el colector manualmente

```bash
node ai-usage-collect.mjs
```

Esto actualiza `~/.cache/ai-usage-status/status.json`. El botón "Actualizar
ahora" del popup ejecuta lo mismo, pero requiere que los scripts estén
accesibles en el `PATH` del usuario (ver siguiente sección).

### Publicar los scripts en `PATH`

```bash
mkdir -p ~/.local/bin
ln -sf "$(pwd)/ai-usage-collect.mjs" ~/.local/bin/ai-usage-collect
ln -sf "$(pwd)/ai-usage-auth.mjs" ~/.local/bin/ai-usage-auth
```

(Asegurate de que `~/.local/bin` esté en tu `PATH`.)

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

Por defecto corre 30s después del login y luego cada 3 minutos
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
- Icono + porcentaje: estado normal, color según severidad
  (normal / advertencia ≥75% / crítico ≥95%).
- Sufijo `!`: datos desactualizados (>15 min) o con errores.
- `login`: el proveedor principal requiere volver a iniciar sesión
  (`ai-usage-auth <provider>`).

El popup (click en el applet) muestra el detalle de Codex y Claude, errores
por proveedor con sugerencias, y botones para forzar una actualización o
abrir los paneles de uso en el navegador.

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
