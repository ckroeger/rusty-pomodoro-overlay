# Recherche: GUI-Technik für das Overlay

Ticket: [01 – GUI-Technik für das Overlay](../tickets/01-gui-technik.md) · Stand: 2026-11-24

## Frage

Welche Rust-GUI-/Fenster-Bibliothek eignet sich unter Windows 11 für ein kleines, randloses, (halb-)transparentes **Overlay**, dessen **Immer-im-Vordergrund** zur Laufzeit umschaltbar ist, das per Maus verschiebbar ist und optional click-through kann? Kriterien: Windows-Unterstützung der Fenster-Features, Binärgröße, Leerlauf-CPU/RAM (Neuzeichnen einmal pro Sekunde), Reife/Pflege, Aufwand für einfaches Theming.

## Kurzantwort

**Empfehlung: `eframe`/`egui` 0.37 mit dem `glow`-Renderer (nicht dem Default `wgpu`).** Alle geforderten Fenster-Features gibt es als öffentliche API, auch zur Laufzeit. Mit `glow` sind laut Messungen aus dem egui-Repo Binärgröße und RAM klein, und es umgeht das bekannte Problem, dass transparente Fenster unter Windows mit wgpu/DX12 schwarz werden.
**Zweitplatz: `slint` 1.19.** Theming ist dort am angenehmsten, aber Drag und Click-through brauchen den Zugriff auf das winit-Fenster (über ein `unstable`-Feature) oder Win32-Aufrufe.

## Geprüfte Versionen (crates.io-API, abgefragt 2026-11-24)

| Crate | Letzte stabile Version | Datum | Bemerkung |
|---|---|---|---|
| `eframe` / `egui` | 0.37.0 | 2026-10-15 | sehr häufige Releases (0.34 im März, 0.35 im Juni, 0.36 im August, 0.37 im Oktober 2026) |
| `slint` | 1.19.1 | 2026-11-24 | etwa ein Minor-Release alle 2–3 Monate |
| `iced` | 0.14.0 | 2025-12-07 | davor 0.13.1 (2024-09); langsamer Takt |
| `tauri` | 2.12.7 | 2026-11-18 | |
| `winit` | 0.30.13 stabil (2026-03-02), 0.31.0-beta.4 (2026-10-24) | | 0.31 ist noch Beta |
| `softbuffer` | 0.4.8 | 2025-12-13 | Alpha-Unterstützung ist für 0.5 geplant |
| `tiny-skia` | 0.12.0 | 2026-02-02 | |
| `windows` (windows-rs) | 0.62.2 | 2025-10-06 | für HWND-Aufrufe |

Quelle: `https://crates.io/api/v1/crates/<name>` und `/versions`.

## Wie Windows die Fenster-Features umsetzt (Hintergrund)

Quelle: [Microsoft Learn – Extended Window Styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)

- **Immer-im-Vordergrund** = `WS_EX_TOPMOST`: „To add or remove this style, use the **SetWindowPos** function“ (`HWND_TOPMOST` / `HWND_NOTOPMOST`). Das geht also mit jedem Framework, das ein HWND herausgibt.
- **Click-through** geht in der Praxis über `WS_EX_LAYERED | WS_EX_TRANSPARENT`. winit setzt dafür in `set_cursor_hittest(false)` `WS_EX_LAYERED` (siehe egui-Issue #2537 unten).
- **Halbtransparenz** gibt es auf zwei Wegen:
  1. **Einheitliche Fenster-Deckkraft**: `WS_EX_LAYERED` + `SetLayeredWindowAttributes(LWA_ALPHA)`. Text und Hintergrund werden gleich stark durchsichtig. Das hängt nicht vom Renderer ab und geht per HWND mit jedem Framework.
  2. **Alpha pro Pixel**, also transparenter Hintergrund mit deckendem Text. Das braucht einen Renderer bzw. eine Swapchain mit Alpha. Genau hier liegen die Probleme mit wgpu/DX12 (siehe unten).
- `WS_EX_TOOLWINDOW` blendet das Fenster aus Taskleiste und Alt+Tab aus; `WS_EX_NOACTIVATE` verhindert, dass ein Klick das Fenster aktiviert. Beides kann für ein Overlay nützlich sein.

> Für die Spec wichtig: Ob „halbtransparent“ *einheitliche Deckkraft* oder *Alpha pro Pixel* bedeutet, entscheidet über das größte technische Risiko. Einheitliche Deckkraft ist mit jeder Bibliothek sicher machbar.

## Kandidaten im Einzelnen

### 1. `eframe`/`egui` 0.37 (Empfehlung)

**Fenster-Features als öffentliche API** ([ViewportBuilder](https://docs.rs/egui/0.37.0/egui/viewport/struct.ViewportBuilder.html), [ViewportCommand](https://docs.rs/egui/0.37.0/egui/viewport/enum.ViewportCommand.html)):
- Beim Start: `with_decorations(false)`, `with_transparent(true)`, `with_window_level(..)` / `with_always_on_top()`, `with_mouse_passthrough(..)`, `with_taskbar(..)` (Doku: „windows: Whether show or hide the window icon in the taskbar“), `with_resizable`, `with_inner_size`, `with_position`.
- Zur Laufzeit: `ViewportCommand::WindowLevel(WindowLevel)` (always-on-top / always-on-bottom / normal), `MousePassthrough(bool)`, `StartDrag` (verschiebt das Fenster, solange die linke Maustaste gedrückt ist; nur zuverlässig direkt nach dem Drücken), `Decorations(bool)`, `Transparent(bool)`, `OuterPosition`, `InnerSize`.
- Damit sind alle Pflicht-Features ohne `windows`-Crate abgedeckt. Einheitliche Fenster-Deckkraft (siehe oben) bräuchte ein HWND; eframe nutzt winit und raw-window-handle, die einzelnen Wege muss der Spike prüfen.

**Renderer und Transparenz unter Windows:**
- Seit eframe 0.34.0 ist **`wgpu` der Default-Renderer** ([eframe CHANGELOG](https://github.com/emilk/egui/blob/main/crates/eframe/CHANGELOG.md), #7615; Default-Features laut [docs.rs](https://docs.rs/crate/eframe/0.37.0/features): u. a. `wgpu`, `winit`, `accesskit`, `x11`, `wayland`; `glow` ist nicht default).
- [egui #7761](https://github.com/emilk/egui/issues/7761) (offen, Windows, Beispiel `run_simple_native`, Default-Features aus) hat glow und wgpu verglichen:
  - **Binärgröße (Release + LTO):** glow 5,62 MB, wgpu 11,3 MB (0.33.3: 4,44 MB). Ohne LTO: 17,7 MB bzw. 27,7 MB. Der Autor sah außerdem einen deutlichen Unterschied zwischen GNU- und MSVC-Toolchain.
  - **RAM (Task-Manager):** glow 31 MB, wgpu-Vulkan 90 MB, wgpu-DX12 142 MB. **VRAM:** glow 31 MB, Vulkan 265 MB.
  - **Start:** glow 100–300 ms, wgpu 1–7 s.
  - „Window transparency appears unsupported on wgpu's DX12 and GL backends.“
- [egui #4451](https://github.com/emilk/egui/issues/4451): Unter Windows 11 erschien ein schwarzes statt transparentes Fenster; mit dem glow-Renderer ging es. Bei Hybrid-GPU funktionierte es nur, wenn die Intel-iGPU verwendet wurde.
- Die Changelog-Einträge zeigen, dass das Thema aktiv gepflegt wird: 0.36.2 „Fix transparent child viewports on Windows with glow“ (#8423), 0.37.0 „Preserve transparent viewport windows on Windows“ (#8616).
- **Folgerung:** `eframe = { default-features = false, features = ["glow", "default_fonts", ...] }`. Den genauen Feature-Satz legt der Spike fest.

**Click-through-Bug:** [egui #2537](https://github.com/emilk/egui/issues/2537) ist **offen**. Mit `mouse_passthrough` + transparent + undecorated bleibt das Fenster unsichtbar (in der Taskleiste nur ein schwarzer Kasten). Ursache laut Thread: winit setzt `WS_EX_LAYERED` in `set_cursor_hittest`, ruft aber `SetLayeredWindowAttributes` nicht auf, und der Effekt hängt mit `with_visible` zusammen. Ein Kommentator konnte es unter Windows 11 nicht reproduzieren. Der meistbestätigte Workaround: `ViewportCommand::MousePassthrough(true)` erst **zur Laufzeit** schicken statt beim Builder; alternativ `SetLayeredWindowAttributes` selbst aufrufen.

**Leerlauf-CPU:** egui zeichnet im Immediate Mode, aber nur auf Anforderung (z. B. `ctx.request_repaint_after(Duration::from_secs(1))`; Standard-API, im Spike verwenden). Relevante Fixes: 0.34.0 „Only run App::ui if the application is visible“ (#7950), „Avoid repaints on device mouse motion outside window“ (#7866), 0.36.2 „Don't busy-loop a CPU core while waiting for a redraw“ (#8398). Wegen des Letzteren **mindestens 0.36.2** verwenden.

**Theming:** In Code über `egui::Visuals`/`Style` (Farben, Rundungen, Schrift) und eigene `Frame`s. Ein Theme als Rust-Struct ist einfach; Themes aus einer Konfigurationsdatei (TOML usw.) muss man selbst auf `Visuals` abbilden. Wenig Aufwand.

**Reife:** Etwa monatliche Releases, viele Nutzer. API-Brüche zwischen Minor-Versionen (0.x) sind üblich, daher die Version pinnen.

### 2. `slint` 1.19 (Zweitplatz)

- **Window-Properties** ([Doku](https://docs.slint.dev/latest/docs/slint/reference/window/window/)): `no-frame`, `always-on-top` („only works on window managers that support it“), `background` (laut Maintainer funktioniert `background: transparent` zusammen mit `no-frame: true`; [Discussion #5111](https://github.com/slint-ui/slint/discussions/5111)). Die Doku sagt **nicht**, ob `always-on-top` zur Laufzeit änderbar ist (ungeprüft; per Property-Binding ist es wahrscheinlich, der Spike muss es zeigen).
- **Click-through:** Slint hat das nicht eingebaut (Discussion #5111). Man braucht HWND/winit.
- **Drag:** Es gibt keine eingebaute Drag-Property. Wege: per `TouchArea` die Position selbst über `Window::set_position` nachführen, oder über [`slint::winit_030::WinitWindowAccessor`](https://docs.rs/slint/1.19.1/slint/winit_030/trait.WinitWindowAccessor.html) (`with_winit_window`, gibt `&winit::window::Window`) `drag_window()` / `set_cursor_hittest()` aufrufen. Dieses Trait braucht das Crate-Feature **`unstable-winit-030`**, ist also nicht unter SemVer stabil.
- **Renderer und Transparenz:** Default-Features 1.19.1 ([docs.rs](https://docs.rs/crate/slint/1.19.1/features)): `renderer-femtovg` (OpenGL), `renderer-software`, `backend-default`, `accessibility`, `system-tray`. [slint #13572](https://github.com/slint-ui/slint/issues/13572) (offen, 1.18.1, Windows 11): Ein transparentes Fenster wird **schwarz mit Skia-wgpu (DX12)**. Bei Skia-OpenGL verlor ein AMD-Treiber das Alpha. FemtoVG und Software-Renderer werden dort nicht erwähnt (ungeprüft).
- **Theming:** Am besten von allen nativen Kandidaten. Deklaratives `.slint` mit `global`-Theme-Objekten und Properties, die man zur Laufzeit aus Rust setzt.
- **Nebenbei:** Das Default-Feature `system-tray` könnte für das spätere Tray-Icon-Ticket relevant sein (nicht weiter geprüft).
- **Lizenz:** Slint wird unter mehreren Lizenzen angeboten (GPLv3 / Royalty-free / kommerziell). Für ein persönliches Tool unkritisch, die Bedingungen im Detail habe ich hier nicht geprüft.
- **Binärgröße/RAM:** keine Primärquelle mit Messwerten gefunden (ungeprüft).

### 3. `iced` 0.14

- **API vollständig:** `iced::window` ([docs.rs](https://docs.rs/iced/0.14.0/iced/window/index.html)) hat `set_level` (`Level::{Normal, AlwaysOnBottom, AlwaysOnTop}`), `drag`, `enable_mouse_passthrough` / `disable_mouse_passthrough`, `move_to`, `toggle_decorations`, `run` (Zugriff auf das Fenster bzw. den Raw-Handle). `Settings` hat `decorations`, `transparent`, `level`, `platform_specific` ([Settings](https://docs.rs/iced/0.14.0/iced/window/struct.Settings.html); `blur` gibt es nur auf macOS/Linux).
- **Transparenz unter Windows kaputt:** [iced #2525](https://github.com/iced-rs/iced/issues/2525) ist **offen**: Unter Windows 11 wird das transparente Fenster schwarz (wgpu/DX12). Workarounds: `ICED_BACKEND=tiny-skia` (CPU, für einfache UIs ausreichend) oder Vulkan. Ein Contributor schrieb im Oktober 2026 „try master, I think it's been fixed upstream“; ein anderer Nutzer meldete, dass 0.14 es für ihn nicht behebt.
- **Reife:** Das letzte Release liegt fast ein Jahr zurück (0.14.0, Dezember 2025). Die Elm-Architektur bringt für eine Timer-Anzeige etwas Boilerplate mit sich.
- **Theming:** eigene `Theme`/`Palette` plus Style-Funktionen pro Widget; mittlerer Aufwand.
- **Binärgröße/RAM:** nicht geprüft.

### 4. `tauri` 2.12

- **API vollständig** ([Window](https://docs.rs/tauri/2.12.7/tauri/window/struct.Window.html)): `set_always_on_top`, `set_ignore_cursor_events` (Click-through), `start_dragging`, `set_decorations`, `set_skip_taskbar`, `set_shadow`. Windows-Hinweise: `set_shadow(true)` gibt undekorierten Fenstern einen 1-px-weißen Rand; `set_background_color` ignoriert den Alpha-Kanal. Transparenz muss man über die Fenster-Konfiguration einstellen.
- **Theming:** am einfachsten von allen (HTML/CSS).
- **Nachteile:** Der Rendering-Stack ist WebView2 (Edge/Chromium) mit eigenen Prozessen, daher klar höherer RAM-Bedarf als ein nativer Renderer. Dazu kommt ein zweiter Sprach-/Build-Stack (Frontend), und das Werkzeug passt nicht zu „nur `cargo build`/`cargo install`“. [Tauri-Seite zur App-Größe](https://v2.tauri.app/concept/size/) nennt **keine** Zahlen; RAM-Werte habe ich nicht aus einer Primärquelle geprüft.
- Für ein Overlay mit einer Zahl darauf ist das zu viel Technik.

### 5. `winit` 0.30 + `softbuffer`/`tiny-skia` (eigenes Rendering)

- **winit-API** ([Window](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html)): `set_window_level` („just a hint to the OS“), `set_cursor_hittest` (Windows nicht als „not supported“ gelistet), `drag_window`, `set_decorations`, `set_transparent` („just a hint“), `request_redraw` (unter Windows über `RedrawWindow` / `WM_PAINT`). Fenster-Features also vollständig.
- **Problem Transparenz:** `softbuffer` 0.4.x kennt kein Alpha. Die Alpha-Modi kamen erst mit [PR #321](https://github.com/rust-windowing/softbuffer/pull/321) (gemergt März 2026, Meilenstein v0.5, noch nicht veröffentlicht), und auch dort gilt für **Windows: `Premultiplied`/`Postmultiplied` „not yet supported“**. Alpha pro Pixel ginge nur über ein eigenes `UpdateLayeredWindow` mit einem DIB aus einer `tiny-skia`-Pixmap, also eigener Win32-Code.
- **Textdarstellung** muss man selbst bauen (z. B. Schrift-Rasterizer und Layout). Das ist für eine Ziffernanzeige machbar, aber Theming, Schriften und DPI-Skalierung sind dann komplett eigene Arbeit.
- **Footprint:** vermutlich der kleinste (keine GPU, kein GL-Kontext), aber nicht gemessen. Der Aufwand ist der höchste von allen Kandidaten.
- winit 0.31 ist Beta, was später einen API-Umbruch bedeutet.

### Weitere kurz erwogen (nicht vertieft)

- **Reines Win32 über `windows`-Crate** (ggf. Direct2D/DirectWrite): volle Kontrolle (Layered Window, Per-Pixel-Alpha mit `UpdateLayeredWindow`, gute Textdarstellung über DirectWrite), aber viel `unsafe` und Handarbeit. Taugt als Rückfallebene, falls Alpha pro Pixel mit egui/slint scheitert.
- `floem`, `gpui`, `xilem`, `freya`: nicht geprüft. Windows-Reife bzw. Overlay-Features ohne Primärquellen-Check nicht einschätzbar.

## Rangliste

| Rang | Option | Fenster-Features (Windows) | Transparenz-Risiko | Größe/RAM | Reife | Theming |
|---|---|---|---|---|---|---|
| **1** | **eframe/egui 0.37 + glow** | alles nativ, auch zur Laufzeit | mittel (mit glow i. d. R. OK; GPU/Treiber-abhängig) | ~5,6 MB LTO / ~31 MB RAM (gemessen, #7761) | hoch, monatliche Releases | einfach (Code) |
| 2 | slint 1.19 | Rahmen/Topmost/Hintergrund nativ; Drag/Click-through über `unstable-winit-030` oder HWND | mittel (Skia-wgpu kaputt, femtovg/software ungeprüft) | ungeprüft | hoch | sehr gut (deklarativ) |
| 3 | iced 0.14 | alles nativ | hoch (offener Bug #2525, Workaround tiny-skia) | ungeprüft | mittel (langsamer Takt) | mittel |
| 4 | tauri 2.12 | alles nativ | niedrig–mittel | hoch (WebView2) | hoch | sehr einfach (CSS) |
| 5 | winit + softbuffer/tiny-skia | alles nativ (winit) | hoch (kein Alpha auf Win32 in softbuffer) | minimal | Bausteine reif, Gesamtlösung selbst gebaut | hoher Aufwand |

## Offene Risiken, die der Overlay-Spike (Ticket 02) prüfen sollte

1. **Alpha pro Pixel mit eframe+glow auf der echten Hardware:** transparenter Hintergrund, deckender Text, Windows 11. Auf Hybrid-GPU bzw. NVIDIA/AMD achten (#4451). Alternativ prüfen, ob **einheitliche Deckkraft** über `SetLayeredWindowAttributes(LWA_ALPHA)` am HWND reicht. Dazu sollte die Spec entscheiden, welche Art von „halbtransparent“ gemeint ist.
2. **Immer-im-Vordergrund zur Laufzeit umschalten** (`ViewportCommand::WindowLevel`) und prüfen, ob es wirklich `HWND_NOTOPMOST` setzt; Verhalten bei Fullscreen-Apps.
3. **Click-through** (`MousePassthrough`), egui #2537: Wird das Fenster unsichtbar? Funktioniert der Workaround über den Laufzeit-Befehl? **Wichtig:** Ist click-through an, kann man das Overlay nicht mehr per Maus bedienen. Man braucht also einen anderen Weg zurück (Hotkey/Tray). Das betrifft Ticket 04 „Bedienung“.
4. **Drag** mit `ViewportCommand::StartDrag` bei randlosem Fenster (direkt beim Maus-Drücken auslösen).
5. **Leerlauf-CPU/RAM** bei `request_repaint_after(1 s)` über einige Minuten im Task-Manager messen; Ziel etwa < 1 % CPU, RAM in der Größenordnung von #7761 (~30 MB).
6. **Binärgröße** mit `default-features = false` + `glow` + Release/LTO/strip messen (MSVC-Toolchain).
7. **Taskleiste / Alt+Tab** (`with_taskbar(false)` bzw. `WS_EX_TOOLWINDOW`) und Fokus-Klau beim Klick (`WS_EX_NOACTIVATE`), falls gewünscht.
8. **DPI-Skalierung** bei mehreren Monitoren und Positionsspeicherung.
9. Fallback, falls 1 scheitert: slint mit femtovg/software prüfen oder eframe mit einheitlicher Fenster-Deckkraft.

## Nicht verifiziert

- Binärgröße und RAM von slint, iced, tauri und winit+softbuffer (keine Primärquellen-Messungen gefunden).
- Ob Slints `always-on-top` zur Laufzeit änderbar ist (Doku sagt nichts dazu) und ob femtovg/software-Renderer unter Windows Alpha pro Pixel können.
- Ob iced-master bzw. ein künftiges 0.15 den Windows-Transparenz-Bug behebt.
- Die genaue Win32-Umsetzung von winit 0.30 `set_window_level` / `set_cursor_hittest` (nur indirekt über egui #2537 belegt).
- Die Messwerte in egui #7761 stammen von einem einzelnen Nutzer auf einer Maschine und sind keine offiziellen Benchmarks.
