---
title: Sekundentakt bei ausgeblendetem Overlay
labels: [wayfinder:prototype]
status: closed
assignee: ckr
blocked_by: []
---

# Sekundentakt bei ausgeblendetem Overlay

## Question

Läuft die App-Logik (`App::logic` von eframe 0.37 + glow) zuverlässig **jede Sekunde**, auch wenn das Overlay **ausgeblendet** ist (`ViewportCommand::Visible(false)`), das Einstellungsfenster zu ist und keine Eingaben kommen? Und reicht das für alles, was am Sekundentakt hängt? Am Takt hängen der Phasenablauf und der Hinweis, der Tray-Tooltip und das Tray-Icon, der Änderungszeitpunkt der Einstellungsdatei, das Event für den zweiten Start und das verzögerte Speichern von Position und Skalierung. Falls nicht: Welcher Taktgeber stattdessen (z. B. `request_repaint_after`, eigener Thread mit `EventLoopProxy` bzw. `ctx.request_repaint()`, `SetTimer` am Fenster)? Ergebnis: Ein Wegwerf-Prototyp auf Branch `prototype/14-sekundentakt` belegt das gewählte Vorgehen, mit gemessenen Abständen zwischen den Takten (ausgeblendet, sichtbar, offenes Menü, nach Standby).

## Kontext

- Aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: Bei offenem nativem Menü steht die Event-Loop. Die Restzeit wird darum aus `Instant`-Differenzen bzw. der Wanduhr berechnet, nicht gezählt. Tray-Code lebt in `App::logic`.
- Aus *[Phasenablauf](03-phasenablauf.md)*: Die Zeitbasis ist die Wanduhr, Standby verbraucht Zeit und wird beim Aufwachen nachgeholt.
- Aus *[Einstellungen und Speichern](08-einstellungen.md)* und *[Lebenszyklus der App](09-lebenszyklus.md)*: Änderungszeitpunkt der Datei und benanntes Event werden im Sekundentakt abgefragt.
- Spike-Code als Ausgangspunkt: Branch `prototype/02-overlay-spike`.

## Resolution

**Ja: `App::logic` läuft auch bei ausgeblendetem Overlay zuverlässig jede Sekunde, wenn `logic` selbst `ctx.request_repaint_after(<bis zur nächsten vollen Sekunde>)` anfordert. Ein eigener Taktgeber ist nicht nötig, alles am Sekundentakt kommt in `logic`.**

Auf der echten Hardware gemessen (Windows 11, Release, mit `--auto`-Skript und zwei Handtests):

- **Versteckt** (`ViewportCommand::Visible(false)`): Median 999 ms, max 1001–1008 ms, keine Lücke in den Skriptläufen (einmal 1950 ms im Handtest). eframe 0.37 ruft bei unsichtbarem Fenster `logic` direkt über `run_logic` auf, ohne egui-Durchlauf, und drosselt auf höchstens alle 100 ms.
- **Sichtbar:** keine echte Lücke. Zusätzlich kommen viele Aufrufe unter einer Sekunde (Median 16–72 ms, egui legt Folge-Frames ein, Eingaben). **`logic` muss also beliebig oft aufrufbar sein:** Alles wird aus der Uhr berechnet, nichts zählt Aufrufe. Fällige Dinge (Datei-Änderungszeitpunkt, Event für den zweiten Start, verzögertes Speichern) prüft `logic` gegen eigene Fälligkeitszeitpunkte.
- **Menü offen:** Die Event-Loop steht wie erwartet (Lücken von 12,5 s und 7,1 s). Nach dem Schließen holt der nächste Aufruf alles nach. Bestätigt die bestehende Regel aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*.
- **Ziehen** (`StartDrag`): Die Uhr läuft weiter, keine Lücke.
- **Wiedereinblenden ohne Eingabe:** `Visible(true)` aus `logic` heraus blendet das Overlay nach exakt der gewünschten Zeit ein. Das trägt *Overlay bei Phasenwechsel einblenden* und das Einblenden beim zweiten Start.
- **Standby** (~130 s): Der erste Aufruf nach dem Aufwachen sieht die volle Lücke, danach läuft der Takt normal weiter. **`Instant` zählt den Standby mit** (Monotonuhr und Wanduhr gleich: 129 755 ms). Restzeit und Fälligkeiten dürfen also auf `Instant` beruhen, das Nachholen aus *[Phasenablauf](03-phasenablauf.md)* funktioniert damit von selbst.
- **Tray-Tooltip** wird in `logic` jede Sekunde gesetzt, auch versteckt.
- **CPU** im sichtbaren Leerlauf 0,63 % eines Kerns, 117 MB Working Set.

Verworfen: **eigener Thread mit `ctx.request_repaint()`** (sichtbar Aussetzer bis 2,4 s, vermutlich weil eframe Anfragen aus fremden Threads nach dazwischenliegenden Frames als veraltet verwirft; versteckt doppelt so viele Aufrufe wie nötig), `SetTimer` am Fenster (nicht nötig, nicht gebaut). `ViewportCommand::RequestPaintWhileHidden` ist nicht nötig, weil ohne `ui` nichts gezeichnet werden muss.

Prototyp (Wegwerf-Code, nicht auf `main`): Branch `prototype/14-sekundentakt`, Ordner `docs/wayfinder/pomodoro-overlay-spec/prototypes/14-sekundentakt/`, Messwerte in `MESSUNG.md` plus CSV. Start: `git switch prototype/14-sekundentakt`, im Ordner `cargo run --release` (oder mit `-- --auto` für den Skriptlauf).
