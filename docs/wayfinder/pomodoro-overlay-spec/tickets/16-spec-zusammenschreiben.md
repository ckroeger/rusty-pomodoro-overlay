---
title: SPEC.md zusammenschreiben
labels: [wayfinder:task]
status: open
assignee:
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
