---
title: SPEC.md zusammenschreiben
labels: [wayfinder:task]
status: closed
assignee: ckr
blocked_by: [15]
---

# SPEC.md zusammenschreiben

## Question

Alle Entscheidungen der Karte (*Decisions so far*) zu einer `SPEC.md` im Repo-Wurzelverzeichnis konsolidieren, nach der ein Agent v1 ohne Rückfragen bauen kann. Dazu kommt das ADR zur GUI-Technik unter `docs/adr/`. Der Entwurf entsteht AFK. Fertig ist das Ticket, wenn ckr den Entwurf durchgesehen und freigegeben hat.

## Kontext

- Die Gliederung folgt den Modulen aus *[Architektur der App](15-architektur.md)*: Verhalten pro Bereich, Schnittstellen der Module, Teststrategie und manuelle Prüfliste (Overlay, Tray, Plattform, Einstellungsfenster).
- Crate-Liste: `eframe`/`egui` 0.37 (glow), `tray-icon` 0.26 + `muda` 0.21, `windows-sys` 0.61 (Ton, „nicht stören“, Mutex/Event, Autostart, MessageBox), `image` 0.25 (nur `jpeg` + `png`), `skrifa` + `vello_cpu` in den Versionen von `epaint`, `toml_edit`.
- ADR zur GUI-Technik: Inhalt aus *[GUI-Technik für das Overlay](01-gui-technik.md)*, *[Overlay-Spike mit der gewählten Technik](02-overlay-spike.md)* und *[iGPU für das Overlay bevorzugen](10-igpu-bevorzugen.md)*.
- Begriffe aus [GLOSSARY.md](../../../../GLOSSARY.md) verwenden. Die Versionen vor dem Schreiben gegen crates.io prüfen.

## Resolution

**Freigegeben am 2026-11-26.** Die Spec liegt in [SPEC.md](../../../../SPEC.md), das ADR in [0001 eframe/egui mit dem glow-Renderer](../../../adr/0001-eframe-egui-mit-glow.md).

- Versionen gegen crates.io geprüft. `tray-icon` 0.27/`muda` 0.22 sind seit 2026-11-25 auf crates.io, v1 bleibt bei den geprüften 0.26/0.21. `skrifa` 0.44/`vello_cpu` 0.2 sind an `epaint` 0.37 gebunden.
- Mit ckr beim Durchsehen entschieden:
  - Eine Phase in *bereit* übernimmt eine geänderte Dauer sofort (Nachtrag in *[Phasenablauf](03-phasenablauf.md)*).
  - Der Theme-Ordner wird im Sekundentakt gelesen, die Schrift ist ein Theme-Feld (`ubuntu`/`inter`/`jetbrains_mono`; Nachtrag in *[Theming](07-theming.md)*, Glossar angepasst).
  - Klänge als erzeugte Sinustöne; Klänge je Theme sind für später vorgemerkt (*Out of scope*).
  - Im Menü steht *Ton* nach *Overlay anzeigen*, per Tastatur geht das Menü oben links am Overlay auf. `GL_RENDERER` und Version stehen in der Fußzeile des Einstellungsfensters.
- Nicht im Spike geprüft und deshalb in der Prüfliste: Einblenden ohne Fokus (`SW_SHOWNOACTIVATE`), Sichern bei `WM_ENDSESSION` per Subclass, Einstellungsfenster bei ausgeblendetem Overlay (`RequestPaintWhileHidden`).
