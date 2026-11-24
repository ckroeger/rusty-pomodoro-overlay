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
