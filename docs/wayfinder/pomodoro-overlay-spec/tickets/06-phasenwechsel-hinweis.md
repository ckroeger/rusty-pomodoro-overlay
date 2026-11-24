---
title: Hinweis bei Phasenwechsel
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: []
---

# Hinweis bei Phasenwechsel

## Question

Wie erfährst du, dass eine Phase endet: Ton, Windows-Toast-Benachrichtigung, visuelles Signal im Overlay (Blinken, Farbwechsel), oder eine Kombination? Abschaltbar? Unterscheidet sich der Hinweis zwischen Fokus→Pause und Pause→Fokus?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Ein Übergang kann automatisch starten oder in *bereit* enden, je Richtung per Schalter. Läuft eine Phase ab, während der Rechner im Standby ist, wird der Stand beim Aufwachen nachgeholt, dabei können mehrere Phasen abgelaufen sein. Zu klären ist, ob und wie dann ein Hinweis kommt (einer oder keiner, nicht einer pro verpasster Phase?).

> Kontext aus *[Bedienung](04-bedienung.md)*: Das Overlay kann *ausgeblendet* sein (nur das Tray-Icon bleibt) oder im Click-through-Modus. Zu klären ist, ob ein Phasenwechsel ein ausgeblendetes Overlay wieder einblendet, und ob das Tray-Icon selbst als Hinweis dient.

> Kontext aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: `tray-icon` hat **keine Balloon- oder Toast-API**, kann aber pro Phase ein anderes Icon setzen (`set_icon`). Ein Windows-Toast braucht ein eigenes Crate oder WinRT. Ein Balloon am selben Icon ginge nur mit einer GUID, und die ist an den Pfad der Exe gebunden (`cargo run` und `cargo install` haben verschiedene Pfade).

## Resolution

**Drei Kanäle: Ton, Aufblinken im Overlay, Tray-Icon je Phase. Kein Windows-Toast in v1.**

- **Auslöser:** Ein Hinweis kommt nur, wenn die Restzeit einer Phase **abläuft**, egal ob die nächste Phase dann automatisch startet oder *bereit* ist. Kein Hinweis beim Überspringen, beim Starten einer bereiten Phase, bei *Phase zurücksetzen* oder *Neu beginnen*, denn da hat der Nutzer selbst gehandelt.
- **Ton:** zwei eigene, kleine `.wav`, eingebettet in die Exe (keine Windows-Systemklänge, die hängen vom Soundschema ab). Einer für *Fokusphase endet → Pause*, einer für *Pause endet → Fokusphase*. Die lange Pause hat keinen eigenen Klang. Eigene Klangdateien gibt es in v1 nicht. Es gilt die Windows-Lautstärke für die App, einen eigenen Lautstärkeregler gibt es nicht.
- **Aufblinken im Overlay:** dreimal in der Akzentfarbe der *neuen* Phase, insgesamt ca. 2 s, danach die normale Anzeige. Bei *bereit* blinkt nichts dauerhaft, die gedimmte Überzeit `+MM:SS` erinnert still. Im Click-through-Modus blinkt es ganz normal.
- **Tray-Icon je Phase:** wird zur Laufzeit gezeichnet. Voreinstellung ist ein gefüllter Kreis in der Akzentfarbe der aktuellen Phase, gedimmt bei *angehalten* und *bereit*. Ein Theme kann statt des Kreises ein Unicode-Zeichen (z. B. ⌛) vorgeben. Das Icon zeigt den Zustand an und lässt sich nicht abschalten. Kein Tortendiagramm, die Restzeit steht im Tooltip.
- **Schalter:**
  - *Ton* (Voreinstellung an): in den Einstellungen **und** als Häkchen im gemeinsamen Menü, zum schnellen Stummschalten.
  - *Signal im Overlay* (Voreinstellung an): nur in den Einstellungen.
  - *Overlay bei Phasenwechsel einblenden* (Voreinstellung aus): nur in den Einstellungen. Ist er an, wird ein *ausgeblendetes* Overlay beim Hinweis eingeblendet, blinkt auf und bleibt danach sichtbar. Ist er aus, bleibt es ausgeblendet, dann bleiben Ton und Tray-Icon als Hinweis.
- **Nachholen** (Standby, ggf. Neustart, siehe *Lebenszyklus der App*): Sind dabei eine oder mehrere Phasen abgelaufen, kommt **genau ein** Hinweis, sofort. Klang und Farbe richten sich nach dem *letzten* verpassten Übergang.
- **Bitte nicht stören / Präsentation / Vollbild:** Meldet Windows, dass der Nutzer nicht gestört werden will, wird **nur der Ton** unterdrückt. Aufblinken und Tray-Icon bleiben. Einen Schalter dafür gibt es nicht. Das gilt nur, wenn sich der Zustand mit einer **dokumentierten API** zuverlässig abfragen lässt, sonst wird er ignoriert. Geklärt wird das in *[Ton abspielen und „nicht stören“ erkennen](12-ton-und-nicht-stoeren.md)*.

Verworfen: Toast (eigenes Crate/WinRT, ohne Installer keine registrierte AppUserModelID), Schalter je Richtung, ein dritter Klang für die lange Pause, Pulsieren bis zur Reaktion, ein Hinweis pro verpasster Phase, ungefragtes Einblenden als Voreinstellung.
