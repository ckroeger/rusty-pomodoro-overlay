# SPEC: Rusty Pomodoro Overlay v1

Stand: 2026-11-27 · Status: **freigegeben**, mit Nachträgen aus der Umsetzung (5.1, 5.2)

Diese Spec beschreibt v1 so, dass ein Agent oder ein Mensch sie **ohne Rückfragen** bauen kann. Begriffe in *Kursiv* stehen in [GLOSSARY.md](GLOSSARY.md) und werden genau so verwendet. Wie es zu den Entscheidungen kam, steht in der [Wayfinder-Karte](docs/wayfinder/pomodoro-overlay-spec/map.md), warum die GUI-Technik gewählt wurde, im [ADR 0001](docs/adr/0001-eframe-egui-mit-glow.md).

Regel für Unklarheiten: **Was hier nicht steht, gehört nicht zu v1.** Werte mit dem Zusatz *Richtwert* darf die Umsetzung anpassen, wenn das Ergebnis besser aussieht. Alles andere ist verbindlich.

---

## 1. Überblick

Ein persönlicher Pomodoro-Timer **nur für Windows 11**. Ein kleines, randloses *Overlay* zeigt die Restzeit der aktuellen *Phase*, ein Tray-Icon bietet dasselbe Menü wie ein Rechtsklick aufs Overlay. Es gibt einen klassischen *Zyklus* (Fokusphasen, kurze Pausen, lange Pause), *Immer-im-Vordergrund* und *Click-through* als Schalter, zwei *eingebaute Themes* und *eigene Themes* als Dateien.

**Verteilung:** nur `cargo build --release` bzw. `cargo install --path .`. Kein Installer, kein Paket.

**Nicht in v1** (siehe Karte, *Out of scope*): Linux/macOS, Statistiken, Aufgabenlisten, Synchronisierung, Installer, frei gestaltbare Overlay-Layouts, globale Hotkeys, Windows-Toasts, gespeicherter Timer-Stand.

---

## 2. Technik

### 2.1 Crate und Build

- Eine **Binär-Crate** `rusty-pomodoro-overlay` (Exe `rusty-pomodoro-overlay.exe`), Edition 2024, kein Workspace, keine getrennte lib.
- Zielplattform `x86_64-pc-windows-msvc`. `#![windows_subsystem = "windows"]` in `main.rs` (kein Konsolenfenster).
- Release-Profil:

  ```toml
  [profile.release]
  lto = true
  codegen-units = 1
  strip = true
  panic = "abort"
  ```

- Ziel für die Größe: unter 6,5 MB (gemessene Summe der Spikes etwa 5,1 MB, dazu zwei Schriften mit zusammen etwa 0,6 MB).

### 2.2 Abhängigkeiten

Geprüft gegen crates.io am 2026-11-26.

| Crate | Version | Features | Wofür |
|---|---|---|---|
| `eframe` | `0.37` | `default-features = false`, `["glow", "default_fonts"]` | Fenster, Rendering, egui |
| `egui` | über eframe | | UI |
| `raw-window-handle` | `0.6` | | HWND aus `eframe::Frame` |
| `tray-icon` | `0.26` | `default-features = false` | Tray-Icon, Tooltip |
| `muda` | `0.21` (über `tray-icon` re-exportiert, `tray_icon::menu`) | | natives Menü |
| `windows-sys` | `0.61` | siehe unten | Plattform |
| `image` | `0.25` | `default-features = false`, `["jpeg", "png"]` | Hintergrundbild |
| `skrifa` | `0.44` | `["std", "autohint_shaping"]` (wie `epaint` 0.37) | Glyphen fürs Tray-Zeichen |
| `vello_cpu` | `0.2` | `["std", "u8_pipeline", "f32_pipeline"]` (wie `epaint` 0.37) | Glyphen rastern |
| `toml_edit` | `0.25` | | Einstellungsdatei und Themes lesen und schreiben |

Hinweise:
- `tray-icon` 0.27 / `muda` 0.22 sind seit 2026-11-25 auf crates.io. Geprüft ist **0.26 / 0.21**, dabei bleibt v1. Ein Update ist ein eigener Schritt nach v1.
- `skrifa` und `vello_cpu` müssen **genau die Versionen und Features von `epaint`** haben, sonst landen sie doppelt im Baum. Bei jedem egui-Update nachziehen (`cargo tree -d` muss für beide leer sein).
- Kein Crate darf `NvOptimusEnablement`/`AmdPowerXpressRequestHighPerformance` exportieren oder `nvapi`/`nvcuda`/`opencl` statisch einbinden.
- Verboten in v1: `rodio`, `notify`, `windows` (windows-rs), Logging-Crates, `serde` für die Einstellungsdatei (gelesen wird mit `toml_edit`).

`windows-sys`-Features: `Win32_Foundation`, `Win32_System_Threading` (Mutex, Event), `Win32_Security` (nötig für `CreateMutexW`/`CreateEventW`), `Win32_Graphics_Dwm` (Ecken, Rahmen), `Win32_UI_Input_KeyboardAndMouse` (`GetAsyncKeyState`), `Win32_System_Registry` (Autostart), `Win32_UI_Shell` (`SHQueryUserNotificationState`, `ShellExecuteW`, `SetWindowSubclass`), `Win32_Media_Audio` (`PlaySoundW`), `Win32_UI_WindowsAndMessaging` (Fensterstile, `SetWindowPos`, `ShowWindow`, `MessageBoxW`, `GetSystemMetricsForDpi`), `Win32_Graphics_Gdi` (`MonitorFromRect`, `GetMonitorInfoW`), `Win32_UI_HiDpi` (`GetDpiForWindow`). Fehlt beim Bauen ein Feature, wird es ergänzt.

### 2.3 Eingebettete Dateien

| Datei | Inhalt |
|---|---|
| `assets/fokus_endet.wav` | Klang *Fokusphase endet* |
| `assets/pause_endet.wav` | Klang *Pause endet* |
| `assets/einstellungen.toml` | kommentierte Vorlage der Einstellungsdatei (Abschnitt 8.2) |
| `assets/Inter-Regular-tnum.ttf` | Schrift *Inter* mit fest eingebauten gleich breiten Ziffern |
| `assets/JetBrainsMono-Regular.ttf` | Schrift *JetBrains Mono* |
| `assets/OFL-Inter.txt`, `assets/OFL-JetBrainsMono.txt` | Lizenztexte (SIL Open Font License 1.1) |

Die Klänge sind selbst erzeugt, damit es keine Lizenzfragen gibt: WAV, PCM 16 bit, mono, 22 050 Hz, Sinustöne mit Amplitude 0,5 und 10 ms Ein-/Ausblenden je Ton.
- *Fokusphase endet*: 880 Hz, dann 660 Hz, je 180 ms, 60 ms Stille dazwischen (fallend: „ausruhen“).
- *Pause endet*: 660 Hz, 880 Hz, 1100 Hz, je 150 ms, 50 ms Stille dazwischen (steigend: „los geht’s“).

Erzeugt werden sie einmalig mit `examples/klaenge_erzeugen.rs` (ohne zusätzliche Crates, schreibt die WAV-Kopfdaten selbst), die fertigen Dateien liegen im Repo.

**Schriften:** Ein Theme wählt eine von drei Schriften (Abschnitt 7.1). Alle haben gleich breite Ziffern 0–9 und `+`, die Anzeige zappelt also nicht:

| Schlüssel | Schrift | Herkunft |
|---|---|---|
| `ubuntu` | Ubuntu Light | `default_fonts` von egui, keine Zusatzdatei (Voreinstellung) |
| `inter` | Inter Regular, `tnum` eingefroren | Inter 4.x (eingecheckt: 4.1, `extras/ttf/Inter-Regular.ttf`) von github.com/rsms/inter. *Inter* hat von sich aus proportionale Ziffern; ob egui das Feature `tnum` anwendet, ist nicht gesichert. Deshalb wird es einmalig fest eingebaut: `pyftfeatfreeze -f tnum -S -U Tnum Inter-Regular.ttf assets/Inter-Regular-tnum.ttf` (Paket `opentype-feature-freezer`). |
| `jetbrains_mono` | JetBrains Mono Regular | 2.x (eingecheckt: 2.304) von github.com/JetBrains/JetBrainsMono, ist ohnehin gleichbreit |

- Dass die Ziffern und `+` in den beiden Dateien gleich breit sind, prüft ein Test in `schriften.rs` (Vorschubbreiten über `skrifa`).
- Eingebunden über `egui::FontDefinitions` als eigene Familien `FontFamily::Name("inter")` bzw. `("jetbrains_mono")`, jeweils mit den egui-Standardschriften als Rückfall. Das Overlay nutzt für Zeit und Phasenwort die Familie des aktiven Themes. Das Einstellungsfenster bleibt bei der egui-Standardschrift.
- Pausen- und Startsymbol werden als Formen gezeichnet, nicht als Zeichen.

---

## 3. Phasenablauf

### 3.1 Phasen und Zyklus

- *Phasen*: *Fokusphase*, kurze Pause, lange Pause.
- Ein *Zyklus* besteht aus *n* Fokusphasen (*Fokusphasen pro Zyklus*). Nach jeder Fokusphase außer der letzten folgt eine kurze Pause, nach der letzten die lange Pause. Danach beginnt ein neuer Zyklus mit Fokusphase 1. Bei *n* = 1 folgt auf jede Fokusphase die lange Pause.
- Einstellungen und Voreinstellungen:

  | Wert | Bereich | Voreinstellung |
  |---|---|---|
  | Dauer Fokusphase | 1–180 ganze Minuten | 25 |
  | Dauer kurze Pause | 1–180 ganze Minuten | 5 |
  | Dauer lange Pause | 1–180 ganze Minuten | 15 |
  | Fokusphasen pro Zyklus | 1–12 | 4 |
  | Pause automatisch starten (Fokus→Pause) | an/aus | an |
  | Fokus automatisch starten (Pause→Fokus) | an/aus | aus |

### 3.2 Zustände

Die aktuelle Phase ist immer in genau einem Zustand:
- *läuft*: Die Restzeit läuft ab.
- *angehalten*: Die Restzeit steht und bleibt erhalten.
- *bereit*: Die Phase steht fest, es ist noch keine Zeit von ihr verstrichen. Ist *bereit* durch Ablauf der vorigen Phase entstanden, läuft die *Überzeit* mit.

Beim App-Start ist immer Fokusphase 1 *bereit*, ohne Überzeit. Es wird **kein Timer-Stand gespeichert**.

### 3.3 Übergänge

| Auslöser | aus | nach | Hinweis |
|---|---|---|---|
| *Primäraktion* „Starten“ | *bereit* | *läuft* (volle Dauer) | nein |
| *Primäraktion* „Anhalten“ | *läuft* | *angehalten* | nein |
| *Primäraktion* „Fortsetzen“ | *angehalten* | *läuft* | nein |
| Restzeit läuft ab | *läuft* | nächste Phase: *läuft*, wenn der Auto-Start-Schalter dieser Richtung an ist, sonst *bereit* mit Überzeit ab dem Ablaufzeitpunkt | **ja** |
| *Überspringen* | jeder Zustand | nächste Phase: *läuft* oder *bereit* nach dem Auto-Start-Schalter, **ohne** Überzeit | nein |
| *Phase zurücksetzen* | jeder Zustand | dieselbe Phase *bereit*, Dauer nach aktueller Einstellung, ohne Überzeit; *Zyklus-Fortschritt* bleibt | nein |
| *Neu beginnen* | jeder Zustand | neuer Zyklus, Fokusphase 1 *bereit*, ohne Überzeit, alle aktuellen Einstellungen | nein |

- Eine übersprungene Fokusphase zählt für den *Zyklus-Fortschritt* wie eine abgelaufene.
- „Richtung“: Fokusphase → kurze oder lange Pause nutzt *Pause automatisch starten*, kurze oder lange Pause → Fokusphase nutzt *Fokus automatisch starten*.
- Die *Überzeit* ist nur eine Anzeige. Sie wirkt nicht auf den Ablauf.

### 3.4 Zeitbasis

- Grundlage ist `std::time::Instant`. Unter Windows zählt es Standby und Ruhezustand mit (im Spike gemessen), also läuft der Ablauf wie mit der Wanduhr. Sperren (Win+L) hat keine Wirkung. Es gibt kein automatisches Anhalten.
- Restzeit und Überzeit werden **immer aus Zeitpunkten berechnet** (`ende - jetzt`, `jetzt - abgelaufen_um`) und nie durch Zählen von Takten. Grund: Bei offenem nativem Menü steht die Event-Loop sekundenlang, und sichtbar kommen beliebig viele Zusatzaufrufe.
- **Nachholen:** Hat ein Zeitsprung (Standby) dazu geführt, dass eine oder mehrere Phasen abgelaufen sind, rechnet der nächste Takt den Stand so nach, als wäre die App wach gewesen (über automatische Übergänge hinweg, mit den echten Ablaufzeitpunkten). Endet das Nachrechnen in *bereit*, beginnt die Überzeit am Ablaufzeitpunkt der letzten abgelaufenen Phase. Es entsteht **genau ein** Ereignis `PhaseAbgelaufen` für den **letzten** Übergang.

### 3.5 Geänderte Werte

- Die Auto-Start-Schalter gelten **sofort** (für den nächsten Übergang).
- Eine geänderte Dauer gilt ab der **nächsten Phase dieses Typs**. Eine Phase in *läuft* oder *angehalten* behält ihre Dauer. Eine Phase in *bereit* hat noch nicht begonnen und übernimmt die neue Dauer **sofort**.
- *Fokusphasen pro Zyklus* gilt ab dem **nächsten Zyklus**. Steht der aktuelle Zyklus noch ganz am Anfang (Fokusphase 1 *bereit*, nichts erledigt), gilt der neue Wert sofort.
- *Phase zurücksetzen* übernimmt die neue Dauer, *Neu beginnen* alle neuen Werte.

---

## 4. Bedienung

### 4.1 Wege

- **Overlay:** Linksklick, Ziehen, Rechtsklick, `Strg`+Mausrad, lokale Tasten.
- **Tray-Icon:** Links- und Rechtsklick öffnen das Menü, der Tooltip zeigt den Stand.
- **Keine globalen Hotkeys.**

### 4.2 Maus auf dem Overlay

- **Linksklick** ohne Bewegung (Schwelle 4 logische Pixel, *Richtwert*) löst die *Primäraktion* aus: aus *bereit* Starten, aus *läuft* Anhalten, aus *angehalten* Fortsetzen.
- **Ziehen** über die Schwelle hinaus verschiebt das Overlay (`ViewportCommand::StartDrag`), frei und ohne Einrasten. Nach dem Loslassen wird es in den Arbeitsbereich geschoben (Abschnitt 5.6).
- **Rechtsklick** öffnet das gemeinsame Menü an der Mausposition.
- **`Strg`+Mausrad** ändert die Größe (Abschnitt 5.5).
- Hovern bewirkt nichts (kein Tooltip, keine Hervorhebung).

### 4.3 Lokale Tasten

Nur wenn das Overlay den Fokus hat, fest, nicht konfigurierbar:
- `Leertaste`: *Primäraktion*.
- `Umschalt+F10` oder `Menütaste`: öffnet das Menü an der **oberen linken Ecke** des Overlays. (`Umschalt+F10` kommt in egui nur als Loslassen an: `key_released(F10)` mit gedrückter Umschalttaste prüfen. Die Menütaste kommt als `egui::Key::Menu`.)
- `Esc` schließt das offene Menü (erledigt Windows).
- Überspringen, Phase zurücksetzen, Neu beginnen und Beenden haben **bewusst kein Kürzel**.

### 4.4 Das gemeinsame Menü

Ein einziges natives `muda::Menu`. Es hängt am Tray-Icon (`with_menu`, `with_menu_on_left_click(true)`), das Overlay öffnet **dasselbe** Menü mit `show_context_menu_for_hwnd`. Einträge in dieser Reihenfolge:

```
⚠ <Fehlerzeile>               (nur bei Fehlern, siehe 9)
─────────                      (nur bei Fehlern)
<Primäraktion>                 Starten / Anhalten / Fortsetzen
Überspringen
Phase zurücksetzen
Neu beginnen
─────────
[✓] Immer im Vordergrund
[ ] Click-through
[✓] Overlay anzeigen
[✓] Ton
Theme ▸
Einstellungen …
─────────
Beenden
```

- **Ausgegraut** ist ein Eintrag, wenn er den Stand nicht ändern würde:
  - *Phase zurücksetzen*: wenn die Phase *bereit* ist, ihre Dauer der aktuellen Einstellung entspricht und keine Überzeit läuft.
  - *Neu beginnen*: wenn Fokusphase 1 *bereit* ist, ohne Überzeit, ohne erledigte Fokusphase.
  - *Primäraktion* und *Überspringen* sind nie ausgegraut.
- **Keine Rückfragen**, auch nicht bei *Neu beginnen* und *Beenden*.
- Häkchen: `muda` schaltet ein Häkchen beim Klick selbst um. Die App setzt danach **alle** Einträge aus ihrem Zustand neu (`Menuemodell`, Abschnitt 11.4).
- *Click-through* lässt sich über beide Wege einschalten. Ausschalten geht praktisch nur über das Tray-Icon, weil das Overlay dann keine Klicks bekommt.
- *Overlay anzeigen*: Ausgeblendet läuft der Ablauf weiter, erreichbar bleibt die App über das Tray-Icon.
- *Ton* ist derselbe Schalter wie im Einstellungsfenster.
- *Theme ▸* siehe Abschnitt 7.4, *Einstellungen …* siehe Abschnitt 8.5, *Beenden* siehe Abschnitt 10.4.
- Popup-Menüs sind nativ und hell, sie folgen dem Theme nicht.

### 4.5 Tray-Tooltip

Eine Zeile im Format des Overlays, jede Sekunde neu gesetzt (auch bei ausgeblendetem Overlay):

| Lage | Tooltip |
|---|---|
| Fokusphase 3 von 4 läuft | `Fokus 3/4 · läuft · 12:34` |
| kurze Pause nach Fokusphase 2, angehalten | `Pause nach 2/4 · angehalten · 03:10` |
| lange Pause bereit mit Überzeit | `Lange Pause · bereit · +07:00` |
| Fokusphase 1 bereit ohne Überzeit | `Fokus 1/4 · bereit · 25:00` |

Bei Fehlern in den Einstellungen kommt eine zweite Zeile `⚠ Fehler in den Einstellungen` dazu. Der Tooltip bleibt unter 128 Zeichen.

---

## 5. Overlay

### 5.1 Fenster

- Randlos (`with_decorations(false)`), transparent (`with_transparent(true)`, `clear_color` = `[0, 0, 0, 0]`), nicht in der Größe ziehbar (`with_resizable(false)`).
- **Alpha pro Pixel:** Nur der Hintergrund ist durchsichtig, Ziffern, Punkte, Balken und Symbole sind immer deckend. Keine einheitliche Fenster-Deckkraft.
- **Keine Taskleisten-Schaltfläche, nicht in Alt+Tab:** `WS_EX_TOOLWINDOW` am HWND (über `plattform`), dazu `with_taskbar(false)`. winit setzt den erweiterten Stil bei jeder eigenen Änderung (Immer-im-Vordergrund, Click-through, Sichtbarkeit) komplett neu und lässt `WS_EX_TOOLWINDOW` dabei weg. Deshalb fängt die Unterklasse der Fensterprozedur (dieselbe wie für `WM_ENDSESSION`, Abschnitt 10.4) `WM_STYLECHANGING` für `GWL_EXSTYLE` ab und erzwingt dort `WS_EX_TOOLWINDOW` ohne `WS_EX_APPWINDOW`.
- **Ecken und Rahmen:** Die Ecken rundet Windows 11 (`DWMWA_WINDOW_CORNER_PREFERENCE` = `DWMWCP_ROUND`), die helle Rahmenlinie von Windows ist aus (`DWMWA_BORDER_COLOR` = `DWMWA_COLOR_NONE`). Das Overlay zeichnet seine Fläche eckig. Selbst gezeichnete runde Ecken gehen nicht: Pixel außerhalb davon erscheinen schwarz statt durchsichtig (gesehen auf NVIDIA RTX A2000 mit Intel UHD, Ursache unbekannt).
- **Nimmt sich nie selbst den Fokus**: nicht beim Start, nicht beim Phasenwechsel, nicht beim Einblenden. Das Fenster wird unsichtbar erzeugt (`with_visible(false)`, `with_active(false)`) und im ersten `logic`-Aufruf positioniert. Zeigen tut es eframe nach dem ersten Bild selbst; wegen `with_active(false)` geschieht das ohne Aktivierung (`SW_SHOWNOACTIVATE`). Jedes spätere Ein- und Ausblenden läuft **nur** über `plattform` (`ShowWindow(SW_SHOWNOACTIVATE)` bzw. `SW_HIDE`), **nie** über `ViewportCommand::Visible`: winit zeigt ab dem zweiten Mal mit `SW_SHOW` und nimmt damit den Fokus, und sein eigener Merker käme aus dem Takt. Die App vergleicht in jedem Takt den Sollwert mit `IsWindowVisible`; so wird ein beim Start ausgeblendetes Overlay nach dem ersten Bild wieder versteckt (es kann ein Bild lang aufblitzen). Den Fokus bekommt das Overlay nur durch Klick, Ziehen oder Rechtsklick.
- **Immer-im-Vordergrund** über `ViewportCommand::WindowLevel(AlwaysOnTop | Normal)`, auch zur Laufzeit. Voreinstellung an.
- **Click-through** über `ViewportCommand::MousePassthrough(true/false)`, **erst zur Laufzeit** gesetzt, nie im `ViewportBuilder` (Workaround für egui #2537). Beim Start immer aus, wird nicht gespeichert. Bei aktivem Click-through wirken Mausrad und Klicks nicht.
- **Alt+F4** auf dem Overlay wird verworfen: Kommt `close_requested` ohne gesetztes `beenden`-Flag, sendet die App `ViewportCommand::CancelClose`.

### 5.2 Elemente und Aufbau

Grundgröße bei 100 %: **180 × 90 logische Pixel** (*Richtwert*). Die Ecken rundet Windows (Abschnitt 5.1, 8 px bei 96 dpi, mit der Anzeigeskalierung, aber nicht mit der Skalierung des Overlays); der Balken hält 10 px Abstand zu den Seiten. Von oben nach unten:

1. **Zeit**, groß und mittig, etwa 40 px Schrifthöhe (*Richtwert*). Links davon bei *angehalten* das Pausensymbol (zwei senkrechte Balken), bei *bereit* das Startsymbol (Dreieck nach rechts), in der Farbe der Zeit.
2. **Phasenwort** darunter, etwa 13 px (*Richtwert*), in der *Akzentfarbe*: „Fokus“, „Pause“, „Lange Pause“.
3. **Zyklus-Punkte** auf einer Zeile, mittig.
4. **Fortschrittsbalken** am unteren Rand, etwa 3 px hoch (*Richtwert*), über die ganze Breite.

Der Zustand hat kein eigenes Element. Das Layout ist fest.

### 5.3 Zeitformat

- Immer `MM:SS`. Die Minuten dürfen mehr als zwei Stellen haben (`180:00`).
- Restzeit wird **aufgerundet** auf ganze Sekunden: Start bei `25:00`, `00:00` erst im Moment des Ablaufs.
- Überzeit mit `+` davor, **abgerundet**: beginnt bei `+00:00`, z. B. `+125:30`.

### 5.4 Darstellung je Zustand

| Zustand | Zeit | Symbol | Phasenwort, Punkte | Balken |
|---|---|---|---|---|
| *läuft* | Restzeit, normal | – | normal | Anteil verstrichen, Akzentfarbe |
| *angehalten* | Restzeit, gedimmt | Pause, gedimmt | gedimmt | Anteil verstrichen, gedimmt |
| *bereit* ohne Überzeit | volle Dauer, gedimmt | Start, gedimmt | gedimmt | leer |
| *bereit* mit Überzeit | Überzeit `+MM:SS`, **nicht gedimmt** | Start, nicht gedimmt | Farbe und Wort der aktuellen (nächsten) Phase, normal | leer |

- **Gedimmt** heißt: die Farbe wird zu 50 % (*fester Faktor*, gleich für alle Themes) mit der RGB-Farbe von `hintergrund` gemischt.
- **Zyklus-Punkte:** ein Punkt je Fokusphase des aktuellen Zyklus. **Voll** (`vordergrund`) für eine erledigte Fokusphase, **hervorgehoben** (Ring in der Akzentfarbe, innen leer) für die aktuelle Fokusphase, **leer** (`gedaempft`) für eine offene. In Pausen ist kein Punkt hervorgehoben. Bei bis zu 12 Punkten werden sie kleiner, damit sie in eine Zeile passen. Beispiele bei 4: Fokusphase 1 `◐○○○`, kurze Pause nach 2 `●●○○`, Fokusphase 3 `●●◐○`, lange Pause `●●●●`.
- **Fortschrittsbalken:** Spur in `gedaempft`, Füllung in der Akzentfarbe, Anteil `0.0..=1.0` der verstrichenen Phasendauer.

### 5.5 Größe

- Skalierung **50 % bis 300 %**, Voreinstellung 100 %, nur über **`Strg`+Mausrad**: **10 % pro Rastung**, proportional (inklusive Schrift, gleiches Seitenverhältnis), um die **Mitte** des Overlays herum. Kein Regler, keine Ziehränder.
- Umsetzung: `overlay::zeichnen` bekommt den Faktor und skaliert Maße und Schriftgrößen selbst; neue Fenstergröße = Grundgröße × Skalierung × DPI-Faktor. Kein `ctx.set_zoom_factor`: Der Zoom gilt in egui für alle Viewports und träfe auch das Einstellungsfenster. Die Abstände des Balkens zu den Ecken skalieren nicht mit, weil Windows die Ecken rundet. Danach wird das Overlay in den Arbeitsbereich geschoben (Abschnitt 5.6).
- Die Windows-Anzeigeskalierung (DPI) wird beachtet (Per-Monitor-DPI wie in winit).
- Die Skalierung wird gespeichert (verzögert, Abschnitt 8.3).

### 5.6 Position

- **Erster Start** (kein Wert `position` in der Datei): unten rechts im **Arbeitsbereich des Hauptmonitors** (ohne Taskleiste), 24 logische Pixel Abstand zu rechtem und unterem Rand.
- Gespeichert wird die obere linke Ecke in **physischen Pixeln des virtuellen Bildschirms**. Gesetzt und gelesen wird die Lage über `SetWindowPos`/`GetWindowRect` am HWND (über `plattform`), nicht über egui-Koordinaten, damit gemischte DPI keine Umrechnungsfehler erzeugt.
- **In den Arbeitsbereich schieben:** Liegt das Overlay nicht vollständig im Arbeitsbereich eines Monitors, wird es in den Arbeitsbereich des nächstgelegenen Monitors (`MonitorFromRect(MONITOR_DEFAULTTONEAREST)`) geschoben. Liegt es auf keinem Monitor mehr (`MONITOR_DEFAULTTONULL`), kommt es an die Startposition.
- Geprüft wird **im Sekundentakt** und nach jedem Ziehen und Skalieren, solange keine Maustaste gedrückt ist. So greift die Regel auch, wenn ein Monitor abgesteckt oder die Auflösung geändert wird. Ändert sich die Position, geht sie verzögert in die Einstellungsdatei.

---

## 6. Hinweis bei Phasenwechsel

### 6.1 Auslöser

Ein *Hinweis* kommt **nur, wenn die Restzeit einer Phase abläuft** (Ereignis `PhaseAbgelaufen`), egal ob die nächste Phase automatisch startet oder *bereit* ist. Kein Hinweis beim Überspringen, Starten, *Phase zurücksetzen* oder *Neu beginnen*. Nach dem Nachholen (Standby) kommt genau ein Hinweis, sofort, mit Klang und Farbe des letzten Übergangs.

### 6.2 Kanäle

1. **Ton** (Schalter *Ton*, Voreinstellung an):
   - Endet eine Fokusphase: `fokus_endet.wav`. Endet eine kurze oder lange Pause: `pause_endet.wav`.
   - Abgespielt mit `PlaySoundW(BUFFER.as_ptr(), 0, SND_MEMORY | SND_ASYNC | SND_NODEFAULT)` auf einen `static … = include_bytes!(…)`-Puffer, **ohne** `SND_SYSTEM` (dann erscheint die App im Lautstärkemixer). Kein eigener Thread, der Rückgabewert wird ignoriert. Kein eigener Lautstärkeregler.
   - **Nicht stören:** Direkt vor dem Ton fragt die App `SHQueryUserNotificationState` ab (jedes Mal neu, nicht zwischenspeichern). Kein Ton bei `QUNS_BUSY`, `QUNS_RUNNING_D3D_FULL_SCREEN` und `QUNS_PRESENTATION_MODE`. Bei allen anderen Werten und bei Fehlern wird gespielt. *Bitte nicht stören* von Windows 11 ist nicht dokumentiert abfragbar und wird ignoriert.
2. **Aufblinken im Overlay** (Schalter *Signal im Overlay*, Voreinstellung an): dreimal, insgesamt 2 s. Der Wert `aufblinken(jetzt)` läuft in drei gleich langen Pulsen jeweils als `sin²` von 0 auf 1 und zurück. Die Hintergrundfläche wird mit diesem Wert × 60 % (*Richtwert*) zur Akzentfarbe der **neuen** Phase gemischt. Danach normale Anzeige. Es blinkt auch bei aktivem Click-through. Bei *bereit* blinkt nichts dauerhaft.
3. **Tray-Icon je Phase:** immer aktiv, nicht abschaltbar (Abschnitt 7.5).

### 6.3 Einblenden

Schalter *Overlay bei Phasenwechsel einblenden* (Voreinstellung aus). Ist er an und das Overlay *ausgeblendet*, wird es beim Hinweis eingeblendet (ohne Fokus), blinkt auf (wenn *Signal im Overlay* an ist) und bleibt sichtbar. *Overlay anzeigen* wird dabei auf an gesetzt und gespeichert.

---

## 7. Theming

### 7.1 Felder eines Themes

| Feld | Typ | Bedeutung |
|---|---|---|
| `name` | Text, optional | Name im Menü; fehlt er, gilt der Dateiname ohne Endung |
| `schrift` | `"ubuntu"`, `"inter"` oder `"jetbrains_mono"` | Schrift für Zeit und Phasenwort (Abschnitt 2.3) |
| `fokus` | Farbe | Akzentfarbe Fokusphase |
| `kurze_pause` | Farbe | Akzentfarbe kurze Pause |
| `lange_pause` | Farbe | Akzentfarbe lange Pause |
| `hintergrund` | Farbe mit Deckkraft | Fläche des Overlays |
| `vordergrund` | Farbe | Zeit, volle Zyklus-Punkte, Symbole |
| `gedaempft` | Farbe | leere Zyklus-Punkte, Spur des Balkens |
| `bild` | Pfad, optional | Hintergrundbild JPG oder PNG, relativ zur Theme-Datei |
| `tray_zeichen` | Text, optional | ein Unicode-Zeichen statt des Kreises im Tray |

- Farben als `"#RRGGBB"` oder `"#RRGGBBAA"`. Die Deckkraft von `hintergrund` steht im Alphakanal (`"#1E1E1ED9"` = 85 %). Bei den anderen Farben wird ein Alphakanal ignoriert (immer deckend).
- Die Akzentfarbe gilt für Phasenwort, Füllung des Balkens, Ring um den aktuellen Zyklus-Punkt, Aufblinken und Tray-Icon.
- Layout und Größe gehören **nicht** zum Theme. Die Schrift ist nur aus den drei eingebauten wählbar, beliebige Schriftdateien gibt es nicht.

### 7.2 Eingebaute Themes

Eingebettet, nicht änderbar, ohne Bild, mit Kreis im Tray. *Dunkel* ist die Voreinstellung. Die App folgt **nicht** dem Hell-/Dunkelmodus von Windows.

| Feld | Dunkel | Hell |
|---|---|---|
| `hintergrund` | `#1E1E1ED9` (85 %) | `#F5F5F5E6` (90 %) |
| `vordergrund` | `#F0F0F0` | `#202020` |
| `gedaempft` | `#5A5A5A` | `#B0B0B0` |
| `fokus` | `#E5534B` | `#C62828` |
| `kurze_pause` | `#4CAF50` | `#2E7D32` |
| `lange_pause` | `#42A5F5` | `#1565C0` |
| `schrift` | `ubuntu` | `ubuntu` |

### 7.3 Eigene Themes

- Je eine TOML-Datei `*.toml` im **Theme-Ordner** `%USERPROFILE%\.rusty_pomodoro_overlay\themes\`.
- Fehlende Felder **erben von *Dunkel***. Ein ungültiges oder unbekanntes Feld wird ignoriert, dann gilt der Wert aus *Dunkel*.
- Gespeichert wird ein Theme über seinen **Schlüssel**: `dunkel` und `hell` für die eingebauten, sonst der Dateiname ohne Endung (`pipboy` für `pipboy.toml`). Die Schlüssel `dunkel` und `hell` sind reserviert. Eine eigene Datei mit diesem Namen erscheint ausgegraut mit dem Zusatz `(Name reserviert)`.
- Beispiel `pipboy.toml`:

  ```toml
  name = "Pip-Boy"
  schrift = "jetbrains_mono"
  bild = "pipboy.jpg"
  tray_zeichen = "☢"
  fokus = "#33FF66"
  kurze_pause = "#99FFAA"
  lange_pause = "#66CCFF"
  hintergrund = "#000000CC"
  ```

### 7.4 Menü *Theme ▸*

```
[✓] Dunkel
[ ] Hell
─────────
[ ] <eigene Themes, alphabetisch nach Name>
─────────
Theme-Ordner öffnen
```

- Das aktive Theme hat ein Häkchen. Ein Klick wechselt **sofort** und speichert die Wahl. Ein Klick auf das aktive Theme lädt es neu.
- Der Ordner wird **im Sekundentakt** neu gelesen, kein Datei-Watcher: nur Dateinamen und Änderungszeitpunkte; neu geparst wird nur, was sich geändert hat. Das Untermenü ist beim Öffnen also höchstens 1 s alt. (Direkt beim Öffnen lesen geht beim Tray-Menü nicht: `tray-icon` öffnet es selbst, und solange es offen ist, steht die Event-Loop.)
- *Theme-Ordner öffnen* legt den Ordner an, falls er fehlt, und öffnet ihn im Explorer (`ShellExecuteW` mit `explore`).
- Fehler stehen nur als Zusatz im Menütext:
  - Datei nicht lesbar (Syntaxfehler): Eintrag ausgegraut, z. B. `Pip-Boy (Fehler in Zeile 4)`. Als Name dient dann der Dateiname.
  - Bild fehlt oder lässt sich nicht dekodieren: Theme bleibt wählbar, verwendet die Farbe `hintergrund`, Menütext z. B. `Pip-Boy (Bild fehlt)`.
- Ist das gespeicherte aktive Theme beim Start oder nach einer Änderung weg oder kaputt, verwendet die App *Dunkel*. Der gespeicherte Schlüssel bleibt erhalten und gilt wieder, sobald die Datei repariert ist.

### 7.5 Hintergrundbild

- Es **ersetzt** die Füllfarbe von `hintergrund`. Die **Deckkraft von `hintergrund` gilt für das Bild**, bei PNG wird der Alphakanal damit multipliziert.
- Es **füllt** das Overlay: proportional skaliert, bis die Fläche ganz bedeckt ist, und **mittig beschnitten** (über das UV-Rechteck). Keine Verzerrung, die Form des Overlays bleibt.
- Dekodieren in einem **kurzen Hintergrund-Thread** (dem einzigen eigenen Thread der App): `ImageReader::open` → `with_guessed_format` → `into_decoder` → `orientation` → `DynamicImage::from_decoder` → `apply_orientation` → bei Bedarf `thumbnail_exact` → `into_rgba8`. Ergebnis per Kanal an die UI, dann `request_repaint()`. Bis dahin gilt die Farbe `hintergrund`. Jeder Fehler wird zu `(Bild fehlt)`.
- **Verkleinern ist Pflicht:** Ziel ist die Fläche des Overlays bei 300 % × DPI-Faktor des aktuellen Monitors. Größere Bilder werden verkleinert, nie vergrößert. Sonst bricht `egui_glow` bei zu großen Texturen ab.
- Textur mit `ColorImage::from_rgba_unmultiplied` und `TextureOptions { mipmap_mode: Some(TextureFilter::Linear), ..TextureOptions::LINEAR }`. Gezeichnet mit dem Farbton `Color32::from_white_alpha(deckkraft)`.

### 7.6 Tray-Icon

- Größe `GetSystemMetricsForDpi(SM_CXSMICON, dpi)` (16/20/24/32 px), RGBA nicht vormultipliziert (`Icon::from_rgba`).
- **Ohne Tray-Zeichen:** gefüllter Kreis in der Akzentfarbe der aktuellen Phase, mit 1 px Rand Abstand.
- **Mit Tray-Zeichen:** ein Zeichen für alle Phasen, **einfarbig** in der Akzentfarbe gezeichnet. Die Kontur kommt aus *Segoe UI Symbol*, sonst aus *Segoe UI Emoji* (`%WINDIR%\Fonts\seguisym.ttf`, `seguiemj.ttf`, zur Laufzeit gelesen, nicht eingebettet) und wird mit `skrifa` + `vello_cpu` gerastert, auf ihren Umriss eingepasst und zentriert. Farbige Emoji werden einfarbig (nur `glyf`, `COLR`/`CPAL` werden nicht beachtet).
- Ein Zeichen ist **ein** Unicode-Skalar. Variantenselektoren U+FE0E/U+FE0F am Ende werden abgeschnitten. **Nicht darstellbar** (Rückfall auf den Kreis): leeres Feld, mehr als ein Skalar (ZWJ-Folgen, Hautfarben, Flaggen, Tastenkappen), keine der beiden `cmap` kennt das Zeichen, oder die Kontur ist leer.
- Gedimmt bei *angehalten* und *bereit* (wie im Overlay, Mischung zu `hintergrund`).
- Gerastert wird einmal pro Theme und Größe als Maske, danach nur umgefärbt. Das Icon wird nur bei Änderung von Phase, gedimmt oder Theme neu gesetzt.

---

## 8. Einstellungen

### 8.1 Ort

```
%USERPROFILE%\.rusty_pomodoro_overlay\
├── einstellungen.toml
├── absturz.txt          (nur nach einem Absturz)
└── themes\
    ├── pipboy.toml
    └── pipboy.jpg
```

Kein `%APPDATA%`, kein portabler Modus.

### 8.2 Die Datei

Beim ersten Start legt die App Ordner und Datei an, als Kopie von `assets/einstellungen.toml`:

```toml
# Rusty Pomodoro Overlay – Einstellungen
# Die App liest diese Datei jede Sekunde neu, wenn sie sich geändert hat.
# Ungültige Werte ersetzt sie durch die Voreinstellung und meldet das im Menü.
# Wer ganz neu anfangen will, löscht diese Datei.

# Dauer der Phasen in ganzen Minuten (1 bis 180)
fokus_minuten = 25
kurze_pause_minuten = 5
lange_pause_minuten = 15

# Fokusphasen pro Zyklus (1 bis 12); danach kommt die lange Pause
fokusphasen_pro_zyklus = 4

# Nächste Phase automatisch starten (true) oder bereit stellen (false)
pause_automatisch_starten = true
fokus_automatisch_starten = false

# Hinweis, wenn eine Phase abläuft
ton = true
signal_im_overlay = true
overlay_bei_phasenwechsel_einblenden = false

# Theme: "dunkel", "hell" oder der Dateiname (ohne .toml) im Ordner themes
theme = "dunkel"

# Overlay
immer_im_vordergrund = true
overlay_anzeigen = true
# Größe in Prozent (50 bis 300)
skalierung = 100
# Obere linke Ecke in Bildschirmpixeln; fehlt der Wert, erscheint das Overlay unten rechts
# position = [1600, 900]
```

- Typen: Zahlen als TOML-Integer, Schalter als Boolean, `theme` als String, `position` als Array aus zwei Integern.
- *Click-through* steht **nicht** in der Datei (beim Start immer aus). *Mit Windows starten* steht **nicht** in der Datei (maßgeblich ist der Registry-Eintrag, Abschnitt 10.3).
- Ungültiger Wert (falscher Typ, außerhalb des Bereichs): Voreinstellung verwenden, melden. Jeder Wert wird **einzeln** geprüft. Unbekannter Schlüssel: ignorieren, melden. Fehlender Schlüssel: Voreinstellung, **nicht** melden.

### 8.3 Schreiben

- Mit `toml_edit`: Kommentare, Reihenfolge und Formatierung bleiben, nur der geänderte Wert wird ersetzt bzw. am Ende ergänzt.
- Vor jedem Schreiben liest die App die Datei frisch ein und ändert nur ihre Werte.
- **Atomar:** in `einstellungen.toml.tmp` schreiben, dann über die alte Datei umbenennen (`std::fs::rename`).
- **Position und Skalierung** werden erst nach 1 s ohne weitere Änderung geschrieben, alles andere sofort.
- Bei einem Syntaxfehler in der Datei schreibt die App **nicht**, bis die Datei wieder lesbar ist. Änderungen aus Menü und Maus gelten bis dahin nur im Speicher.

### 8.4 Hand-Änderungen

- Im Sekundentakt prüft die App den **Änderungszeitpunkt** der Datei und liest sie bei Änderung neu (kein Datei-Watcher). Eigene Schreibvorgänge lösen kein erneutes Einlesen aus (die App merkt sich den Zeitpunkt nach dem eigenen Schreiben).
- Speichert ein Editor, gewinnt der Stand aus dem Editor.
- Was gilt wann: Dauern und Zyklus nach Abschnitt 3.5, alles andere sofort.
- Wird die Datei gelöscht, legt die App sie beim nächsten Takt neu an (Vorlage) und übernimmt die Voreinstellungen.

### 8.5 Einstellungsfenster

- Geöffnet über **Einstellungen …**. Ist es schon offen, wird es nach vorn geholt (`ViewportCommand::Focus`). Höchstens eins.
- Ein **immediate Viewport** (`show_viewport_immediate`) nur, solange es offen ist. Ein normales Fenster mit Titelleiste „Rusty Pomodoro Overlay – Einstellungen“, Taskleisten-Schaltfläche und Fokus. Feste Größe, öffnet in der Bildschirmmitte, Position wird nicht gespeichert. Standard-Aussehen von egui, folgt nicht dem Overlay-Theme.
- Ist das Overlay *ausgeblendet*, sendet die App an den Haupt-Viewport `ViewportCommand::RequestPaintWhileHidden`, solange das Fenster offen ist, damit das Fenster gezeichnet wird.
- Geschlossen mit ✕, `Esc` oder Alt+F4.
- Jede Änderung **gilt und speichert sofort**, kein *OK*/*Abbrechen*, kein eigener Zwischenstand: Das Formular liest bei jedem Zeichnen den aktuellen Stand und schickt jede Änderung sofort an die Einstellungsdatei.
- Inhalt von oben nach unten:

  ```
  ┌ ⚠ <alle Fehler der Einstellungsdatei, eine Zeile je Fehler>   (nur bei Fehlern)
  Ablauf
    Fokusphase            [ 25 ] min     (DragValue 1–180)
    Kurze Pause           [  5 ] min
    Lange Pause           [ 15 ] min
    Fokusphasen pro Zyklus [ 4 ]         (DragValue 1–12)
    [✓] Pause automatisch starten
    [ ] Fokus automatisch starten
  Hinweis
    [✓] Ton
    [✓] Signal im Overlay
    [ ] Overlay bei Phasenwechsel einblenden
  Aussehen
    Theme  [ Dunkel ▾ ]                  (dieselbe Liste wie im Menü, ohne kaputte)
  System
    [ ] Mit Windows starten
  ─────────
  [Datei öffnen]  [Theme-Ordner öffnen]
  Grafik: <GL_RENDERER> · Version <CARGO_PKG_VERSION>
  ```

- *Datei öffnen* öffnet `einstellungen.toml` mit dem Standardprogramm (`ShellExecuteW` mit `open`). Gibt es keins (Rückgabe ≤ 32), mit `notepad.exe`.
- **Grafik:** `GL_RENDERER` wird einmal beim Start über `frame.gl()` gelesen. So lässt sich prüfen, ob die iGPU läuft (Abschnitt 12).

---

## 9. Fehler sichtbar machen

- **Keine Dialoge** im Normalbetrieb, kein Logging-Crate, keine Logdatei.
- **Einstellungsdatei:** Solange ein Fehler besteht, steht ganz oben im Menü eine Zeile, ein Klick darauf öffnet die Datei:
  - `⚠ Einstellungen: Fehler in Zeile 4` (Syntaxfehler; die zuletzt gültigen Werte bleiben)
  - `⚠ Einstellungen: fokus_minuten = 500 ungültig, 25 verwendet`
  - `⚠ Einstellungen: unbekannter Schlüssel farbe`
  - bei mehreren Fehlern der erste mit dem Zusatz `(+2 weitere)`.
- **Sonstige Laufzeitfehler** erscheinen als eigene Zeile oben im Menü, ein Klick blendet sie aus:
  - `⚠ Autostart: Eintrag konnte nicht gesetzt werden`
  - `⚠ Einstellungen: Datei konnte nicht geschrieben werden`
- Theme-Fehler stehen nur im Untermenü *Theme ▸* (Abschnitt 7.4). Ein fehlendes Audiogerät ist kein Fehler (Rückgabewert von `PlaySoundW` wird ignoriert).
- Tray-Tooltip: zweite Zeile `⚠ Fehler in den Einstellungen`, solange irgendeine Fehlerzeile besteht. Das Overlay zeigt keine Fehler.
- **Absturz:** Ein Panic-Hook (gesetzt als Erstes in `main`) schreibt Meldung und Ort nach `%USERPROFILE%\.rusty_pomodoro_overlay\absturz.txt` (überschreibend) und zeigt `MessageBoxW` mit Text und Pfad. Gibt `eframe::run_native` `Err` zurück, zeigt die App dieselbe MessageBox mit dem Fehlertext.

---

## 10. Lebenszyklus

### 10.1 Start

1. Panic-Hook setzen.
2. Mutex `Local\rusty_pomodoro_overlay` anlegen. Existiert er schon: Event `Local\rusty_pomodoro_overlay_zeigen` öffnen, setzen, still beenden (Exit-Code 0).
3. Sonst Event `Local\rusty_pomodoro_overlay_zeigen` anlegen (Auto-Reset).
4. Einstellungsdatei öffnen (bei Bedarf anlegen), Theme laden.
5. `eframe::run_native` mit unsichtbarem Overlay. Im ersten `logic`-Aufruf: HWND holen, Fensterstile setzen, Tray-Icon und Menü bauen, Position setzen, Overlay ohne Fokus zeigen (wenn *Overlay anzeigen* an ist).

Fokusphase 1 ist *bereit*. Ein Autostart verhält sich wie jeder andere Start.

### 10.2 Zweiter Start

Ein `cargo run` neben einer installierten Instanz zählt ebenso als zweiter Start. Die laufende Instanz fragt das Event im Sekundentakt mit `WaitForSingleObject(event, 0)` ab. Ist es gesetzt: Overlay einblenden (ohne Fokus, *Overlay anzeigen* = an, gespeichert) und einmal aufblinken lassen (wie 6.2 Punkt 2, in der aktuellen Akzentfarbe, auch wenn *Signal im Overlay* aus ist). Kein Ton, kein *Hinweis*.

### 10.3 Autostart

- Schalter *Mit Windows starten* nur im Einstellungsfenster, Voreinstellung aus.
- Wert `RustyPomodoroOverlay` unter `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` mit dem Pfad der **laufenden Exe** in Anführungszeichen. Anschalten legt den Wert an, Ausschalten löscht ihn. Der Schalter zeigt, ob der Wert existiert.
- Die App korrigiert einen veralteten Pfad nicht selbst (z. B. wenn er aus `cargo run` heraus auf `target\…` gesetzt wurde). Das README weist darauf hin.

### 10.4 Beenden

- Nur über **Beenden** im Menü, sofort, ohne Rückfrage, auch wenn eine Phase läuft: `beenden`-Flag setzen, ausstehende Einstellungen schreiben (`sichern()`), `ViewportCommand::Close`.
- Alt+F4 auf dem Overlay wird verworfen (Abschnitt 5.1).
- **Herunterfahren und Abmelden:** `plattform` hängt sich mit `SetWindowSubclass` an das Overlay-HWND und ruft bei `WM_ENDSESSION` (`wParam` = TRUE) synchron die Sicherung der Einstellungsdatei auf. Dafür hält `einstellungsdatei` ausstehende Änderungen hinter `Arc<Mutex<…>>` und gibt einen klonbaren Griff `Sicherung` heraus, dessen `sichern()` aus der Fensterprozedur aufgerufen werden darf. Windows wird nicht aufgehalten.
- Beim normalen Beenden wird das `TrayIcon` gedroppt (Icon verschwindet) und der Mutex freigegeben. Nach einem Absturz bleibt ein totes Tray-Icon stehen, bis man mit der Maus darüberfährt. Das nehmen wir hin.

---

## 11. Architektur

### 11.1 Dateien

```
src/
├── main.rs               windows_subsystem, Panic-Hook, Mutex/Event, run_native
├── app.rs                Schale: App, logic, ausfuehren
├── ablauf.rs             rein
├── hinweis.rs            rein
├── anzeige.rs            rein
├── menue.rs              rein
├── theme.rs              rein
├── einstellungsdatei.rs  Ein- und Ausgabe
├── grafik.rs             Bild dekodieren, Tray-Icon rastern
├── tray.rs               Adapter für tray-icon/muda
├── plattform.rs          windows-sys, freie Funktionen
├── overlay.rs            egui: zeichnet Anzeige, übersetzt Eingaben
└── einstellungsfenster.rs egui: zweiter Viewport
assets/                   WAVs, Vorlage einstellungen.toml
examples/klaenge_erzeugen.rs
```

„Rein“ heißt: keine Abhängigkeit von egui, `tray-icon`/`muda`, `windows-sys` oder dem Dateisystem, keine Seiteneffekte, Zeit nur als Parameter.

### 11.2 `ablauf` (reiner Kern)

Keine Uhr als Objekt: Jeder Aufruf bekommt `jetzt: Instant`.

```rust
pub enum Phase { Fokus, KurzePause, LangePause }
pub enum Befehl { Primaeraktion, Ueberspringen, PhaseZuruecksetzen, NeuBeginnen }
pub enum Ereignis { PhaseAbgelaufen { von: Phase, nach: Phase } }

pub struct Ablaufeinstellungen {
    pub fokus: Duration, pub kurze_pause: Duration, pub lange_pause: Duration,
    pub fokusphasen_pro_zyklus: u8,
    pub pause_automatisch: bool, pub fokus_automatisch: bool,
}

pub enum Zustand { Laeuft, Angehalten, Bereit }

pub struct Stand {
    pub phase: Phase,
    pub zustand: Zustand,
    pub fokusphase: u8,          // 1-basiert: aktuelle bzw. zuletzt beendete Fokusphase
    pub erledigt: u8,            // erledigte Fokusphasen im Zyklus
    pub fokusphasen_im_zyklus: u8,
    pub rest: Duration,          // bei Bereit: volle Dauer
    pub ueberzeit: Option<Duration>,
    pub anteil: f32,             // 0.0..=1.0
}

impl Ablauf {
    pub fn neu(e: Ablaufeinstellungen) -> Self;
    pub fn befehl(&mut self, b: Befehl, jetzt: Instant) -> Vec<Ereignis>;
    pub fn takt(&mut self, jetzt: Instant) -> Vec<Ereignis>;   // beliebig oft aufrufbar, höchstens ein Ereignis
    pub fn stand(&self, jetzt: Instant) -> Stand;
    pub fn naechste_faelligkeit(&self, jetzt: Instant) -> Option<Instant>;
    pub fn einstellungen(&mut self, e: Ablaufeinstellungen);   // setzt die Regel aus 3.5 um
}
```

Die genaue Form der Typen darf die Umsetzung anpassen, solange die Schnittstelle schmal bleibt und Zeit nur als Parameter hereinkommt.

### 11.3 `hinweis`, `anzeige`, `theme`

- `hinweis`: `ausloesen(neue_phase, schalter, stoeren_erlaubt, jetzt) -> Hinweisaktionen { klang: Option<Klang>, einblenden: bool }` merkt sich den Beginn des Aufblinkens. `blinken_starten(jetzt)` für den zweiten Start. `aufblinken(jetzt) -> f32` (0 außerhalb) und `naechste_faelligkeit(jetzt)` (etwa alle 16 ms, solange es blinkt). `stoeren_erlaubt` fragt die Schale nur beim Ablauf einer Phase über `plattform` ab.
- `anzeige(stand, theme, aufblinken) -> Anzeige`: ein fertiger, rein darstellender Datensatz: Zeittext (`24:13`/`+01:05`), Symbol (keins/Pause/Start), Phasenwort, Farben (schon gedimmt bzw. aufgeblinkt), Hintergrund, Fortschritt `0.0..=1.0`, Zyklus-Punkte (Liste aus voll/hervorgehoben/leer), Tooltip-Text und Tray-Icon-Schlüssel (Phase, gedimmt). Das Overlay platziert nur noch.
- `theme`: `Theme { name, farben, schrift, bild, tray_zeichen }`, `lesen(&str, ordner) -> Result<(Theme, Vec<Warnung>), Fehler>` (Fehlendes von *Dunkel*), `eingebaute()`, `ordner_lesen(pfad) -> Vec<Themeeintrag>` fürs Menü.

### 11.4 `menue`

- `Aktion` gilt für **alle** Quellen: `Primaeraktion`, `Ueberspringen`, `PhaseZuruecksetzen`, `NeuBeginnen`, `Umschalten(Schalter)` (ImmerImVordergrund, ClickThrough, OverlayAnzeigen, Ton), `ThemeWaehlen(Schluessel)`, `ThemeOrdnerOeffnen`, `EinstellungsfensterOeffnen`, `EinstellungsdateiOeffnen`, `FehlerAusblenden(id)`, `Beenden`.
- `menue(stand, einstellungen, fehler, themes) -> Menuemodell` (Text, aktiv, angehakt je Eintrag, Fehlerzeilen oben, Theme-Untermenü) ist rein. `tray.rs` überträgt es nach jedem Event auf die `muda`-Einträge und ordnet die `MenuId`s den `Aktion`en zu.

### 11.5 `einstellungsdatei`

Tiefes Modul, die einzige Quelle der Einstellungen (auch Position und Skalierung):
- `oeffnen(ordner) -> Self` (legt Datei bei Bedarf an)
- `aktuell() -> (&Einstellungen, &[Fehler])`
- `aendern(Aenderung, jetzt)` (sofort oder verzögert schreiben, 8.3)
- `takt(jetzt) -> Option<&Einstellungen>` (Hand-Änderung erkannt → neue Werte; schreibt Fälliges)
- `sichern()` und `sicherung() -> Sicherung` (10.4)
- intern die reine Naht `lesen(&str) -> (Einstellungen, Vec<Fehler>)`.

Kein Dateisystem-Trait. Datei-Tests laufen gegen ein temporäres Verzeichnis (`std::env::temp_dir()` plus eindeutiger Unterordner).

### 11.6 Schale und Ereignisfluss

- Eine Verteilstelle `App::ausfuehren(aktion, jetzt)`. Linksklick und Leertaste ergeben `Primaeraktion`. Ziehen und `Strg`+Mausrad gehen als `Aenderung` direkt in die Einstellungsdatei.
- `tray-icon`/`muda` liefern Events über `MenuEvent::set_event_handler` / `TrayIconEvent::set_event_handler` in einen Kanal, gefolgt von `ctx.request_repaint()`. Kein eigener Thread.
- **`App::logic` läuft jede Sekunde, auch bei ausgeblendetem Overlay**, und muss beliebig oft aufrufbar sein. Feste Reihenfolge:
  1. Eingaben (Kanal, Overlay-Eingaben aus dem letzten `ui`) → `Aktion` → `ausfuehren`
  2. `einstellungsdatei.takt(jetzt)`; bei neuen Werten `ablauf.einstellungen(…)`, Theme, Fensterzustand anwenden
  3. `ablauf.takt(jetzt)`; für jedes `PhaseAbgelaufen`: `hinweis.ausloesen(…)`, Klang abspielen, ggf. einblenden
  4. fällige Prüfungen: Event zweiter Start, Position im Arbeitsbereich, Bild fertig dekodiert
  5. `anzeige(…)` berechnen; Tooltip, Tray-Icon (bei Änderung) und `Menuemodell` anwenden
  6. `ctx.request_repaint_after(min(alle Fälligkeiten, nächste volle Sekunde der Restzeit, 1 s))`
- `ui` zeichnet nur die zuletzt berechnete `Anzeige` (`overlay.rs`) und gegebenenfalls das Einstellungsfenster. Kein Timer-Thread, kein `SetTimer`.

### 11.7 `plattform`, `grafik`, `tray`

Freie Funktionen ohne Trait: Mutex/Event, Autostart (Registry), `stoeren_erlaubt()`, `klang_abspielen(&'static [u8])`, Fensterstile, `zeigen_ohne_fokus`, `fenster_rechteck`/`fenster_setzen`, Arbeitsbereich, Subclass für `WM_ENDSESSION`, `oeffnen(pfad)`, `MessageBoxW`. `grafik`: `Hintergrund` (`setzen(ctx, bild, flaeche, neu_laden)`, `abholen(ctx)`, `textur()`, `fehlgeschlagen()`) und `Trayzeichner::icon(tray_zeichen, farbe, groesse) -> RGBA` mit Zwischenspeicher der Maske. Die Farbe kommt fertig gedimmt aus `Anzeige.akzent`; das `Icon` baut die Schale. `tray`: `Tray` überträgt das `Menuemodell` (neu aufgebaut nur bei geänderter Aktionsfolge, sonst angeglichen), setzt Tooltip und Icon und liefert die angeklickten `Aktion`en.

---

## 12. Doku (README.md)

Gehört zu v1. Das README erklärt:
- Bauen und Installieren (`cargo install --path .`), Starten, Beenden über das Menü.
- Ort der Einstellungsdatei und des Theme-Ordners, das Beispiel-Theme aus 7.3.
- **iGPU bevorzugen:** Auf Laptops mit zwei Grafikkarten braucht die App auf der iGPU deutlich weniger RAM (gemessen ~94 MB statt ~168 MB). Weg: Einstellungen → System → Anzeige → Grafik → `%USERPROFILE%\.cargo\bin\rusty-pomodoro-overlay.exe` hinzufügen → *Energiesparmodus*. Prüfen über die Zeile *Grafik* im Einstellungsfenster. Die App greift dafür nicht selbst ein. Der Eintrag hängt am Exe-Pfad, `cargo run` braucht einen eigenen.
- **Autostart** zeigt auf die Exe, aus der er eingeschaltet wurde. Also in der installierten Version einschalten, nicht aus `cargo run` heraus.
- Grenzen: *Bitte nicht stören* von Windows 11 wird nicht erkannt, nur Vollbild und Präsentation.

---

## 13. Tests

### 13.1 Automatisch (`cargo test`)

`#[cfg(test)]` im jeweiligen Modul, nur über dessen Schnittstelle. Zeit als `t0 + Duration`, Standby als Sprung. Keine GUI-Tests, kein Mocking von egui, kein Dateisystem-Fake.

`ablauf` (am ausführlichsten), mindestens:
- Start: Fokusphase 1 *bereit*, volle Dauer, keine Überzeit.
- Starten, Anhalten, Fortsetzen: Restzeit stimmt nach beliebigen Takten und Pausen.
- Ablauf Fokus→Pause mit Auto-Start an: kurze Pause *läuft*, ein `PhaseAbgelaufen`.
- Ablauf Pause→Fokus mit Auto-Start aus: Fokusphase *bereit*, Überzeit wächst ab dem Ablaufzeitpunkt.
- Vierte Fokusphase → lange Pause → neuer Zyklus; *n* = 1 und *n* = 12.
- Überspringen aus *läuft*, *angehalten*, *bereit*: zählt als erledigt, kein Ereignis, keine Überzeit.
- *Phase zurücksetzen* behält den Zyklus-Fortschritt und übernimmt die neue Dauer; *Neu beginnen* setzt alles zurück.
- Geänderte Dauer: laufende Phase unverändert, *bereit* sofort, nächste Phase dieses Typs neu. Geänderte Fokusphasen pro Zyklus: erst ab nächstem Zyklus, sofort am Zyklusanfang.
- Standby-Sprung über mehrere Phasen mit Auto-Start: genau ein Ereignis, Stand wie ohne Schlaf, Überzeit ab echtem Ablaufzeitpunkt.
- `takt` hundertmal mit demselben `jetzt` ändert nichts.
- `naechste_faelligkeit` fällt auf den Ablaufzeitpunkt bzw. die nächste volle Sekunde.

Weitere:
- `hinweis`: Klang je Richtung, kein Klang bei *Ton* aus oder `stoeren_erlaubt` = false, Aufblinken genau 2 s mit drei Pulsen, Einblenden nur mit Schalter.
- `anzeige`: Auf- und Abrunden (`24:59,1` → `25:00`, `00:00` genau am Ablauf, `+00:00` zu Beginn), mehr als zwei Minutenstellen, gedimmte Farben, Zyklus-Punkte aus 5.4, Tooltip-Texte aus 4.5.
- `menue`: Ausgrau-Regeln, Primäraktion-Text, Häkchen, Fehlerzeile mit `(+n weitere)`, Theme-Untermenü (Reihenfolge, reserviert, Fehlerzusätze).
- `theme::lesen`: Erben von *Dunkel*, ungültige und unbekannte Felder (auch `schrift = "comic_sans"` → `ubuntu`), Farbformate, Syntaxfehler mit Zeile.
- `einstellungsdatei`: `lesen` mit jedem Fehlerfall aus 8.2; im temporären Verzeichnis: Anlegen der Vorlage, Kommentare bleiben beim Schreiben erhalten, atomares Schreiben, verzögertes Schreiben nach 1 s, kein Schreiben bei Syntaxfehler, Hand-Änderung wird erkannt, eigene Schreibvorgänge nicht.
- `grafik`: Verkleinern auf die Zielfläche (nie vergrößern), Rückfall auf den Kreis bei U+F0000, leerem Feld und ZWJ-Folge (mit den Systemschriften), einige Pixel von Kreis und Zeichen.
- `schriften`: Ziffern und `+` gleich breit in Inter und JetBrains Mono.

### 13.2 Manuelle Prüfliste

Vor dem Abschluss von v1 einmal auf echter Hardware (Windows 11, möglichst Hybrid-GPU) abhaken:

**Overlay**
- [ ] Erster Start: Overlay unten rechts im Arbeitsbereich, Fokusphase 1 *bereit*, gedimmt `25:00` mit Startsymbol.
- [ ] Hintergrund durchsichtig, Ziffern deckend, auf iGPU und dGPU.
- [ ] Kein Taskleisten-Eintrag, nicht in Alt+Tab, nimmt beim Start keinen Fokus (Tippen in einem Editor geht weiter).
- [ ] Linksklick startet, hält an, setzt fort. Leertaste ebenso (nach einem Klick ins Overlay).
- [ ] Ziehen verschiebt ohne Primäraktion. Halb über den Rand gezogen, rutscht es zurück.
- [ ] `Strg`+Mausrad: 10 %-Schritte, 50 % und 300 % als Grenze, um die Mitte.
- [ ] Position und Größe nach Neustart gleich. Monitor abstecken: Overlay kommt zurück.
- [ ] Zeit zappelt nicht (gleich breite Ziffern), `180:00` passt bei 50 %.
- [ ] Alt+F4 aufs Overlay bewirkt nichts.
- [ ] Immer-im-Vordergrund an/aus wirkt sofort.
- [ ] Click-through an: Klicks gehen durch, Overlay bleibt sichtbar. Aus nur übers Tray. Nach Neustart aus.

**Tray und Menü**
- [ ] Links- und Rechtsklick aufs Tray-Icon und Rechtsklick aufs Overlay zeigen dasselbe Menü.
- [ ] `Umschalt+F10` und Menütaste öffnen das Menü an der oberen linken Ecke, `Esc` schließt.
- [ ] Ausgrauen von *Phase zurücksetzen* und *Neu beginnen* stimmt.
- [ ] Tooltip aktualisiert sich sekündlich, auch bei ausgeblendetem Overlay.
- [ ] Bei offenem Menü friert die Zahl ein, danach springt sie auf den richtigen Wert.
- [ ] Explorer neu starten: Tray-Icon kommt wieder.
- [ ] Tray-Icon wechselt Farbe je Phase, gedimmt bei *angehalten*/*bereit*.

**Hinweis**
- [ ] Ablauf einer Fokusphase (Dauer 1 min): Klang *Fokus endet*, dreimal Aufblinken in Pausenfarbe, Pause läuft.
- [ ] Ablauf einer Pause: Klang *Pause endet*, Fokusphase *bereit*, `+00:00` zählt hoch.
- [ ] *Ton* im Menü aus: kein Klang. App im Lautstärkemixer sichtbar.
- [ ] Vollbild-Video bzw. Spiel im Vordergrund: kein Klang, Aufblinken bleibt.
- [ ] Ausgeblendet mit *Overlay bei Phasenwechsel einblenden* an: Overlay erscheint ohne Fokus und blinkt.
- [ ] Standby über einen Ablauf hinweg: nach dem Aufwachen genau ein Hinweis, Stand stimmt.

**Themes**
- [ ] *Dunkel*/*Hell* wechseln sofort, Wahl bleibt nach Neustart.
- [ ] Theme mit `schrift = "inter"` und `"jetbrains_mono"`: Zeit zappelt nicht, auch bei 50 % gut lesbar.
- [ ] Eigenes Theme mit Bild (großes Handyfoto, hochkant mit EXIF-Drehung): richtig gedreht, mittig beschnitten, Deckkraft wirkt, kein Absturz.
- [ ] Theme mit Syntaxfehler ausgegraut mit Zeilennummer, Bild gelöscht → `(Bild fehlt)`.
- [ ] Tray-Zeichen `☢` und `🍅` einfarbig in der Akzentfarbe, `👍🏽` ergibt den Kreis.
- [ ] *Theme-Ordner öffnen* legt den Ordner an und öffnet ihn.

**Einstellungen**
- [ ] Erster Start legt die kommentierte Datei an.
- [ ] Änderung im Fenster wirkt und steht sofort in der Datei, Kommentare bleiben.
- [ ] Hand-Änderung im Editor wirkt innerhalb von etwa 1 s.
- [ ] `fokus_minuten = 500`: Fehlerzeile im Menü, Tooltip-Zeile, Balken im Fenster, 25 verwendet.
- [ ] Syntaxfehler: alte Werte bleiben, App schreibt nicht (Overlay verschieben ändert die Datei nicht).
- [ ] Fenster öffnet auch bei ausgeblendetem Overlay, ein zweites *Einstellungen …* holt es nach vorn.
- [ ] Zeile *Grafik* zeigt den Renderer.

**Plattform**
- [ ] Zweiter Start: keine zweite Instanz, laufendes Overlay erscheint und blinkt.
- [ ] *Mit Windows starten*: Eintrag im Task-Manager unter *Autostart-Apps*, nach Abmelden/Anmelden läuft die App.
- [ ] Overlay verschieben und sofort abmelden: Position ist gespeichert.
- [ ] Erzwungener Panic (Testbuild): `absturz.txt` und MessageBox.
- [ ] Release-Build unter 6,5 MB, `cargo tree -d` ohne doppelte `skrifa`/`vello_cpu`.
