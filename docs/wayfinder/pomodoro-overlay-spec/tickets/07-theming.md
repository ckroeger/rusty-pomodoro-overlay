---
title: Theming
labels: [wayfinder:grilling]
status: open
assignee:
blocked_by: [05]
---

# Theming

## Question

Was legt ein Theme fest (Farben je Phase, Hintergrund-Transparenz, Schrift, Größe?), wie viele Themes bringt v1 mit, kann man eigene Themes definieren, und wie wechselt man das Theme?

## Kontext

- Aus [GUI-Technik für das Overlay](01-gui-technik.md): Transparenz pro Pixel (Hintergrund durchsichtig, Text deckend) hängt von Renderer/GPU ab; eine einheitliche Fenster-Deckkraft (`SetLayeredWindowAttributes`) ist überall sicher. Entscheiden, ob einheitliche Deckkraft für v1 reicht. Ergebnis von [Overlay-Spike mit der gewählten Technik](02-overlay-spike.md) beachten.
