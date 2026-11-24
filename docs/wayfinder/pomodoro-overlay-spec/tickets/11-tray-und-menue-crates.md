---
title: Crates für Tray-Icon und gemeinsames Menü
labels: [wayfinder:research]
status: open
assignee: ckr
blocked_by: []
---

# Crates für Tray-Icon und gemeinsames Menü

## Question

Mit welchem Crate bekommt eine Rust-App mit `eframe`/`egui` 0.37 (+ `glow`, winit) unter Windows 11 ein Tray-Icon mit **dynamischem Tooltip** (sekündlich aktualisierte Restzeit) und ein **Menü**, das sich sowohl per Links-/Rechtsklick aufs Tray-Icon als auch per Rechtsklick aufs Overlay öffnen lässt, also **dasselbe Menü an beiden Stellen**? Das Menü braucht Häkchen-Einträge, ausgegraute Einträge, Trenner, ein Untermenü (*Theme ▸*) und zur Laufzeit wechselnden Eintragstext (Primäraktion). Kandidaten: `tray-icon` + `muda` (tauri), `trayicon`, direkt `windows`-rs (`Shell_NotifyIconW`, `TrackPopupMenu`). Zu prüfen: Verträgt sich das Crate mit der Event-Loop von eframe/winit (eigener Thread, Message-Pump)? Kann das native Menü auch an der Mausposition über dem Overlay aufgehen, oder muss das Overlay-Menü in egui nachgebaut werden? Funktioniert es mit Tastatur (`Esc`, `Umschalt+F10`/`Menütaste`)? Bewertet nach: Zuverlässigkeit auf Windows 11, Aufwand, Binärgröße. Ergebnis: Empfehlung mit Begründung.

## Kontext

- Aus *[Bedienung](04-bedienung.md)*: Menüinhalt, Ausgrau-Regel, Tooltip-Inhalt und Tastenkürzel sind dort festgelegt. Bei aktivem Click-through oder ausgeblendetem Overlay ist das Tray-Icon der einzige Weg zurück, deshalb muss es zuverlässig sein.
- Aus *[Overlay-Spike mit der gewählten Technik](02-overlay-spike.md)*: Spike-Code auf Branch `prototype/02-overlay-spike` zum Ausprobieren.
