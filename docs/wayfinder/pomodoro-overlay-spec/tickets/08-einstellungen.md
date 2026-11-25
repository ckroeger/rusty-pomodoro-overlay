---
title: Einstellungen und Speichern
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: [04]
---

# Einstellungen und Speichern

## Question

Wo und wie werden Einstellungen (Dauern, Theme, Immer-im-Vordergrund, Position, …) gespeichert (z. B. TOML unter `%APPDATA%`), werden sie nur per Datei bearbeitet oder auch über eine Oberfläche, und werden Änderungen zur Laufzeit übernommen?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Zu den Einstellungen gehören die drei Dauern (ganze Minuten, 1 bis 180), *Fokusphasen pro Zyklus* (1 bis 12) und zwei Schalter für den automatischen Start (Fokus→Pause, Pause→Fokus). Bei ungültigen Werten gilt die Voreinstellung mit Hinweis; **wie** der Hinweis erscheint, wird hier entschieden. Geänderte Werte gelten erst ab der nächsten Phase dieses Typs bzw. ab dem nächsten Zyklus.

> Kontext aus *[Bedienung](04-bedienung.md)*: Das gemeinsame Menü enthält **Einstellungen …**. Was dieser Eintrag öffnet (Datei im Editor, eigenes Fenster), wird hier entschieden. Die Menü-Schalter *Immer im Vordergrund*, *Click-through* und *Overlay anzeigen* sind Kandidaten für gespeicherte Einstellungen.

> Kontext aus *[Inhalt und Verhalten des Overlays](05-overlay-anzeige.md)*: Zu speichern sind außerdem die **Position** des Overlays (virtuelle Bildschirmkoordinaten, nach jedem Verschieben) und die **Skalierung** (50 % bis 300 %, nach jeder Änderung per `Strg`+Mausrad). Einen Regler für die Größe gibt es nicht, sie wird nur über das Mausrad geändert.

> Kontext aus *[Hinweis bei Phasenwechsel](06-phasenwechsel-hinweis.md)*: Drei weitere Einstellungen: *Ton* (an), *Signal im Overlay* (an), *Overlay bei Phasenwechsel einblenden* (aus). *Ton* gibt es zusätzlich als Häkchen im gemeinsamen Menü, ändert man es dort, muss es gespeichert werden. Eine Lautstärke wird nicht gespeichert.

> Kontext aus *[Theming](07-theming.md)*: Zu speichern ist das **aktive Theme**. Ist es beim Start weg oder kaputt, gilt *Dunkel*, die gespeicherte Wahl wird aber nicht überschrieben. Zu entscheiden sind außerdem der Ort des **Theme-Ordners** („neben den Einstellungen“) und das **Dateiformat eigener Themes**, am besten dasselbe wie für die Einstellungen. Eine Datei pro Theme, Felder `name`, `bild` (Pfad relativ zur Theme-Datei), `tray_zeichen`, sechs Farben (`hintergrund` mit Deckkraft). Fehlende Felder erben von *Dunkel*. Ein Syntaxfehler soll mit Zeilennummer erkennbar sein, z. B. `Fehler in Zeile 4`.

## Resolution

**Eine TOML-Datei unter `%USERPROFILE%\.rusty_pomodoro_overlay\`, die die App behutsam mitschreibt, plus ein Einstellungsfenster für die Werte, die sonst nirgends einstellbar sind. Änderungen gelten ohne Neustart.**

1. **Ort und Format:**

   ```
   %USERPROFILE%\.rusty_pomodoro_overlay\
   ├── einstellungen.toml
   └── themes\
       ├── pipboy.toml
       └── pipboy.jpg
   ```

   - Einstellungen und eigene Themes sind beide **TOML**. Das Crate meldet Syntaxfehler mit Zeilennummer.
   - Die Schlüssel sind deutsch und folgen dem Glossar, z. B. `fokus_minuten = 25`, `ton = true`. Theme-Felder wie in *[Theming](07-theming.md)*.
   - Farben werden als `"#RRGGBB"` oder `"#RRGGBBAA"` geschrieben, die Deckkraft von `hintergrund` steht im Alphakanal (`"#1E1E1ED9"` = 85 %).
   - Bewusst nicht `%APPDATA%`: Der Dot-Ordner im Profil ist üblich für Entwicklerwerkzeuge (`.cargo`, `.rustup`) und passt zu `cargo install`. Er ist unter Windows nicht versteckt. Einen portablen Modus neben der Exe gibt es nicht.
2. **Was wo steht:**

   | Wert | Fenster | Datei |
   |---|---|---|
   | Dauern Fokus / kurze / lange Pause (1–180 min), Fokusphasen pro Zyklus (1–12) | ✔ | ✔ |
   | automatischer Start Fokus→Pause, Pause→Fokus | ✔ | ✔ |
   | *Ton*, *Signal im Overlay*, *Overlay bei Phasenwechsel einblenden* | ✔ | ✔ |
   | aktives Theme (Auswahlliste) | ✔ | ✔ |
   | *Immer im Vordergrund*, *Overlay anzeigen* (Menü) | – | ✔ |
   | Position, Skalierung (Maus) | – | ✔ |

   Was man im Menü oder mit der Maus direkt einstellt, steht nicht zusätzlich im Fenster. Unten im Fenster: **Datei öffnen** und **Theme-Ordner öffnen**.
3. **Gespeichert** werden alle Werte der Tabelle, also auch *Immer im Vordergrund*, *Overlay anzeigen*, *Ton* und das aktive Theme. **Nicht gespeichert** wird *Click-through*, es ist beim Start immer aus, sonst wirkt das Overlay nach einem Neustart kaputt. Einen Menüeintrag zum Zurücksetzen gibt es nicht: Wer neu anfangen will, löscht die Datei.
4. **Schreiben:**
   - Die App schreibt mit **`toml_edit`**. Kommentare, Reihenfolge und Formatierung bleiben erhalten, nur geänderte Werte werden ersetzt.
   - Beim **ersten Start** legt die App den Ordner und eine **vollständig kommentierte** Datei mit allen Schlüsseln und Voreinstellungen an.
   - Gespeichert wird **atomar** (temporäre Datei, dann umbenennen). Vor jedem Schreiben liest die App die Datei frisch ein und ändert nur ihren Wert.
   - **Position und Skalierung** werden erst nach etwa 1 s Ruhe gespeichert, nicht bei jedem Pixel.
   - Es gibt eine Datei, keine zweite für Laufzeitwerte.
5. **Übernehmen von Hand-Änderungen:**
   - Die App prüft im Sekundentakt den **Änderungszeitpunkt** der Datei und liest sie bei einer Änderung neu, ohne Datei-Watcher-Crate und ohne Neustart.
   - Dauern gelten ab der nächsten Phase ihres Typs, *Fokusphasen pro Zyklus* ab dem nächsten Zyklus (wie in *Phasenablauf*). Alles andere gilt sofort.
   - Speichert man im Editor, gewinnt der Stand aus dem Editor. Überschreibt ein Editor die Datei mit einem alten Stand, geht höchstens ein App-Wert wie die Position verloren. Das ist harmlos.
6. **Fehler:** Es gibt keine Dialoge.
   - Bei einem **Syntaxfehler** behält die App die zuletzt gültigen Werte und **schreibt nicht**, bis die Datei wieder lesbar ist.
   - Ein **ungültiger Wert** wird durch die Voreinstellung ersetzt. Ein **unbekannter Schlüssel** wird ignoriert. Beides wird gemeldet.
   - Solange ein Fehler besteht, steht **ganz oben im gemeinsamen Menü** ein Eintrag wie `⚠ Einstellungen: Fehler in Zeile 4` oder `⚠ Einstellungen: fokus_minuten = 500 ungültig, 25 verwendet`, bei mehreren mit dem Zusatz `(+2 weitere)`. Ein Klick öffnet die Datei.
   - Der **Tray-Tooltip** bekommt eine zweite Zeile `⚠ Fehler in den Einstellungen`. Im **Einstellungsfenster** zeigt ein Balken oben alle Fehler. Das Overlay zeigt keine Fehler.
7. **Einstellungsfenster:**
   - Es ist ein **normales Windows-Fenster** (zweiter Viewport von eframe) mit Titelleiste und Taskleisten-Schaltfläche, solange es offen ist, und es darf beim Öffnen den Fokus nehmen.
   - Geöffnet wird es über **Einstellungen …** im Menü. Ist es schon offen, wird es nach vorn geholt, es gibt höchstens eins.
   - Jede Änderung **gilt und speichert sofort**, es gibt kein *OK*/*Abbrechen*.
   - Die Eingaben sind auf gültige Werte begrenzt: Zahlenfelder mit festem Bereich, Häkchen, eine Auswahlliste für das Theme.
   - Es hat das Standard-Aussehen von egui und folgt nicht dem Overlay-Theme.
   - Geschlossen wird mit X oder Esc. Die Größe ist fest, es öffnet in der Bildschirmmitte, seine Position wird nicht gespeichert.

Ob der **Timer-Stand** einen Neustart übersteht, entscheidet *Lebenszyklus der App*.

Verworfen: nur Datei oder nur Fenster, `%APPDATA%`/`%LOCALAPPDATA%`, ein portabler Modus, JSON/RON, eine getrennte Datei für Laufzeitwerte, ein Datei-Watcher, *OK*/*Abbrechen*, ein gespeichertes *Click-through*, ein Menüeintrag zum Zurücksetzen.
