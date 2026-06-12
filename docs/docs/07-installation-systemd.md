# 07 — Instalación y systemd user timer

## Instalación del collector

Ubicación sugerida:

```txt
~/apps/ai-usage-status/
```

Binarios/symlinks sugeridos:

```txt
~/.local/bin/ai-usage-auth
~/.local/bin/ai-usage-collect
```

## Timer systemd

Crear:

```txt
~/.config/systemd/user/ai-usage-collector.service
~/.config/systemd/user/ai-usage-collector.timer
```

Contenido base incluido en `/systemd`.

## Activación

```bash
systemctl --user daemon-reload
systemctl --user enable --now ai-usage-collector.timer
systemctl --user list-timers | grep ai-usage
```

## Ejecutar manualmente

```bash
ai-usage-collect
cat ~/.cache/ai-usage-status/status.json | jq
```

## Logs

```bash
journalctl --user -u ai-usage-collector.service -n 100 --no-pager
```

## Instalación applet

Basado en el patrón de `visual-mute`:

```bash
just
just install-user
pkill cosmic-panel
```

Desktop id sugerido:

```txt
com.github.brandonhc33.ai-usage
```

## Archivo desktop

Debe incluir:

```ini
X-CosmicApplet=true
X-CosmicHoverPopup=Auto
```

## Panel plugin

Si no aparece automáticamente, revisar configuración del panel COSMIC y agregar el desktop id correspondiente.
