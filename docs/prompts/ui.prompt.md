# Prompt — Diseñar UI popup AI Usage

Diseña la UI de un popup para un COSMIC applet que muestra uso de IA.

Topbar:

```txt
[icono provider] 26%
```

Popup:

```txt
AI Usage
Actualizado hace 2 min

Codex
5 horas    100% usado    reinicia 4:39
Semanal     26% usado    reinicia 18 jun 2026 15:29
Créditos    128

Claude
Sesión      13% usado     reinicia en 2 h 33 min
Semanal      1% usado     reinicia mié., 11:59 p.m.
Rutinas      0 / 5
```

Estilo:

- Minimalista.
- Similar a applet de sistema.
- Iconos a color.
- Usar color de provider cuando está normal.
- Amarillo si uso >= 75%.
- Rojo si uso >= 95%.
- Gris si no hay datos.

Estados:

- `AI --`: sin archivo.
- `AI !`: error.
- `AI login`: sesión vencida.
- `AI 26%!`: datos viejos/error parcial.

No agregar gráficos complejos en la primera versión.
