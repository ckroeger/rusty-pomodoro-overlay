---
title: Lebenszyklus der App
labels: [wayfinder:grilling]
status: open
assignee:
blocked_by: [04]
---

# Lebenszyklus der App

## Question

Startet die App mit Windows (Autostart, optional?), wird nur eine Instanz zugelassen, wie beendet man sie, und übersteht der Timer-Stand einen Neustart der App (fortsetzen vs. frisch beginnen)?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Die Zeitbasis ist die Wanduhr. Wird der Timer-Stand über einen Neustart gerettet, kann man ihn darum wie nach einem Standby nachholen. Ohne gespeicherten Stand beginnt die App mit Fokusphase 1 im Zustand *bereit*.

> Kontext aus *[Bedienung](04-bedienung.md)*: Beendet wird über **Beenden** im gemeinsamen Menü, ohne Tastenkürzel. Zu klären ist, ob *ausgeblendet* und *Click-through* einen Neustart überdauern. Startet die App mit aktivem Click-through oder ausgeblendet, ist sie nur über das Tray-Icon erreichbar.

> Kontext aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: `tray-icon` verhindert **keine zweite Instanz**, dann gäbe es zwei Tray-Icons. Single-Instance muss die App selbst lösen. Das Tray-Icon verschwindet erst, wenn `TrayIcon` gedroppt wird. Bei einem Absturz mit `panic = "abort"` bleibt ein totes Icon stehen, bis man mit der Maus darüberfährt.
