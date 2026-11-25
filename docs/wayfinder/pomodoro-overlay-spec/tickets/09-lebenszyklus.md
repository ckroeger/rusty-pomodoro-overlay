---
title: Lebenszyklus der App
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: [04]
---

# Lebenszyklus der App

## Question

Startet die App mit Windows (Autostart, optional?), wird nur eine Instanz zugelassen, wie beendet man sie, und übersteht der Timer-Stand einen Neustart der App (fortsetzen vs. frisch beginnen)?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Die Zeitbasis ist die Wanduhr. Wird der Timer-Stand über einen Neustart gerettet, kann man ihn darum wie nach einem Standby nachholen. Ohne gespeicherten Stand beginnt die App mit Fokusphase 1 im Zustand *bereit*.

> Kontext aus *[Bedienung](04-bedienung.md)*: Beendet wird über **Beenden** im gemeinsamen Menü, ohne Tastenkürzel. Zu klären ist, ob *ausgeblendet* und *Click-through* einen Neustart überdauern. Startet die App mit aktivem Click-through oder ausgeblendet, ist sie nur über das Tray-Icon erreichbar.

> Kontext aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: `tray-icon` verhindert **keine zweite Instanz**, dann gäbe es zwei Tray-Icons. Single-Instance muss die App selbst lösen. Das Tray-Icon verschwindet erst, wenn `TrayIcon` gedroppt wird. Bei einem Absturz mit `panic = "abort"` bleibt ein totes Icon stehen, bis man mit der Maus darüberfährt.

> Kontext aus *[Hinweis bei Phasenwechsel](06-phasenwechsel-hinweis.md)*: Wird der Timer-Stand beim Start nachgeholt und sind dabei Phasen abgelaufen, kommt **genau ein** Hinweis, passend zum letzten verpassten Übergang (wie nach Standby). Es gibt keinen Toast, eine eigene AppUserModelID ist dafür also nicht nötig.

> Kontext aus *[Einstellungen und Speichern](08-einstellungen.md)*: Einstellungen liegen in `%USERPROFILE%\.rusty_pomodoro_overlay\einstellungen.toml`, geschrieben mit `toml_edit`, atomar. *Overlay anzeigen* wird gespeichert, *Click-through* nicht (beim Start immer aus). Ein ausgeblendetes Overlay bleibt also auch nach dem Neustart ausgeblendet. Ein **gespeicherter Timer-Stand** gehört nicht in diese Datei, die der Nutzer bearbeitet. Falls er gespeichert wird, wäre eine eigene Datei im selben Ordner naheliegend. Zwei Instanzen würden außerdem gleichzeitig in dieselbe Datei schreiben, auch das spricht für Single-Instance.

## Resolution

**Der Timer-Stand wird nicht gespeichert, die Einstellungen schon. Es gibt nur eine Instanz, Autostart ist optional, beendet wird nur über das Menü.**

1. **Timer-Stand:** wird **nicht gespeichert**. Jeder Start beginnt mit Fokusphase 1 *bereit*, ohne Überzeit (wie in *Phasenablauf* für den Start ohne gespeicherten Stand), auch nach einem Neustart von Windows oder einem Absturz mitten in einer Phase. Es gibt keine Zustandsdatei und kein Nachholen beim Start. Das Nachholen aus *Phasenablauf* und *Hinweis bei Phasenwechsel* gilt also nur für Standby und Ruhezustand. Die **Einstellungen** bleiben erhalten (siehe *[Einstellungen und Speichern](08-einstellungen.md)*), einschließlich Position, Skalierung, *Overlay anzeigen* und Theme.
2. **Eine Instanz** pro Benutzersitzung:
   - Geprüft wird beim Start mit einem benannten Mutex (`Local\rusty_pomodoro_overlay`) über `windows-sys`, ohne zusätzliches Crate.
   - Ein **zweiter Start** setzt ein benanntes Event und beendet sich still. Die laufende Instanz fragt das Event in ihrem Sekundentakt ab, **blendet ihr Overlay ein**, falls es ausgeblendet ist, und lässt es kurz **aufblinken** („Ich bin schon da“). Das ist kein *Hinweis* im Sinne des Glossars, es gibt keinen Ton.
   - Ein `cargo run` neben einer laufenden, per `cargo install` installierten Instanz zählt ebenfalls als zweite Instanz.
3. **Autostart:**
   - Optional, **Voreinstellung aus**. Der Schalter *Mit Windows starten* steht **nur im Einstellungsfenster**.
   - Umgesetzt wird er mit einem Wert unter `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` über `windows-sys`. Dafür sind keine Adminrechte nötig, und der Eintrag ist im Task-Manager unter *Autostart-Apps* sichtbar.
   - **Maßgeblich ist der Registry-Eintrag**, nicht die Einstellungsdatei. Der Schalter zeigt, ob der Eintrag existiert, und legt ihn an oder löscht ihn.
   - Der Eintrag zeigt auf den Pfad der **laufenden Exe**. Schaltet man ihn aus `cargo run` heraus ein, zeigt er auf `target\…`. Die App korrigiert das nicht selbst, die Doku weist darauf hin.
   - Beim Autostart verhält sich die App wie bei jedem anderen Start. Einen eigenen Startmodus gibt es nicht.
4. **Beenden:**
   - Beendet wird nur über **Beenden** im gemeinsamen Menü, **sofort und ohne Rückfrage**, auch wenn eine Phase läuft.
   - **Alt+F4 auf dem Overlay** wird verworfen. Im Einstellungsfenster schließt Alt+F4 nur dieses Fenster, so wie X und Esc.
   - Beim **Beenden, Herunterfahren und Abmelden** (`WM_QUERYENDSESSION`/`WM_ENDSESSION`) schreibt die App noch ausstehende Einstellungen sofort, etwa eine Position, die wegen der Wartezeit von 1 s noch nicht gespeichert ist. Sie entfernt das Tray-Icon und gibt den Mutex frei. Windows hält sie nicht auf.
   - **Absturz:** Das tote Tray-Icon bleibt stehen, bis man mit der Maus darüberfährt. Das nehmen wir in Kauf.

Verworfen: einen Timer-Stand speichern (mit oder ohne Verfallsgrenze), den Stand nur bei Herunterfahren oder Absturz zu retten, Mehrfachinstanzen, einen stillen zweiten Start ohne Signal, eine Verknüpfung im Autostart-Ordner, eine geplante Aufgabe, eine Rückfrage beim Beenden, Alt+F4 als Beenden.
