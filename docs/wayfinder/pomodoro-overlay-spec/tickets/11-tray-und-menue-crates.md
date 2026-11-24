---
title: Crates für Tray-Icon und gemeinsames Menü
labels: [wayfinder:research]
status: closed
assignee: ckr
blocked_by: []
---

# Crates für Tray-Icon und gemeinsames Menü

## Question

Mit welchem Crate bekommt eine Rust-App mit `eframe`/`egui` 0.37 (+ `glow`, winit) unter Windows 11 ein Tray-Icon mit **dynamischem Tooltip** (sekündlich aktualisierte Restzeit) und ein **Menü**, das sich sowohl per Links-/Rechtsklick aufs Tray-Icon als auch per Rechtsklick aufs Overlay öffnen lässt, also **dasselbe Menü an beiden Stellen**? Das Menü braucht Häkchen-Einträge, ausgegraute Einträge, Trenner, ein Untermenü (*Theme ▸*) und zur Laufzeit wechselnden Eintragstext (Primäraktion). Kandidaten: `tray-icon` + `muda` (tauri), `trayicon`, direkt `windows`-rs (`Shell_NotifyIconW`, `TrackPopupMenu`). Zu prüfen: Verträgt sich das Crate mit der Event-Loop von eframe/winit (eigener Thread, Message-Pump)? Kann das native Menü auch an der Mausposition über dem Overlay aufgehen, oder muss das Overlay-Menü in egui nachgebaut werden? Funktioniert es mit Tastatur (`Esc`, `Umschalt+F10`/`Menütaste`)? Bewertet nach: Zuverlässigkeit auf Windows 11, Aufwand, Binärgröße. Ergebnis: Empfehlung mit Begründung.

## Kontext

- Aus *[Bedienung](04-bedienung.md)*: Menüinhalt, Ausgrau-Regel, Tooltip-Inhalt und Tastenkürzel sind dort festgelegt. Bei aktivem Click-through oder ausgeblendetem Overlay ist das Tray-Icon der einzige Weg zurück, deshalb muss es zuverlässig sein.
- Aus *[Overlay-Spike mit der gewählten Technik](02-overlay-spike.md)*: Spike-Code auf Branch `prototype/02-overlay-spike` zum Ausprobieren.

## Resolution

**Empfehlung: `tray-icon` 0.26 mit dem mitgelieferten `muda` 0.21 (tauri-apps), `default-features = false`.** Ein einziges `muda::Menu` hängt am Tray-Icon (`with_menu`, `with_menu_on_left_click(true)`). Das Overlay öffnet **dasselbe** Menü per `show_context_menu_for_hwnd(hwnd, …)` an der Mausposition, ein Nachbau in egui ist nicht nötig. Text (`set_text`), Ausgrauen (`set_enabled`), Häkchen (`set_checked`), Trenner und Untermenü lassen sich zur Laufzeit am selben nativen Menü ändern. Der Tooltip ist ein einfaches `Shell_NotifyIconW(NIM_MODIFY)` über `set_tooltip`, unter 128 Zeichen bleiben.
- **Event-Loop:** kein eigener Thread. Das versteckte Tray-Fenster bekommt seine Nachrichten aus der Message-Pump von winit. Events laufen über `MenuEvent`/`TrayIconEvent::set_event_handler` → Kanal + `ctx.request_repaint()`. Tray-Code gehört in `App::logic`, weil eframe 0.37 das auch bei verstecktem Fenster aufruft.
- **Zuverlässigkeit:** `tray-icon` macht, was Microsoft verlangt (`SetForegroundWindow` vor und `PostMessage(WM_NULL)` nach `TrackPopupMenu`) und meldet sich nach einem Explorer-Neustart (`TaskbarCreated`) neu an. Große Nutzerbasis über tauri.
- **Tastatur:** `Esc` und Pfeiltasten erledigt Windows. Die Menütaste kommt als `egui::Key::Menu` an. `Umschalt+F10` kommt in egui nur als *Loslassen* an, deshalb `key_released(F10)` mit Umschalt prüfen oder `WM_CONTEXTMENU` am Overlay-HWND abfangen. Per Tastatur geöffnet, gehört das Menü an eine feste Stelle des Overlays.
- **Größe:** +161 KB (4,51 → 4,68 MB, Release/LTO/strip). `windows-sys` 0.61 ist schon im Baum.
- **Verworfen:** `trayicon` (Text nur durch Neubau des Menüs, fehlendes `WM_NULL`, `winapi` 0.3, wenige Nutzer), Eigenbau mit windows-sys (gleiches Ergebnis, viel `unsafe`, bleibt Rückfallebene), egui-Menü (fürs Tray bei ausgeblendetem Overlay nicht tragfähig, zwei Umsetzungen).

**Risiken:** Solange ein natives Menü offen ist, **steht die Event-Loop** (gemessen: 4 s ohne `logic()`). Overlay-Zahl und Tooltip frieren dann ein, deshalb muss die Restzeit aus `Instant`-Differenzen kommen und darf keine Ticks zählen. Ein eigener Tray-Thread würde das vermeiden, lohnt sich für v1 aber nicht. Beim Klick schaltet muda ein Häkchen selbst um, die App muss es danach aus ihrem Zustand neu setzen. Popup-Menüs bleiben hell und folgen nicht dem egui-Theme. `tray-icon` hat keine Balloon-/Toast-API, das betrifft *Phasenwechsel-Hinweis*. Single-Instance löst es nicht, das betrifft *Lebenszyklus*. Nicht geprüft: echter Tray-Klick im Überlaufbereich, Live-Aktualisierung eines gerade sichtbaren Tooltips, `logic()`-Takt bei `Visible(false)`.

Details: [Recherche](../research/11-tray-und-menue-crates.md)
