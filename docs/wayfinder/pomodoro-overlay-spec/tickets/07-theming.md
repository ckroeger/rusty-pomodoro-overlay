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

> Kontext aus *[Bedienung](04-bedienung.md)*: Das gemeinsame Menü (Overlay und Tray) enthält einen Eintrag **Theme ▸**. Wie das Untermenü aussieht (Liste der Themes mit Häkchen?), wird hier entschieden.

> Kontext aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: Das gemeinsame Menü ist ein **natives Windows-Menü** (muda). Es bleibt hell und folgt dem Theme nicht. Theming betrifft also nur das Overlay, optional dazu ein Tray-Icon pro Phase.

> Kontext aus *[Inhalt und Verhalten des Overlays](05-overlay-anzeige.md)*: Das Layout ist in v1 **fest**. Ein Theme ändert nur das Aussehen (Farben, Transparenz, ggf. Schrift), nicht Anordnung oder Elemente. Die **Größe gehört nicht zum Theme**, sie wird per `Strg`+Mausrad eingestellt. Ein Theme braucht mindestens: eine **Akzentfarbe je Phase** (Fokus, kurze Pause, lange Pause; daran wird die Phase erkannt, auch bei Fortschrittsbalken und hervorgehobenem Zyklus-Punkt), eine Darstellung für **gedimmt** (*angehalten*, *bereit*), Text-, Hintergrund- und Punktfarben. Die Restzeit verwendet Ziffern gleicher Breite, was bei der Schriftwahl zu beachten ist.
