---
title: Ton abspielen und „nicht stören“ erkennen
labels: [wayfinder:research]
status: open
assignee:
blocked_by: []
---

# Ton abspielen und „nicht stören“ erkennen

## Question

Zwei Faktenfragen für eine Rust-App mit `eframe`/`egui` 0.37 (+ `glow`, winit, `tray-icon` 0.26) unter Windows 11:

1. **Ton:** Wie spielt die App am einfachsten eine kleine, per `include_bytes!` eingebettete `.wav` (wenige KB) asynchron ab, ohne die Event-Loop zu blockieren? Kandidaten: `PlaySoundW` mit `SND_MEMORY | SND_ASYNC` über `windows-sys` (schon im Baum), `rodio`, andere. Bewertet nach Aufwand, Binärgröße, Zuverlässigkeit. Hält `SND_ASYNC` + `SND_MEMORY` den Puffer korrekt (muss `'static` sein)? Folgt der Ton der Windows-Lautstärke für die App (Lautstärkemixer)?
2. **„Nicht stören“:** Kann die App mit **dokumentierter** API zuverlässig erkennen, dass der Nutzer nicht gestört werden will: Windows-11-*Bitte nicht stören* (früher *Benachrichtigungsassistent*/*Focus Assist*), Präsentationsmodus/Bildschirm teilen, Vollbild-App (D3D-Vollbild, Vollbild-Video)? Kandidaten: `SHQueryUserNotificationState` (welche `QUNS_*`-Werte, deckt `QUNS_QUIET_TIME` *Bitte nicht stören* von Windows 11 ab?), WinRT `UserNotificationListener`/`FocusSessionManager`, WNF (undokumentiert, nur zur Abgrenzung). Ergebnis: Was geht dokumentiert, was nicht, wie teuer ist eine Abfrage beim Phasenwechsel.

Ergebnis: Empfehlung mit Begründung für beide Teile.

## Kontext

- Aus *[Hinweis bei Phasenwechsel](06-phasenwechsel-hinweis.md)*: Zwei eingebettete Klänge (Fokus endet / Pause endet), Windows-Lautstärke, kein eigener Regler. Bei „nicht stören“ wird nur der Ton unterdrückt, **nur wenn das per dokumentierter API geht**. Sonst wird der Zustand ignoriert.
- Aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: `windows-sys` 0.61 ist bereits im Abhängigkeitsbaum. Binärgröße zuletzt 4,68 MB (Release/LTO/strip).
- Spike-Code zum Ausprobieren: Branch `prototype/02-overlay-spike`.
