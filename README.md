# Rusty Pomodoro Overlay

Ein persönlicher Pomodoro-Timer für **Windows 11**. Ein kleines, randloses Overlay zeigt die Restzeit der aktuellen Phase. Das Tray-Icon bietet dasselbe Menü wie ein Rechtsklick aufs Overlay.

## Bauen und installieren

Voraussetzung ist Rust mit dem Ziel `x86_64-pc-windows-msvc`.

```
cargo install --path .
```

Das legt `%USERPROFILE%\.cargo\bin\rusty-pomodoro-overlay.exe` an. Gestartet wird die Exe direkt oder über *Mit Windows starten*. Ein zweiter Start öffnet keine zweite Instanz, er holt das laufende Overlay hervor.

**Beenden** geht nur über *Beenden* im Menü (Rechtsklick aufs Overlay oder Klick aufs Tray-Icon). Alt+F4 auf dem Overlay bewirkt absichtlich nichts.

## Bedienung

- **Linksklick** aufs Overlay: Starten, Anhalten, Fortsetzen. Mit dem Fokus im Overlay geht das auch mit der **Leertaste**.
- **Ziehen** verschiebt das Overlay, **`Strg`+Mausrad** ändert die Größe (50 % bis 300 %).
- **Rechtsklick**, `Umschalt+F10` oder die Menütaste öffnen das Menü. Das Tray-Icon öffnet dasselbe Menü mit Links- und Rechtsklick.
- Ist *Click-through* an, gehen Klicks durchs Overlay. Ausschalten geht dann nur noch über das Tray-Icon.

## Einstellungen

Alle Einstellungen stehen in

```
%USERPROFILE%\.rusty_pomodoro_overlay\einstellungen.toml
```

Beim ersten Start legt die App die Datei mit Kommentaren an. Änderungen im Editor wirken innerhalb von etwa einer Sekunde. Ungültige Werte ersetzt die App durch die Voreinstellung und meldet das oben im Menü. Wer neu anfangen will, löscht die Datei. Das Fenster *Einstellungen …* im Menü ändert dieselbe Datei.

## Eigene Themes

Eigene Themes sind TOML-Dateien im Theme-Ordner:

```
%USERPROFILE%\.rusty_pomodoro_overlay\themes\
```

*Theme-Ordner öffnen* im Menü legt ihn an und öffnet ihn. Fehlende Felder übernimmt ein Theme von *Dunkel*. Beispiel `pipboy.toml`:

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

- Farben als `"#RRGGBB"` oder `"#RRGGBBAA"`. Der Alphakanal von `hintergrund` ist die Deckkraft des Overlays und gilt auch für ein Bild.
- `schrift`: `"ubuntu"` (Voreinstellung), `"inter"` oder `"jetbrains_mono"`.
- `bild`: JPG oder PNG, relativ zur Theme-Datei. Es füllt das Overlay und wird mittig beschnitten.
- `tray_zeichen`: genau ein Unicode-Zeichen, einfarbig in der Farbe der Phase. Lässt es sich nicht darstellen (z. B. 👍🏽 oder 👨‍💻), zeigt das Tray-Icon einen Kreis.

## iGPU bevorzugen

Auf Laptops mit zwei Grafikkarten braucht die App auf der integrierten Grafik deutlich weniger Arbeitsspeicher (gemessen etwa 94 MB statt 168 MB). So stellt man das ein:

1. Einstellungen → System → Anzeige → Grafik
2. `%USERPROFILE%\.cargo\bin\rusty-pomodoro-overlay.exe` hinzufügen
3. *Energiesparmodus* wählen

Ob es wirkt, zeigt die Zeile *Grafik* im Einstellungsfenster. Die App greift dafür nicht selbst ein. Der Eintrag hängt am Pfad der Exe, für `cargo run` braucht es einen eigenen.

## Autostart

*Mit Windows starten* im Einstellungsfenster trägt den Pfad der **gerade laufenden** Exe ein. Also in der installierten Version einschalten, nicht aus `cargo run` heraus. Sonst zeigt der Eintrag auf `target\…`.

## Grenzen

- Der Modus *Bitte nicht stören* von Windows 11 wird nicht erkannt. Nur bei Vollbild-Anwendungen und Präsentationen spielt die App keinen Ton.
- Der Timer-Stand wird nicht gespeichert. Nach einem Neustart beginnt Fokusphase 1.

## Lizenzen

Die Schriften *Inter* und *JetBrains Mono* stehen unter der SIL Open Font License 1.1 (`assets/OFL-Inter.txt`, `assets/OFL-JetBrainsMono.txt`). In *Inter* sind die gleich breiten Ziffern (`tnum`) fest eingebaut.
