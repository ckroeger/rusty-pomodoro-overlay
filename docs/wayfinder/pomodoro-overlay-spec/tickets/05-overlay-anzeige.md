---
title: Inhalt und Verhalten des Overlays
labels: [wayfinder:grilling]
status: open
assignee:
blocked_by: [04]
---

# Inhalt und Verhalten des Overlays

## Question

Was zeigt das Overlay (Restzeit-Format, Phase, Zyklus-Fortschritt, Fortschrittsbalken/-ring?), wie groß ist es, wo erscheint es beim Start, ist es verschiebbar und merkt es sich seine Position, und gibt es einen Click-through-Modus (und wie verträgt der sich mit der in *Bedienung* gewählten Steuerung)?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Es gibt drei Zustände (läuft, angehalten, bereit), die das Overlay unterscheidbar zeigen muss, dazu den Zyklus-Fortschritt (z. B. 2/4). Im Zustand *bereit* nach einer abgelaufenen Phase kommt die **Überzeit** dazu (z. B. „+07:00“). Wie sie aussieht, wird hier entschieden.

> Kontext aus *[Bedienung](04-bedienung.md)*: Click-through ist entschieden: ein Schalter, Voreinstellung aus, ausschalten nur über das Tray-Icon. Ein Linksklick ohne Bewegung löst die Primäraktion aus, Ziehen verschiebt das Overlay. Der Tray-Tooltip zeigt den Stand **im gleichen Format wie das Overlay**, das hier festgelegte Format gilt also für beide. Das Overlay kann *ausgeblendet* sein (Schalter „Overlay anzeigen“).
