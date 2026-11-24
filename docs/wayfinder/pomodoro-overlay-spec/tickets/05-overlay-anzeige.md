---
title: Inhalt und Verhalten des Overlays
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: [04]
---

# Inhalt und Verhalten des Overlays

## Question

Was zeigt das Overlay (Restzeit-Format, Phase, Zyklus-Fortschritt, Fortschrittsbalken/-ring?), wie groß ist es, wo erscheint es beim Start, ist es verschiebbar und merkt es sich seine Position, und gibt es einen Click-through-Modus (und wie verträgt der sich mit der in *Bedienung* gewählten Steuerung)?

## Kontext

- Aus *[Phasenablauf](03-phasenablauf.md)*: Es gibt drei Zustände (läuft, angehalten, bereit), die das Overlay unterscheidbar zeigen muss, dazu den Zyklus-Fortschritt (z. B. 2/4). Im Zustand *bereit* nach einer abgelaufenen Phase kommt die **Überzeit** dazu (z. B. „+07:00“). Wie sie aussieht, wird hier entschieden.

> Kontext aus *[Bedienung](04-bedienung.md)*: Click-through ist entschieden: ein Schalter, Voreinstellung aus, ausschalten nur über das Tray-Icon. Ein Linksklick ohne Bewegung löst die Primäraktion aus, Ziehen verschiebt das Overlay. Der Tray-Tooltip zeigt den Stand **im gleichen Format wie das Overlay**, das hier festgelegte Format gilt also für beide. Das Overlay kann *ausgeblendet* sein (Schalter „Overlay anzeigen“).

## Resolution

Begriffe (*Überzeit*, *Zyklus-Fortschritt*, Zustände) stehen in [GLOSSARY.md](../../../../GLOSSARY.md).

1. **Elemente:** Groß in der Mitte steht die **Restzeit**. Darunter steht die **Phase**: Sie wird über die Akzentfarbe unterschieden und zusätzlich mit einem kurzen Wort benannt („Fokus“, „Pause“, „Lange Pause“). Es folgen die **Zyklus-Punkte** und am unteren Rand ein **dünner Fortschrittsbalken** (Anteil der verstrichenen Phase). Es gibt keinen Ring. Der Zustand hat kein eigenes Element: *Läuft* wird normal dargestellt. *Angehalten* zeigt die Restzeit gedimmt mit Pausensymbol. *Bereit* zeigt die volle Dauer gedimmt mit Startsymbol.
2. **Zeitformat:** Immer `MM:SS`, die Minuten dürfen mehr als zwei Stellen haben (`180:00`, `+125:30`). Die Restzeit wird **aufgerundet** (Start bei `25:00`, `00:00` erst im Moment des Ablaufs). Die Überzeit wird mit vorangestelltem `+` gezeigt und **abgerundet** (beginnt bei `+00:00`). Alle Ziffern haben **gleiche Breite**, damit die Anzeige nicht zappelt.
3. ***Bereit* mit Überzeit:** Das Overlay zeigt Farbe und Wort der **nächsten** (also der aktuellen) Phase. Groß steht die Überzeit, **nicht gedimmt**, sondern in voller Helligkeit, dazu das Startsymbol. Die Dauer der nächsten Phase wird nicht gezeigt, und der Fortschrittsbalken ist leer.
4. **Zyklus-Punkte:** ein Punkt je Fokusphase des Zyklus, alle auf **einer Zeile** (bei bis zu 12 Punkten werden sie kleiner). **Voll** steht für eine erledigte Fokusphase (abgelaufen oder übersprungen), **hervorgehoben** (Ring in der Akzentfarbe) für die aktuelle Fokusphase, **leer** für eine noch offene. Beispiele bei 4: Fokusphase 1 → `◐○○○`, kurze Pause nach 2 → `●●○○`, Fokusphase 3 → `●●◐○`, lange Pause → `●●●●`.
5. **Größe:** Die Grundgröße liegt bei etwa **180 × 90 logischen Pixeln** (Richtwert). Das Overlay beachtet die Windows-Anzeigeskalierung (DPI). Die Größe ändert sich **nur mit `Strg`+Mausrad**. Es gibt weder Ränder zum Ziehen noch einen Regler. Die Skalierung ist proportional (inklusive Schrift, gleiches Seitenverhältnis), **10 % pro Rastung, 50 % bis 300 %**, um die **Mitte** herum. Danach wird das Overlay so verschoben, dass es vollständig im Arbeitsbereich bleibt. Zurück geht es nur durch Zurückrollen. Bei aktivem Click-through wirkt das Mausrad nicht. Die Größe wird gespeichert.
6. **Position:** Beim ersten Start erscheint das Overlay **unten rechts im Arbeitsbereich des Hauptmonitors** (ohne Taskleiste), mit etwa 24 logischen Pixeln Abstand zum Rand. Danach wird die Position in virtuellen Bildschirmkoordinaten gemerkt und beim Start wiederhergestellt. Liegt das Overlay nicht vollständig in einem Arbeitsbereich, wird es in den nächstgelegenen geschoben. Ist es gar nicht mehr sichtbar, kommt es an die Startposition zurück. Das gilt auch zur Laufzeit, z. B. wenn ein Monitor abgesteckt oder die Auflösung geändert wird.
7. **Verschieben:** frei und **ohne Einrasten**. Nach dem Loslassen wird das Overlay wie in Punkt 6 in den Arbeitsbereich hineingeschoben, es kann also nie halb verschwinden.
8. **Fenster:** Das Overlay hat **keine Taskleisten-Schaltfläche** und erscheint **nicht in Alt+Tab**. Es nimmt sich **nie selbst den Fokus**, weder beim Start noch beim Phasenwechsel oder beim Wiedereinblenden. Den Fokus bekommt es nur durch eine Mausaktion: Klick (der zugleich die Primäraktion auslöst), Ziehen oder Rechtsklick. Beim Hovern passiert nichts (kein Tooltip, keine Hervorhebung).
9. **Tray-Tooltip:** eine Zeile mit Phase und Zyklus-Fortschritt, Zustand und Zeit, z. B. `Fokus 3/4 · läuft · 12:34`, `Pause nach 2/4 · angehalten · 03:10`, `Lange Pause · bereit · +07:00`.

Für die Architektur festgehalten: Das Zeichnen bekommt die Anzeigedaten (Restzeit bzw. Überzeit, Phase, Zustand, Zyklus-Fortschritt, Anteil) als einen Datensatz und kennt die Timer-Logik nicht. So kann ein späteres, frei gestaltbares Layout dieselben Daten anders darstellen (siehe *Out of scope* der Karte).
