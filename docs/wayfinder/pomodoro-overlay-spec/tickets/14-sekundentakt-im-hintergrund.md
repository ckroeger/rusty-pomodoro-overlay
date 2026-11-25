---
title: Sekundentakt bei ausgeblendetem Overlay
labels: [wayfinder:prototype]
status: open
assignee: ckr
blocked_by: []
---

# Sekundentakt bei ausgeblendetem Overlay

## Question

Läuft die App-Logik (`App::logic` von eframe 0.37 + glow) zuverlässig **jede Sekunde**, auch wenn das Overlay **ausgeblendet** ist (`ViewportCommand::Visible(false)`), das Einstellungsfenster zu ist und keine Eingaben kommen? Und reicht das für alles, was am Sekundentakt hängt? Am Takt hängen der Phasenablauf und der Hinweis, der Tray-Tooltip und das Tray-Icon, der Änderungszeitpunkt der Einstellungsdatei, das Event für den zweiten Start und das verzögerte Speichern von Position und Skalierung. Falls nicht: Welcher Taktgeber stattdessen (z. B. `request_repaint_after`, eigener Thread mit `EventLoopProxy` bzw. `ctx.request_repaint()`, `SetTimer` am Fenster)? Ergebnis: Ein Wegwerf-Prototyp auf Branch `prototype/14-sekundentakt` belegt das gewählte Vorgehen, mit gemessenen Abständen zwischen den Takten (ausgeblendet, sichtbar, offenes Menü, nach Standby).

## Kontext

- Aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: Bei offenem nativem Menü steht die Event-Loop. Die Restzeit wird darum aus `Instant`-Differenzen bzw. der Wanduhr berechnet, nicht gezählt. Tray-Code lebt in `App::logic`.
- Aus *[Phasenablauf](03-phasenablauf.md)*: Die Zeitbasis ist die Wanduhr, Standby verbraucht Zeit und wird beim Aufwachen nachgeholt.
- Aus *[Einstellungen und Speichern](08-einstellungen.md)* und *[Lebenszyklus der App](09-lebenszyklus.md)*: Änderungszeitpunkt der Datei und benanntes Event werden im Sekundentakt abgefragt.
- Spike-Code als Ausgangspunkt: Branch `prototype/02-overlay-spike`.
