---
title: Architektur der App
labels: [wayfinder:grilling]
status: open
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
