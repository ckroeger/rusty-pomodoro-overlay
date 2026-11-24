---
title: Overlay-Spike mit der gewählten Technik
labels: [wayfinder:prototype]
status: closed
assignee: ckr
blocked_by: [01]
---

# Overlay-Spike mit der gewählten Technik

## Question

Bestätigt ein Wegwerf-Prototyp mit dem in *GUI-Technik für das Overlay* empfohlenen Kandidaten, dass unter Windows 11 randlos + transparent + zur Laufzeit umschaltbares Immer-im-Vordergrund + Verschieben per Maus tatsächlich funktionieren, und fühlt sich das Ergebnis richtig an? Falls nicht: welcher Kandidat stattdessen?

## Resolution

**Bestätigt: `eframe`/`egui` 0.37 + `glow` bleibt die Technik.** Alles wurde vom Menschen auf der echten Hardware unter Windows 11 geprüft (Hybrid-GPU: NVIDIA RTX A2000 Laptop + Intel UHD), und es fühlt sich gut an:
- **Randlos + Alpha pro Pixel:** funktioniert, mit `with_transparent(true)`, `clear_color` = `[0,0,0,0]` und halbtransparentem Panel-Fill. Der Hintergrund ist per Regler einstellbar, der Text bleibt deckend. Das geht auf NVIDIA und auf der Intel-iGPU.
- **Einheitliche Deckkraft** (`WS_EX_LAYERED` + `SetLayeredWindowAttributes(LWA_ALPHA)` am HWND, zusätzlich zum Alpha pro Pixel): funktioniert ebenfalls, ohne Artefakte. Beide Arten von „halbtransparent“ stehen der Spec also offen.
- **Immer-im-Vordergrund zur Laufzeit** (`ViewportCommand::WindowLevel(AlwaysOnTop/Normal)`): funktioniert, `WS_EX_TOPMOST` wechselt tatsächlich mit.
- **Verschieben per Maus** (`ViewportCommand::StartDrag` beim Drücken auf freie Fläche): flüssig.
- **Click-through** (`ViewportCommand::MousePassthrough(true)` erst zur Laufzeit): Das Overlay bleibt sichtbar, Klicks gehen durch, danach ist es wieder bedienbar. Der Bug egui #2537 tritt mit diesem Workaround nicht auf. Im Spike läuft Click-through nach 5 s automatisch ab. In der echten App braucht es einen anderen Weg zurück (Hotkey/Tray, siehe *Bedienung*).

**Messwerte** (Release, LTO, `strip`, `codegen-units = 1`, `panic = "abort"`, MSVC):
- Binärgröße: **4,7 MB**.
- Leerlauf (`request_repaint_after(1 s)`) auf NVIDIA (Default-Treiberwahl): ~168 MB Working Set / 193 MB privat, ~1,6 % eines Kerns.
- Leerlauf auf Intel-iGPU (Windows-Grafikeinstellung „Stromsparend“ für die exe): **~94 MB Working Set / 53 MB privat, ~0,6 % eines Kerns**.
- Der Mensch findet den RAM auf NVIDIA „könnte besser sein“. Wie die App die iGPU bevorzugen kann, klärt das neue Ticket *iGPU für das Overlay bevorzugen*.

**Für den Bau festgehalten:**
- `eframe = { version = "0.37", default-features = false, features = ["glow", "default_fonts"] }` reicht. Für HWND-Zugriff kommen `raw-window-handle` 0.6 (`eframe::Frame` implementiert `HasWindowHandle`) und `windows-sys` dazu.
- **API-Änderung in 0.37:** `eframe::App` verlangt `fn ui(&mut self, ui: &mut egui::Ui, frame)` statt `update`. `CentralPanel::show` nimmt `&mut Ui`, `Id::new` ist deprecated (`Id::unique`/`IdSalt`). Die Recherche-Beispiele mit `update` sind veraltet.

Prototyp (Wegwerf-Code, kein git vorhanden, deshalb neben der Karte abgelegt): [prototypes/02-overlay-spike](../prototypes/02-overlay-spike/), Start mit `cargo run --release` im Ordner.
