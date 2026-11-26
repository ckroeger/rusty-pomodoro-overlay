# eframe/egui mit dem glow-Renderer für das Overlay

Das Overlay braucht unter Windows 11 ein randloses Fenster mit Alpha pro Pixel, zur Laufzeit umschaltbares *Immer-im-Vordergrund*, Verschieben per Maus und *Click-through*. Wir nehmen `eframe`/`egui` 0.37 mit `default-features = false` und dem **`glow`-Renderer (OpenGL)**, nicht dem seit eframe 0.34 voreingestellten `wgpu`. Alle Fenster-Features sind dort öffentliche API (`ViewportCommand::WindowLevel`, `MousePassthrough`, `StartDrag`), und der Spike hat sie auf echter Hybrid-GPU-Hardware bestätigt: 4,7 MB Binärgröße, Alpha pro Pixel auf NVIDIA und Intel.

## Considered Options

- **`wgpu` als Renderer (Default):** transparente Fenster werden unter Windows mit DX12 schwarz, dazu etwa doppelte Binärgröße und mehrfacher RAM (egui #7761). Wer das „Default“ wiederherstellen will, bricht das Overlay.
- **`slint` 1.19:** bestes Theming, aber Drag und Click-through nur über ein `unstable`-Feature oder HWND-Aufrufe.
- **`iced`** (offener Transparenz-Bug unter Windows), **`tauri`** (WebView2, zu schwer), **`winit` + `softbuffer`** (kein Alpha unter Win32, viel Eigenbau).

## Consequences

- *Click-through* wird **erst zur Laufzeit** gesetzt (`MousePassthrough(true)` nach dem Start, nie im `ViewportBuilder`). Sonst wird das Fenster unsichtbar (egui #2537).
- Auf Hybrid-GPU-Laptops läuft OpenGL standardmäßig oft auf der dGPU (~168 MB statt ~94 MB Working Set). Der Stack bietet keine Adapter-Wahl, die Optimus-Symbole können nur die dGPU anfordern. Die App greift nicht ein; das README beschreibt die Windows-Grafikeinstellung *Energiesparmodus*, und das Einstellungsfenster zeigt den `GL_RENDERER`.
- egui veröffentlicht etwa alle zwei Monate eine Minor-Version mit API-Brüchen (0.37: `App::ui` statt `update`). `skrifa`/`vello_cpu` fürs Tray-Zeichen müssen bei jedem Update auf die Versionen von `epaint` nachgezogen werden.

Quellen: [GUI-Technik für das Overlay](../wayfinder/pomodoro-overlay-spec/tickets/01-gui-technik.md), [Overlay-Spike mit der gewählten Technik](../wayfinder/pomodoro-overlay-spec/tickets/02-overlay-spike.md), [iGPU für das Overlay bevorzugen](../wayfinder/pomodoro-overlay-spec/tickets/10-igpu-bevorzugen.md).
