---
title: Bedienung
labels: [wayfinder:grilling]
status: open
assignee:
blocked_by: []
---

# Bedienung

## Question

Über welche Wege bedient man die App: Klick aufs Overlay, Rechtsklick-Kontextmenü, globale Hotkeys, Tray-Icon? Welche Aktion liegt auf welchem Weg (Start/Pause, Überspringen, Zurücksetzen, Immer-im-Vordergrund umschalten, Theme wechseln, Beenden)?

## Kontext

- Aus [GUI-Technik für das Overlay](01-gui-technik.md): Ist Click-through aktiv, erreicht die Maus das Overlay nicht mehr. Mindestens ein Weg (globaler Hotkey oder Tray-Icon) muss dann ohne Overlay-Klick funktionieren.

> Kontext aus *Overlay-Spike mit der gewählten Technik*: Click-through funktioniert technisch. Ist es aktiv, erreicht die Maus das Overlay aber nicht mehr, deshalb braucht es dann einen anderen Weg zurück (Hotkey, Tray, Zeitablauf). Falls Click-through in v1 gewollt ist, gehört das hier mit entschieden.

> Kontext aus *[Phasenablauf](03-phasenablauf.md)*: Diese Ablauf-Aktionen brauchen einen Weg in der Bedienung: Starten (aus *bereit*), Anhalten/Fortsetzen, Überspringen, **Phase zurücksetzen** und **Neu beginnen**. Phase zurücksetzen und Neu beginnen sind getrennte Aktionen.
