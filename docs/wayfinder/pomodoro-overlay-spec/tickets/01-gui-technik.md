---
title: GUI-Technik für das Overlay
labels: [wayfinder:research]
status: closed
assignee: ckr
blocked_by: []
---

# GUI-Technik für das Overlay

## Question

Welche Rust-GUI-/Fenster-Bibliothek eignet sich unter Windows 11 für ein kleines, **randloses**, **(halb-)transparentes** Overlay-Fenster, dessen **Immer-im-Vordergrund** zur Laufzeit an- und abschaltbar ist, das sich per Maus verschieben lässt und optional Klicks durchreichen kann (click-through)? Kandidaten u. a. `eframe`/`egui`, `iced`, `slint`, `tauri`, `winit` + eigenes Rendering (`softbuffer`/`tiny-skia`). Bewertet nach: Windows-Unterstützung dieser Fenster-Features (nativ oder per `windows`-Crate-Hack), Binärgröße, Leerlauf-CPU/RAM, Reife/Pflege, Aufwand für einfaches Theming. Ergebnis: Rangliste mit Empfehlung und offenen Risiken.

## Resolution

**Empfehlung: `eframe`/`egui` 0.37 (≥ 0.36.2) mit `glow`-Renderer** (`default-features = false`; nicht der Default `wgpu`). Alle Fenster-Features sind öffentliche API, auch zur Laufzeit: `WindowLevel`, `MousePassthrough`, `StartDrag`, `Decorations`, `Transparent`, `with_taskbar`. Mit glow laut egui #7761 etwa 5,6 MB (LTO) und etwa 31 MB RAM.
**Zweitplatz: `slint` 1.19.** Theming ist dort am besten, aber Drag und Click-through brauchen `unstable-winit-030` oder HWND-Aufrufe. Danach folgen iced (offener Windows-Transparenz-Bug), tauri (WebView2, zu schwer) und winit+softbuffer (kein Alpha auf Win32, viel Eigenbau).
**Hauptrisiken für den Spike (02):** Alpha pro Pixel unter Windows hängt von Renderer und GPU ab (wgpu/DX12 wird schwarz). Der offene Click-through-Bug egui #2537 macht das Fenster unsichtbar. Bei aktivem Click-through braucht man einen anderen Weg zurück (Hotkey/Tray). Die Spec sollte klären, ob einheitliche Fenster-Deckkraft (`SetLayeredWindowAttributes`) reicht.
Details: [Recherche](../research/01-gui-technik.md)
