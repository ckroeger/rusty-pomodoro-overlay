---
title: Ton abspielen und „nicht stören“ erkennen
labels: [wayfinder:research]
status: closed
assignee: ckr
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

## Resolution

**Ton: `PlaySoundW` aus `windows-sys` 0.61 (Feature `Win32_Media_Audio`) mit `SND_MEMORY | SND_ASYNC | SND_NODEFAULT`, ohne `SND_SYSTEM`. Kein `rodio`.**
- Der Puffer ist `static BEEP: &[u8] = include_bytes!(…)`. Das erfüllt die Bedingung von Microsoft, dass der Speicher bei `SND_MEMORY | SND_ASYNC` bis zum Ende des Tons gültig bleibt.
- Ohne `SND_SYSTEM` landet der Ton in der Audiositzung des Prozesses, also unter der App im Lautstärkemixer (mit dem Flag wäre es der Regler *Systemsounds*). Ein neuer Ton stoppt einen noch laufenden, das passt.
- Gemessen: +17 KB gegenüber +652 KB und 49 Crates für `rodio` 0.23 (nur `playback` + `wav`). Erster Aufruf kehrt nach 22 ms zurück, jeder weitere nach ~16 µs. Ein eigener Thread ist nicht nötig. Rückgabewert (`FALSE` ohne Audiogerät) wird ignoriert.

**„Nicht stören“: direkt vor dem Ton `SHQueryUserNotificationState` (`windows-sys`, Feature `Win32_UI_Shell`, ~0,08 ms) abfragen. Kein Ton bei `QUNS_BUSY` (Vollbild-App oder Präsentationseinstellungen), `QUNS_RUNNING_D3D_FULL_SCREEN` und `QUNS_PRESENTATION_MODE`, sonst spielen (auch bei Fehler).**
- Das eigentliche **Bitte nicht stören von Windows 11 ist nicht dokumentiert abfragbar** und wird darum nach der Regel aus *Hinweis bei Phasenwechsel* ignoriert. `QUNS_QUIET_TIME` meint nur die erste Stunde nach Ersteinrichtung oder OS-Upgrade, nicht *Bitte nicht stören*. Erkannt werden also nur einige Situationen, in denen Windows es automatisch einschaltet (Vollbild, Spiel), nicht der Schalter selbst.
- `QUNS_NOT_PRESENT` (gesperrt, Bildschirmschoner), `QUNS_QUIET_TIME` und `QUNS_APP` unterdrücken nicht: abwesend heißt nicht beschäftigt (vom Nutzer bestätigt).
- Für Vollbild gibt es keine Änderungsnachricht, also jedes Mal neu fragen und nicht zwischenspeichern.
- **Verworfen:** `FocusSessionManager.IsFocusActive` (WinRT, dokumentiert, läuft auch ohne Paket, ~0,7 ms) meldet nur *Fokussitzungen* und bräuchte zusätzlich das `windows`-Crate, bleibt Option für später. `UserNotificationListener` hat keinen solchen Zustand. WNF ist undokumentiert.

**Risiken / nicht geprüft:** Ob randlose Vollbildfenster (Browser-Video, PowerPoint-Präsentation) und *Bildschirm duplizieren* wirklich `QUNS_BUSY` bzw. einen Sperr-Zustand liefern, ist nicht gemessen (der Test lief bei gesperrter Sitzung). Den Eintrag im Lautstärkemixer und einen Gerätewechsel während des Tons habe ich nicht angesehen.

Details: [Recherche](../research/12-ton-und-nicht-stoeren.md)
