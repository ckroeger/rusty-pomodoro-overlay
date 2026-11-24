---
title: Spec für Rusty Pomodoro Overlay
labels: [wayfinder:map]
status: open
---

# Spec für Rusty Pomodoro Overlay

## Destination

Eine `SPEC.md` (plus `GLOSSARY.md` und ADRs nur für schwer umkehrbare Entscheidungen), nach der ein Agent oder du **v1 ohne Rückfragen** bauen kann: ein Pomodoro-Overlay in Rust, **nur Windows**, inkl. festgelegter Technik. v1 = klassischer Pomodoro-Zyklus, randloses Overlay, an-/abschaltbares Immer-im-Vordergrund, einfaches Theming.

## Notes

- Domäne: persönliches Desktop-Tool, Rust, Windows 11. Begriffe siehe [GLOSSARY.md](../../../GLOSSARY.md).
- Sprache: Deutsch. Beim Grilling **immer nur eine Frage pro Nachricht**.
- Skills pro Session: `grilling` + `domain-modeling` (Glossar inline pflegen); `research` für Research-Tickets; `prototype` für Prototyp-Tickets.
- Verteilung: nur `cargo build` / `cargo install`, kein Installer.
- **Tracker = lokales Markdown** im git-Repo (kein Issue-Tracker; das Repo wird nach GitHub gespiegelt):
  - Karte: diese Datei. Tickets: `tickets/NN-<slug>.md`; die Nummer `NN` ist die ID.
  - Frontmatter je Ticket: `title`, `labels` (`wayfinder:<type>`), `status` (`open`/`closed`), `assignee` (leer = nicht geclaimt; Claim = `ckr` eintragen, **vor** jeder Arbeit), `blocked_by` (Liste von Ticket-IDs).
  - Frontier = `status: open`, `assignee` leer, alle `blocked_by` geschlossen. Reihenfolge = Ticket-Nummer.
  - Auflösung: Abschnitt `## Resolution` ans Ticket anhängen, `status: closed`, eine Zeile unter *Decisions so far*.
  - Research-Ergebnisse liegen unter `research/`, verlinkt aus dem Ticket.
  - Prototypen sind Wegwerf-Code auf einem eigenen Branch `prototype/<NN>-<slug>` (nie auf `main`), verlinkt aus dem Ticket.

## Decisions so far

<!-- eine Zeile pro geschlossenem Ticket: [Titel](link): Kern der Antwort -->
- [GUI-Technik für das Overlay](tickets/01-gui-technik.md): Empfehlung eframe/egui 0.37 mit glow-Renderer (alle Overlay-Features nativ, klein); Zweitplatz slint; Transparenz und Click-through im Spike prüfen.
- [Overlay-Spike mit der gewählten Technik](tickets/02-overlay-spike.md): eframe/egui 0.37 + glow bestätigt. Alpha pro Pixel, einheitliche Deckkraft, Immer-im-Vordergrund zur Laufzeit, Drag und Click-through funktionieren; 4,7 MB. RAM ~168 MB auf NVIDIA vs. ~94 MB auf iGPU.
- [iGPU für das Overlay bevorzugen](tickets/10-igpu-bevorzugen.md): Kein Eingriff durch die App. Ein Doku-Hinweis auf die Windows-Grafikeinstellung „Energiesparmodus“ plus Anzeige des `GL_RENDERER` genügt (Optimus-Symbole können nur die dGPU anfordern, glutin/eframe haben keine Adapter-Wahl); optional später ein ausdrücklicher Befehl `--prefer-igpu`, der `UserGpuPreferences` (HKCU, undokumentiert) setzt.
- [Phasenablauf](tickets/03-phasenablauf.md): 25/5/15 und 4 pro Zyklus sind Voreinstellungen, konfigurierbar (ganze Minuten 1 bis 180, 1 bis 12 Fokusphasen pro Zyklus). Zustände läuft/angehalten/bereit. Je Richtung ein Schalter für automatischen Start (Voreinstellung: Pause automatisch, Fokus wartet). Überspringen in jedem Zustand, zählt mit. Getrennt: Phase zurücksetzen und Neu beginnen. Überzeit nur angezeigt. Wanduhr inkl. Standby. Änderungen ab nächster Phase bzw. nächstem Zyklus.
- [Bedienung](tickets/04-bedienung.md): Overlay plus Tray-Icon, keine globalen Hotkeys. Linksklick löst die Primäraktion aus, Ziehen verschiebt. Ein gemeinsames Menü für Overlay und Tray mit den Schaltern Click-through (aus), Immer im Vordergrund und Overlay anzeigen. Einträge ohne Wirkung sind ausgegraut, keine Rückfragen. Lokal nur Leertaste/Esc/Umschalt+F10. Tray-Tooltip zeigt den Stand.
- [Crates für Tray-Icon und gemeinsames Menü](tickets/11-tray-und-menue-crates.md): `tray-icon` 0.26 + `muda` 0.21. Ein natives Menü hängt am Tray, das Overlay öffnet dasselbe per `show_context_menu_for_hwnd`, kein egui-Nachbau und kein eigener Thread, +161 KB. Bei offenem Menü steht die Event-Loop, deshalb wird die Restzeit aus der Uhr berechnet und nicht gezählt. Kein Toast, keine Single-Instance.

## Not yet specified

- **Begleit-Crates** für Windows-Benachrichtigungen, Ton, Single-Instance, Autostart: welche konkret, hängt davon ab, was *Hinweis bei Phasenwechsel* und *Lebenszyklus der App* entscheiden. Tray-Icon und Menü sind schon ein Ticket (*Crates für Tray-Icon und gemeinsames Menü*), globale Hotkeys sind entfallen.
- **ADR zur GUI-Technik**: Inhalt steht (Recherche, Spike, GPU-Wahl aus *iGPU für das Overlay bevorzugen*). Geschrieben wird es beim Zusammenschreiben der SPEC.md.
- **Architektur der App**: Trennung Timer-Logik / UI / Konfiguration, Testbarkeit der Timer-Logik (injizierbare Uhr, siehe *Phasenablauf*); erst greifbar, wenn Technik und Bedienung entschieden sind. Aus *Crates für Tray-Icon und gemeinsames Menü*: Restzeit aus `Instant`-Differenzen (die Event-Loop steht bei offenem Menü), Tray-Code in `App::logic`, Menüzustand nach jedem Event aus dem App-Zustand neu setzen. Offen und Kandidat für einen Prototyp: Läuft `logic()` bei ausgeblendetem Overlay (`Visible(false)`) zuverlässig jede Sekunde?
- **Zusammenschreiben der SPEC.md**: Gliederung und finale Konsolidierung aller Entscheidungen; letzter Schritt vor dem Ziel.

## Out of scope

- Linux und macOS: v1 ist nur Windows.
- Statistiken, Aufgabenlisten, Synchronisierung: nicht Teil der schlanken v1.
- Installer / Paketierung: Verteilung nur über `cargo build` / `cargo install`.
