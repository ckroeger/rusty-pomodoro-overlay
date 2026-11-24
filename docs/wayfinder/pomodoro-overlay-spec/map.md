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
- **Tracker = lokales Markdown** (kein git-Repo, kein Issue-Tracker):
  - Karte: diese Datei. Tickets: `tickets/NN-<slug>.md`; die Nummer `NN` ist die ID.
  - Frontmatter je Ticket: `title`, `labels` (`wayfinder:<type>`), `status` (`open`/`closed`), `assignee` (leer = nicht geclaimt; Claim = `ckr` eintragen, **vor** jeder Arbeit), `blocked_by` (Liste von Ticket-IDs).
  - Frontier = `status: open`, `assignee` leer, alle `blocked_by` geschlossen. Reihenfolge = Ticket-Nummer.
  - Auflösung: Abschnitt `## Resolution` ans Ticket anhängen, `status: closed`, eine Zeile unter *Decisions so far*.
  - Research-Ergebnisse liegen unter `research/`, verlinkt aus dem Ticket.

## Decisions so far

<!-- eine Zeile pro geschlossenem Ticket: [Titel](link): Kern der Antwort -->
- [GUI-Technik für das Overlay](tickets/01-gui-technik.md): Empfehlung eframe/egui 0.37 mit glow-Renderer (alle Overlay-Features nativ, klein); Zweitplatz slint; Transparenz und Click-through im Spike prüfen.
- [Overlay-Spike mit der gewählten Technik](tickets/02-overlay-spike.md): eframe/egui 0.37 + glow bestätigt. Alpha pro Pixel, einheitliche Deckkraft, Immer-im-Vordergrund zur Laufzeit, Drag und Click-through funktionieren; 4,7 MB. RAM ~168 MB auf NVIDIA vs. ~94 MB auf iGPU.
- [iGPU für das Overlay bevorzugen](tickets/10-igpu-bevorzugen.md): Kein Eingriff durch die App. Ein Doku-Hinweis auf die Windows-Grafikeinstellung „Energiesparmodus“ plus Anzeige des `GL_RENDERER` genügt (Optimus-Symbole können nur die dGPU anfordern, glutin/eframe haben keine Adapter-Wahl); optional später ein ausdrücklicher Befehl `--prefer-igpu`, der `UserGpuPreferences` (HKCU, undokumentiert) setzt.

## Not yet specified

- **Begleit-Crates** für Tray-Icon, globale Hotkeys, Windows-Benachrichtigungen, Ton, Single-Instance, Autostart: welche konkret, hängt davon ab, was *Bedienung* und *Hinweis bei Phasenwechsel* entscheiden. Vermutlich ein oder mehrere Research-Tickets.
- **ADR zur GUI-Technik**: Inhalt steht (Recherche, Spike, GPU-Wahl aus *iGPU für das Overlay bevorzugen*). Geschrieben wird es beim Zusammenschreiben der SPEC.md.
- **Architektur der App**: Trennung Timer-Logik / UI / Konfiguration, Testbarkeit der Timer-Logik; erst greifbar, wenn Technik und Bedienung entschieden sind.
- **Zusammenschreiben der SPEC.md**: Gliederung und finale Konsolidierung aller Entscheidungen; letzter Schritt vor dem Ziel.

## Out of scope

- Linux und macOS: v1 ist nur Windows.
- Statistiken, Aufgabenlisten, Synchronisierung: nicht Teil der schlanken v1.
- Installer / Paketierung: Verteilung nur über `cargo build` / `cargo install`.
