---
title: Architektur der App
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: [14]
---

# Architektur der App

## Question

In welche Module zerfällt die App, und wo verlaufen die Nähte? Kandidaten sind Timer-Logik (Phasenablauf, Zustände, Zyklus), Anzeige (Overlay-Zeichnen aus einem Datensatz), Tray und Menü, Einstellungen (Laden, Prüfen, Schreiben), Themes, Hinweis (Ton, Aufblinken) und Plattform (Mutex, Event, Autostart, `SHQueryUserNotificationState`). Welche Schnittstellen haben sie, was wird wie getestet (injizierbare Uhr für die Timer-Logik, Tests für das Prüfen der Einstellungen und Themes), und wie fließen Ereignisse (Menü-Event, Klick, Takt, Datei geändert) zum Zustand und zurück zu Anzeige und Menü?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Die Timer-Logik bekommt eine injizierbare Uhr.
- Aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: Die Restzeit wird aus `Instant`-Differenzen berechnet (die Event-Loop steht bei offenem Menü). Tray-Code lebt in `App::logic`. Der Menüzustand wird nach jedem Event aus dem App-Zustand neu gesetzt.
- Aus *[Inhalt und Verhalten des Overlays](05-overlay-anzeige.md)*: Das Zeichnen bekommt die Anzeigedaten als einen Datensatz, getrennt von der Timer-Logik, damit spätere frei gestaltbare Layouts möglich bleiben.
- Aus *[Einstellungen und Speichern](08-einstellungen.md)*: Das Einstellungsfenster ist ein zweiter Viewport derselben App. Die Konfiguration wird im Sekundentakt am Änderungszeitpunkt neu geladen, geschrieben wird verzögert, atomar und mit `toml_edit`.
- Aus *[Lebenszyklus der App](09-lebenszyklus.md)*: Es wird kein Timer-Stand gespeichert. Es gibt eine Instanz (Mutex und Event), Autostart über `HKCU\…\Run`.
- Aus *[Sekundentakt bei ausgeblendetem Overlay](14-sekundentakt-im-hintergrund.md)*: Taktgeber ist `request_repaint_after` aus `App::logic` heraus, das auch versteckt jede Sekunde läuft. `logic` wird sichtbar aber auch viel öfter aufgerufen und muss deshalb beliebig oft aufrufbar sein (Fälligkeiten gegen die Uhr prüfen, nichts zählen). `Instant` zählt den Standby mit.
- Skill `codebase-design` für das Vokabular (tiefe Module, Nähte) hinzuziehen.

## Resolution

Alles Folgende sind Implementierungsentscheidungen, es gibt keine neuen Fachbegriffe und kein ADR (der Schnitt ist leicht änderbar).

1. **Reiner Kern `ablauf`** (Phasenablauf, Zustände, Zyklus-Fortschritt, Überzeit, Fälligkeiten), ohne Abhängigkeit von egui, `tray-icon`/`muda` oder `windows-sys`. Die „injizierbare Uhr“ ist **kein Uhr-Objekt**: Jeder Aufruf bekommt `jetzt: Instant` als Parameter. Die Schnittstelle besteht aus `befehl(Befehl, jetzt) -> Vec<Ereignis>`, `takt(jetzt) -> Vec<Ereignis>` (beliebig oft aufrufbar), `stand(jetzt) -> Stand`, `naechste_faelligkeit(jetzt) -> Option<Instant>` und `einstellungen(Ablaufeinstellungen)`. Ergebnisse werden zurückgegeben, es gibt keine Seiteneffekte. Nach einem Zeitsprung (Standby) liefert `takt` genau ein `PhaseAbgelaufen`.
2. **`ablauf` setzt die Regel für geänderte Werte selbst um.** `Ablaufeinstellungen` enthält die drei Dauern, die Fokusphasen pro Zyklus und die beiden Auto-Start-Schalter. Die Auto-Start-Schalter gelten sofort, die Dauern ab der nächsten Phase dieses Typs, die Fokusphasen ab dem nächsten Zyklus. *Phase zurücksetzen* übernimmt die neue Dauer, *Neu beginnen* alles Neue.
3. **`anzeige(stand, theme, aufblinken) -> Anzeige`** ist eine reine Funktion und erzeugt einen fertigen, rein darstellenden Datensatz: Text (`24:13`/`+01:05`), Phasenwort, Akzentfarbe (gedimmt bzw. aufgeblinkt), Hintergrund, Fortschritt `0.0..=1.0` und Zyklus-Punkte. Das Overlay platziert nur noch. Tray-Tooltip und Tray-Icon leiten sich aus demselben Datensatz ab.
4. **`hinweis`** ist rein. `ausloesen(neue_phase, schalter, stoeren_erlaubt, jetzt) -> Hinweisaktionen { klang, einblenden }` merkt sich den Beginn des Aufblinkens. `aufblinken(jetzt) -> f32` liefert den Wert für `anzeige`, solange das Aufblinken läuft, mit eigener Fälligkeit für flüssige Frames. `stoeren_erlaubt` fragt die Schale nur beim Ablauf einer Phase über `SHQueryUserNotificationState` ab.
5. **`einstellungsdatei`** ist ein tiefes Modul mit `oeffnen(pfad)` (legt die Datei bei Bedarf kommentiert an), `aktuell() -> (&Einstellungen, &[Fehler])` (ungültige Werte fallen einzeln auf die Voreinstellung zurück), `aendern(Aenderung, jetzt)` (plant verzögertes Speichern), `takt(jetzt) -> Option<&Einstellungen>` (erkennt Hand-Änderungen, schreibt Fälliges atomar) und `sichern()` (Beenden, Herunterfahren). Intern gibt es die reine Naht `lesen(&str) -> (Einstellungen, Vec<Fehler>)` für die Prüftests. Es gibt **keinen Dateisystem-Trait**: Datei-Tests laufen gegen ein temporäres Verzeichnis. Die Datei ist die einzige Quelle der Einstellungen, auch Position und Größe des Overlays gehen als `Aenderung` hinein.
6. **`theme`** ist ein reines Datenmodul: `Theme { farben, bild, tray_zeichen }`, `lesen(&str) -> Result<Theme, Fehler>` (Fehlendes kommt von *Dunkel*), `eingebaute()` und `ordner_lesen(pfad)` fürs Menü. **`grafik`** macht daraus Pixel: `theme_setzen(&Theme, max_flaeche_px)` dekodiert im Hintergrund-Thread, `hintergrund(ctx) -> Option<TextureHandle>` holt das fertige Bild ab und `tray_icon(phase, &Theme)` wird zwischengespeichert. Getestet werden dort nur das Verkleinern und der Rückfall auf den Kreis.
7. **`plattform`** besteht aus freien Funktionen über `windows-sys` **ohne Trait**: Mutex/Event, Autostart, `SHQueryUserNotificationState`, `PlaySoundW`, Fensterstile, Alt+F4 verwerfen, Arbeitsbereich, Herunterfahren und `MessageBoxW`. Geprüft wird das manuell per Prüfliste in der SPEC. Den **zweiten Start** erkennt die laufende Instanz, indem sie das Event im Sekundentakt mit `WaitForSingleObject(event, 0)` abfragt. Es gibt keinen Warte-Thread. Einziger eigener Thread der App ist das Bild-Dekodieren.
8. **Ereignisfluss:** Eine Aufzählung `Aktion` (`Primaeraktion`, `Ueberspringen`, `PhaseZuruecksetzen`, `NeuBeginnen`, `Umschalten(Schalter)`, `ThemeWaehlen(Name)`, `EinstellungsfensterOeffnen`, `OverlayZeigen`, `Beenden`, …) gilt für alle Quellen. Die `muda`-IDs werden beim Menüaufbau zugeordnet, Leertaste und Linksklick ergeben beide `Primaeraktion`. Es gibt genau eine Verteilstelle `App::ausfuehren(aktion, jetzt)`. Ziehen und `Strg`+Mausrad gehen als `Aenderung` direkt in die Einstellungsdatei. Zurück erzeugt die reine Funktion `menue(stand, einstellungen, fehler, themes) -> Menuemodell` (Text, aktiv, angehakt, Fehlerzeilen oben), die nach jedem Event auf `muda` übertragen wird. `logic` läuft in fester Reihenfolge ab: Eingaben → `Aktion`, `einstellungsdatei.takt`, `ablauf.takt` plus `hinweis`, `anzeige` und `menue` anwenden, `request_repaint_after(min(Fälligkeiten, 1 s))`.
9. **Das Einstellungsfenster** ist ein immediate Viewport (`show_viewport_immediate`), nur solange es offen ist, mit direktem Zugriff auf `App`. Die Felder sind beschränkt (`DragValue` 1–180 bzw. 1–12, Häkchen, Theme-Auswahl), Fehler kommen deshalb nur aus Hand-Änderungen. Es gibt keinen eigenen Zwischenstand: Das Formular liest bei jedem Zeichnen `einstellungsdatei.aktuell()` und schickt jede Änderung sofort an `aendern`. Es ist ein normales Fenster mit Titelleiste, Taskleisten-Eintrag und Fokus und schließt per ✕ oder Esc.
10. **Crate-Aufbau:** eine Binär-Crate ohne Workspace und ohne getrennte lib, mit flachen Moduldateien:
    - `main.rs`: `windows_subsystem`, Mutex, Start, Panic-Hook
    - `app.rs`: Schale mit `logic` und `ausfuehren`
    - rein: `ablauf.rs`, `hinweis.rs`, `anzeige.rs`, `menue.rs`, `theme.rs`
    - Ein- und Ausgabe: `einstellungsdatei.rs`
    - Plattform: `grafik.rs`, `tray.rs` (Adapter für `tray-icon`/`muda`: `Menuemodell` setzen, Events in `Aktion` übersetzen, Tooltip), `plattform.rs`
    - egui: `overlay.rs` (zeichnet `Anzeige`, übersetzt Klick, Tasten, Ziehen und Mausrad), `einstellungsfenster.rs`
11. **Tests:** `#[cfg(test)]` im jeweiligen Modul, nur über dessen Schnittstelle. `ablauf` wird am ausführlichsten getestet (Zeit als `t0 + Duration`, Standby als Sprung), dazu `hinweis`, `anzeige`, `menue`, `theme::lesen`, `einstellungsdatei` (Strings plus temporäres Verzeichnis) und in `grafik` nur Verkleinern und Kreis-Rückfall. Overlay, Tray, Plattform und Einstellungsfenster werden per manueller Prüfliste in der SPEC geprüft. Es gibt keine GUI-Tests, kein Mocking von egui und keinen Dateisystem-Fake.
12. **Fehler sichtbar machen:** Ein Panic-Hook schreibt Meldung und Ort nach `%USERPROFILE%\.rusty_pomodoro_overlay\absturz.txt` (überschreibend) und zeigt eine `MessageBoxW` mit Text und Pfad. Startfehler (`run_native` mit `Err`) bekommen dieselbe MessageBox. Alle anderen Laufzeitfehler (Einstellungen, Themes, Bild, `PlaySoundW`, Autostart-Eintrag) erscheinen als Fehlerzeile oben im Menü und im Tooltip. Es gibt kein Logging-Crate.
