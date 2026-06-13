#!/usr/bin/env bash
# Automated setup for the AI Usage COSMIC applet.
#
# Builds and installs the applet, prepares ~/.cache and ~/.config
# directories with the right permissions, publishes the collector scripts
# (and node) to ~/.local/bin, embeds the applet in the COSMIC panel, and
# enables the systemd collector timer.
#
# Steps that need interactive input (Chrome login, sudo) are printed at the
# end instead of being run automatically. Safe to re-run.

set -euo pipefail
cd "$(dirname "$0")"

for cmd in cargo just npm node; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: '$cmd' no está instalado. Revisá la sección 'Requisitos' del README." >&2
    exit 1
  fi
done

if ! command -v google-chrome >/dev/null 2>&1; then
  echo "advertencia: 'google-chrome' no encontrado. Los scripts usan channel:'chrome'" >&2
  echo "             y lo necesitan instalado (sudo apt install google-chrome-stable)." >&2
fi

echo "==> Compilando ai-usage-applet (release)"
just build-release

echo "==> Instalando el applet para el usuario actual"
just install-user

echo "==> Preparando directorios en ~/.cache y ~/.config"
mkdir -p ~/.cache/ai-usage-status
mkdir -p ~/.config/ai-usage-status/profiles/chatgpt
mkdir -p ~/.config/ai-usage-status/profiles/claude
chmod 700 ~/.cache/ai-usage-status
chmod 700 ~/.config/ai-usage-status
chmod 700 ~/.config/ai-usage-status/profiles
chmod 700 ~/.config/ai-usage-status/profiles/chatgpt
chmod 700 ~/.config/ai-usage-status/profiles/claude

if [ ! -f ~/.config/ai-usage-status/config.json ]; then
  echo "==> Copiando configuración por defecto del colector"
  cp templates/config.example.json ~/.config/ai-usage-status/config.json
fi

echo "==> Instalando dependencias del colector (npm + Playwright Chromium)"
(cd scripts && npm install && npx playwright install chromium)

echo "==> Publicando scripts en ~/.local/bin"
mkdir -p ~/.local/bin
ln -sf "$(pwd)/scripts/ai-usage-collect.mjs" ~/.local/bin/ai-usage-collect
ln -sf "$(pwd)/scripts/ai-usage-auth.mjs" ~/.local/bin/ai-usage-auth

# El panel de COSMIC lanza el applet con un PATH reducido que normalmente no
# incluye el node de nvm; sin este enlace "Actualizar ahora" falla en
# silencio (ver sección "Publicar los scripts en PATH" del README).
ln -sf "$(command -v node)" ~/.local/bin/node

echo "==> Integrando el applet en el panel de COSMIC"
panel_cfg="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
if [ -f "$panel_cfg" ]; then
  if grep -q "com.github.brandonhc33.ai-usage" "$panel_cfg"; then
    echo "    ya estaba presente en $panel_cfg"
  else
    sed -i '/^\]))$/i\    "com.github.brandonhc33.ai-usage",' "$panel_cfg"
    echo "    agregado a $panel_cfg"
  fi
  pkill cosmic-panel || true
else
  echo "    no se encontró $panel_cfg"
  echo "    agregá \"AI Usage\" manualmente desde Configuración del sistema >"
  echo "    Panel de escritorio > Configurar aplicaciones del panel"
fi

echo "==> Habilitando el timer de systemd del colector"
mkdir -p ~/.config/systemd/user
cp systemd/ai-usage-collector.service systemd/ai-usage-collector.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ai-usage-collector.timer

cat <<'EOF'

==> Listo. Pasos manuales pendientes:

1. Abrí el popup del applet y presioná "Iniciar sesión Codex" e
   "Iniciar sesión Claude" para autenticarte (se abre una ventana de Chrome
   visible, una vez por proveedor).
2. (opcional) Para que "Actualizar ahora" y el timer corran sin abrir una
   ventana de Chrome visible:
     sudo apt install xvfb
EOF
