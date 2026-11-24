---
title: Einstellungen und Speichern
labels: [wayfinder:grilling]
status: open
assignee:
blocked_by: [04]
---

# Einstellungen und Speichern

## Question

Wo und wie werden Einstellungen (Dauern, Theme, Immer-im-Vordergrund, Position, …) gespeichert (z. B. TOML unter `%APPDATA%`), werden sie nur per Datei bearbeitet oder auch über eine Oberfläche, und werden Änderungen zur Laufzeit übernommen?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Zu den Einstellungen gehören die drei Dauern (ganze Minuten, 1 bis 180), *Fokusphasen pro Zyklus* (1 bis 12) und zwei Schalter für den automatischen Start (Fokus→Pause, Pause→Fokus). Bei ungültigen Werten gilt die Voreinstellung mit Hinweis; **wie** der Hinweis erscheint, wird hier entschieden. Geänderte Werte gelten erst ab der nächsten Phase dieses Typs bzw. ab dem nächsten Zyklus.

> Kontext aus *[Bedienung](04-bedienung.md)*: Das gemeinsame Menü enthält **Einstellungen …**. Was dieser Eintrag öffnet (Datei im Editor, eigenes Fenster), wird hier entschieden. Die Menü-Schalter *Immer im Vordergrund*, *Click-through* und *Overlay anzeigen* sind Kandidaten für gespeicherte Einstellungen.

> Kontext aus *[Inhalt und Verhalten des Overlays](05-overlay-anzeige.md)*: Zu speichern sind außerdem die **Position** des Overlays (virtuelle Bildschirmkoordinaten, nach jedem Verschieben) und die **Skalierung** (50 % bis 300 %, nach jeder Änderung per `Strg`+Mausrad). Einen Regler für die Größe gibt es nicht, sie wird nur über das Mausrad geändert.

> Kontext aus *[Hinweis bei Phasenwechsel](06-phasenwechsel-hinweis.md)*: Drei weitere Einstellungen: *Ton* (an), *Signal im Overlay* (an), *Overlay bei Phasenwechsel einblenden* (aus). *Ton* gibt es zusätzlich als Häkchen im gemeinsamen Menü, ändert man es dort, muss es gespeichert werden. Eine Lautstärke wird nicht gespeichert.
