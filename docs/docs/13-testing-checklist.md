# 13 — Checklist de pruebas

## Pruebas del collector

- [ ] `ai-usage-auth codex` abre Chromium visible.
- [ ] Login Codex queda guardado.
- [ ] `ai-usage-auth claude` abre Chromium visible.
- [ ] Login Claude queda guardado.
- [ ] `ai-usage-collect` genera `status.json`.
- [ ] El archivo no contiene HTML completo.
- [ ] El archivo no contiene cookies.
- [ ] El archivo contiene `schema_version`.
- [ ] Codex parsea 5 horas.
- [ ] Codex parsea semanal.
- [ ] Codex parsea créditos.
- [ ] Claude parsea sesión actual.
- [ ] Claude parsea semanal.
- [ ] Claude parsea rutinas.

## Pruebas del applet

- [ ] Sin `status.json`, muestra `AI --`.
- [ ] Con JSON válido, muestra icono + porcentaje.
- [ ] Si `primary.provider=codex`, usa icono Codex.
- [ ] Si `primary.provider=claude`, usa icono Claude.
- [ ] Si `used_percent >= 75`, muestra estado advertencia.
- [ ] Si `used_percent >= 95`, muestra estado crítico.
- [ ] Click abre popup.
- [ ] Popup muestra Codex y Claude.
- [ ] Popup muestra errores.
- [ ] JSON inválido no crashea applet.
- [ ] Archivo viejo muestra alerta.

## Pruebas systemd

- [ ] Timer activo con `systemctl --user list-timers`.
- [ ] Servicio ejecuta collector.
- [ ] Logs visibles con `journalctl --user`.
- [ ] Al reiniciar sesión, timer vuelve a correr.

## Pruebas de instalación COSMIC

- [ ] `just` compila.
- [ ] `just install-user` instala desktop file.
- [ ] `X-CosmicApplet=true` está presente.
- [ ] Applet aparece en panel settings.
- [ ] `pkill cosmic-panel` reinicia panel sin romper sesión.
