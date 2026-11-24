---
title: Bedienung
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: []
---

# Bedienung

## Question

Über welche Wege bedient man die App: Klick aufs Overlay, Rechtsklick-Kontextmenü, globale Hotkeys, Tray-Icon? Welche Aktion liegt auf welchem Weg (Start/Pause, Überspringen, Zurücksetzen, Immer-im-Vordergrund umschalten, Theme wechseln, Beenden)?

## Kontext

- Aus [GUI-Technik für das Overlay](01-gui-technik.md): Ist Click-through aktiv, erreicht die Maus das Overlay nicht mehr. Mindestens ein Weg (globaler Hotkey oder Tray-Icon) muss dann ohne Overlay-Klick funktionieren.

> Kontext aus *Overlay-Spike mit der gewählten Technik*: Click-through funktioniert technisch. Ist es aktiv, erreicht die Maus das Overlay aber nicht mehr, deshalb braucht es dann einen anderen Weg zurück (Hotkey, Tray, Zeitablauf). Falls Click-through in v1 gewollt ist, gehört das hier mit entschieden.

> Kontext aus *[Phasenablauf](03-phasenablauf.md)*: Diese Ablauf-Aktionen brauchen einen Weg in der Bedienung: Starten (aus *bereit*), Anhalten/Fortsetzen, Überspringen, **Phase zurücksetzen** und **Neu beginnen**. Phase zurücksetzen und Neu beginnen sind getrennte Aktionen.

## Resolution

Begriffe (*Primäraktion*, *Click-through*, *Ausgeblendet*) stehen in [GLOSSARY.md](../../../../GLOSSARY.md).

1. **Bedienwege:** Overlay (Maus, lokale Tasten) und Tray-Icon. **Keine globalen Hotkeys** in v1.
2. **Click-through** ist ein Schalter, Voreinstellung **aus**. Einschalten geht über beide Menüs, ausschalten nur über das Tray-Icon, weil das Overlay dann nicht mehr anklickbar ist.
3. **Linksklick aufs Overlay** ohne Bewegung löst die *Primäraktion* aus (aus *bereit* Starten, aus *läuft* Anhalten, aus *angehalten* Fortsetzen). **Ziehen** über eine kleine Bewegungsschwelle (wenige Pixel) verschiebt das Overlay.
4. **Ein gemeinsames Menü:** Rechtsklick aufs Overlay und Links- oder Rechtsklick aufs Tray-Icon öffnen alle **dasselbe** Menü. Die Einträge in dieser Reihenfolge:
   ```
   <Primäraktion>  (Text je nach Zustand: Starten / Anhalten / Fortsetzen)
   Überspringen
   Phase zurücksetzen
   Neu beginnen
   ─────────
   [✓] Immer im Vordergrund
   [ ] Click-through
   [✓] Overlay anzeigen
   Theme ▸
   Einstellungen …
   ─────────
   Beenden
   ```
   Wie *Theme ▸* aussieht, regelt *Theming*. Was hinter *Einstellungen …* steckt, regelt *Einstellungen und Speichern*.
5. **Ausgrauen:** Ein Eintrag ist ausgegraut, wenn er den Stand nicht ändern würde (z. B. *Phase zurücksetzen* bei voller Dauer im Zustand *bereit*, oder *Neu beginnen* bei Fokusphase 1 im Zustand *bereit*). *Überspringen* ist nie ausgegraut.
6. **Keine Rückfragen** bei *Phase zurücksetzen* und *Neu beginnen*.
7. **Lokale Tastenkürzel:** Sie wirken nur, wenn das Overlay den Fokus hat, und sind fest, nicht konfigurierbar: `Leertaste` löst die Primäraktion aus, `Esc` schließt das Menü, `Umschalt+F10`/`Menütaste` öffnet es. Überspringen, Phase zurücksetzen, Neu beginnen und Beenden haben **bewusst kein Kürzel** (sie verwerfen etwas und kommen ohne Rückfrage).
8. **Tray-Tooltip** zeigt den aktuellen Stand: Phase, Zyklus-Fortschritt, Zustand und Rest- bzw. Überzeit, im **gleichen Format wie das Overlay**.
9. **Overlay anzeigen** ist ein Schalter. Ausgeblendet läuft der Ablauf weiter. Ist das Overlay ausgeblendet, ist der Schalter nur noch über das Tray-Icon erreichbar.
