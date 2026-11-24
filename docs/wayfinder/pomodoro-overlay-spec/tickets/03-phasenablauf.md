---
title: Phasenablauf
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: []
---

# Phasenablauf

## Question

Wie läuft der Pomodoro-Zyklus genau ab? Dauer von Fokusphase, kurzer und langer Pause (fest oder konfigurierbar), nach wie vielen Fokusphasen die lange Pause kommt, ob die nächste Phase automatisch startet oder auf Bestätigung wartet, und was Pausieren, Überspringen und Zurücksetzen mit dem Zyklus machen.

## Resolution

Begriffe (Zustände *läuft*/*angehalten*/*bereit*, *Zyklus*, *Zyklus-Fortschritt*, *Überspringen*, *Phase zurücksetzen*, *Neu beginnen*) stehen in [GLOSSARY.md](../../../../GLOSSARY.md).

1. **Zyklus:** Voreinstellungen sind Fokusphase 25 Min, kurze Pause 5 Min, lange Pause 15 Min und 4 Fokusphasen pro Zyklus. Alle vier Werte sind konfigurierbar. Die Dauern werden in **ganzen Minuten** angegeben, **1 bis 180**. *Fokusphasen pro Zyklus* reicht von **1 bis 12** (bei 1 folgt auf jede Fokusphase die lange Pause). Bei ungültigen Werten gilt die Voreinstellung, und die App weist darauf hin. Wie sie das tut, regelt *Einstellungen und Speichern*.
2. **Zustände:** Die aktuelle Phase *läuft*, ist *angehalten* (Restzeit bleibt erhalten, *Fortsetzen* lässt sie weiterlaufen) oder ist *bereit* (die nächste Phase steht fest, aber noch ist keine Zeit verstrichen). Beim App-Start ohne gespeicherten Stand ist Fokusphase 1 *bereit*. Was bei gespeichertem Stand gilt, regelt *Lebenszyklus der App*.
3. **Übergang beim Ablauf:** Für jede Richtung gibt es einen eigenen Schalter (automatisch starten oder *bereit*). Voreinstellung: Fokus→Pause startet **automatisch**, Pause→Fokus wird ***bereit***.
4. **Überspringen** geht in jedem Zustand (läuft, angehalten, bereit). Danach folgt die nächste Phase so, wie es der Schalter aus Punkt 3 vorgibt. Eine übersprungene Fokusphase zählt für den Zyklus-Fortschritt **wie eine abgelaufene**. Nach der langen Pause beginnt ein neuer Zyklus.
5. **Zwei getrennte Aktionen:**
   - *Phase zurücksetzen*: Die aktuelle Phase bekommt wieder ihre volle Dauer (nach aktueller Einstellung) und ist *bereit*. Der Zyklus-Fortschritt bleibt.
   - *Neu beginnen*: Der aktuelle Zyklus wird verworfen. Danach ist Fokusphase 1 *bereit*.
6. **Überzeit:** Ist eine Phase abgelaufen und die nächste *bereit*, wird die Zeit seit dem Ablauf mitgezählt und angezeigt (z. B. „+07:00“). Das ist **nur eine Anzeige** und wirkt nicht auf den Ablauf. Wurde *bereit* durch *Phase zurücksetzen*, *Neu beginnen* oder den App-Start erreicht, gibt es keine Überzeit. Das Aussehen regelt *Inhalt und Verhalten des Overlays*.
7. **Zeitbasis = Wanduhr:** Standby und Ruhezustand verbrauchen Zeit, Sperren (Win+L) hat keine Wirkung. Es gibt kein automatisches Anhalten. Sind beim Aufwachen eine oder mehrere Phasen abgelaufen, wird der Stand so nachgeholt, als wäre die App wach gewesen (bei automatischen Übergängen über mehrere Phasen hinweg).
8. **Geänderte Werte** gelten erst ab der **nächsten Phase dieses Typs**, *Fokusphasen pro Zyklus* erst ab dem **nächsten Zyklus**. Die laufende Phase behält ihre Dauer. *Phase zurücksetzen* übernimmt die neue Dauer.

Für die Architektur festgehalten: Die Timer-Logik bekommt eine injizierbare Uhr. Dadurch sind Sekunden-Dauern in der Konfiguration zum Testen nicht nötig.
