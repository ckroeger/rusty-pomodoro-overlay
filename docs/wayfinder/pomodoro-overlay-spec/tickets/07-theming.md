---
title: Theming
labels: [wayfinder:grilling]
status: closed
assignee: ckr
blocked_by: [05]
---

# Theming

## Question

Was legt ein Theme fest (Farben je Phase, Hintergrund-Transparenz, Schrift, Größe?), wie viele Themes bringt v1 mit, kann man eigene Themes definieren, und wie wechselt man das Theme?

## Kontext

- Aus [GUI-Technik für das Overlay](01-gui-technik.md): Transparenz pro Pixel (Hintergrund durchsichtig, Text deckend) hängt von Renderer/GPU ab; eine einheitliche Fenster-Deckkraft (`SetLayeredWindowAttributes`) ist überall sicher. Entscheiden, ob einheitliche Deckkraft für v1 reicht. Ergebnis von [Overlay-Spike mit der gewählten Technik](02-overlay-spike.md) beachten.

> Kontext aus *[Bedienung](04-bedienung.md)*: Das gemeinsame Menü (Overlay und Tray) enthält einen Eintrag **Theme ▸**. Wie das Untermenü aussieht (Liste der Themes mit Häkchen?), wird hier entschieden.

> Kontext aus *[Crates für Tray-Icon und gemeinsames Menü](11-tray-und-menue-crates.md)*: Das gemeinsame Menü ist ein **natives Windows-Menü** (muda). Es bleibt hell und folgt dem Theme nicht. Theming betrifft also nur das Overlay, optional dazu ein Tray-Icon pro Phase.

> Kontext aus *[Inhalt und Verhalten des Overlays](05-overlay-anzeige.md)*: Das Layout ist in v1 **fest**. Ein Theme ändert nur das Aussehen (Farben, Transparenz, ggf. Schrift), nicht Anordnung oder Elemente. Die **Größe gehört nicht zum Theme**, sie wird per `Strg`+Mausrad eingestellt. Ein Theme braucht mindestens: eine **Akzentfarbe je Phase** (Fokus, kurze Pause, lange Pause; daran wird die Phase erkannt, auch bei Fortschrittsbalken und hervorgehobenem Zyklus-Punkt), eine Darstellung für **gedimmt** (*angehalten*, *bereit*), Text-, Hintergrund- und Punktfarben. Die Restzeit verwendet Ziffern gleicher Breite, was bei der Schriftwahl zu beachten ist.

> Kontext aus *[Hinweis bei Phasenwechsel](06-phasenwechsel-hinweis.md)*: Das **Tray-Icon** wird zur Laufzeit gezeichnet: Voreinstellung ist ein gefüllter Kreis in der Akzentfarbe der Phase, gedimmt bei *angehalten*/*bereit*. Ein Theme kann **statt des Kreises ein Unicode-Zeichen** (z. B. ⌛) vorgeben. Hier zu entscheiden: ein Zeichen für alle Phasen oder eins je Phase, Farbe (Akzentfarbe?), und welche Schrift das Zeichen in 16×16 rendert (die Standardschrift von egui hat z. B. keine Emoji). Das **Aufblinken** beim Phasenwechsel nutzt die Akzentfarbe der neuen Phase, ein Theme braucht dafür keine eigene Farbe.

## Resolution

**Ein Theme legt nur Farben, die Deckkraft des Hintergrunds, optional ein Hintergrundbild und optional ein Tray-Zeichen fest. Eingebaut sind *Dunkel* (Voreinstellung) und *Hell*. Eigene Themes sind Dateien im Theme-Ordner.**

1. **Transparenz:** Alpha pro Pixel. Nur der Hintergrund ist durchsichtig, Ziffern, Punkte, Balken und Symbole sind immer deckend. Eine einheitliche Fenster-Deckkraft gibt es nicht.
2. **Schrift:** gehört nicht zum Theme. Die App bettet eine feste Schrift mit gleich breiten Ziffern ein, welche genau, legt die Spec fest. Wählbare Schriften gehören zu den frei gestaltbaren Layouts (*Out of scope*).
3. **Farbfelder** (sechs):
   - `fokus`, `kurze_pause`, `lange_pause`: Akzentfarbe je Phase für Phasenwort, Fortschrittsbalken, Ring um den aktuellen Zyklus-Punkt, Aufblinken und Tray-Kreis.
   - `hintergrund` (mit Deckkraft): Fläche des Overlays.
   - `vordergrund`: Restzeit/Überzeit, volle Zyklus-Punkte, Pause- und Startsymbol.
   - `gedaempft`: leere Zyklus-Punkte, Spur des Fortschrittsbalkens.
   - **Gedimmt** (*angehalten*/*bereit*) ist keine eigene Farbe. Die App mischt mit einem festen Faktor (Richtwert 50 %) zum Hintergrund hin, gleich für alle Themes.
4. **Hintergrundbild** (optional, `bild`, JPG oder PNG):
   - Es ersetzt die Füllfarbe von `hintergrund`. Die **Deckkraft von `hintergrund` gilt für das Bild**, bei PNG wird der Alphakanal damit multipliziert. Freigestellte Motive sind an den Rändern also ganz durchsichtig.
   - Es **füllt** das Overlay: Es wird proportional skaliert, bis es die Fläche ganz bedeckt, und mittig beschnitten. Es verzerrt nicht, und die Form des Overlays bleibt fest. Passt das Motiv nicht, schneidet man das Bild selbst auf etwa 2:1 zu. Eine Form, die sich nach dem Bild richtet, wäre frei gestaltbares Layout (*Out of scope*).
5. **Eingebaute Themes:** Eingebettet und nicht änderbar sind *Dunkel* (Voreinstellung) und *Hell*, beide ohne Bild und mit Kreis im Tray. Startwerte:

   | Feld | Dunkel | Hell |
   |---|---|---|
   | `hintergrund` | `#1E1E1E` bei 85 % | `#F5F5F5` bei 90 % |
   | `vordergrund` | `#F0F0F0` | `#202020` |
   | `gedaempft` | `#5A5A5A` | `#B0B0B0` |
   | `fokus` | `#E5534B` | `#C62828` |
   | `kurze_pause` | `#4CAF50` | `#2E7D32` |
   | `lange_pause` | `#42A5F5` | `#1565C0` |

   Die App folgt **nicht** automatisch dem Hell-/Dunkelmodus von Windows.
6. **Eigene Themes:** je eine Datei im **Theme-Ordner** neben den Einstellungen. Der Name im Menü kommt aus einem Feld `name` oder aus dem Dateinamen. Fehlende Felder **erben von *Dunkel***, ein Pip-Boy-Theme braucht also nur `bild` und die Akzente. Bildpfade werden **relativ zur Theme-Datei** aufgelöst. Dateiformat und genauer Ort entscheidet *Einstellungen und Speichern*.
7. **Menü *Theme ▸*:** In dieser Reihenfolge: *Dunkel*, *Hell*, Trenner, eigene Themes alphabetisch, Trenner, *Theme-Ordner öffnen*. Das aktive Theme trägt ein Häkchen. Ein Klick wechselt **sofort**, die Wahl wird gespeichert. Der Ordner wird **bei jedem Öffnen des Menüs** neu gelesen, ein erneuter Klick auf das aktive Theme lädt es neu. Einen Datei-Watcher gibt es nicht. *Theme-Ordner öffnen* öffnet den Ordner im Explorer und legt ihn vorher an, falls er fehlt.
8. **Fehler** stehen nur als Zusatz im Menütext. Es gibt keine Meldungsfenster und keine Logdatei.
   - Ist die Datei nicht lesbar (Syntaxfehler), ist das Theme ausgegraut, z. B. `Pip-Boy (Fehler in Zeile 4)`.
   - Ist ein Feld ungültig oder unbekannt, wird es ignoriert und der Wert aus *Dunkel* verwendet.
   - Fehlt das Bild oder lässt es sich nicht dekodieren, bleibt das Theme wählbar und verwendet die Farbe `hintergrund`. Im Menü steht z. B. `Pip-Boy (Bild fehlt)`.
   - Ist das gespeicherte aktive Theme beim Start weg oder kaputt, verwendet die App *Dunkel*. Die gespeicherte Wahl bleibt erhalten und gilt wieder, sobald die Datei repariert ist.
9. **Tray-Zeichen** (optional, `tray_zeichen`): **ein Zeichen für alle Phasen**, gezeichnet **einfarbig** in der Akzentfarbe der Phase und gedimmt wie der Kreis. Farbige Emoji werden ebenfalls einfarbig gezeichnet, soweit die Schrift das hergibt. Ist das Feld leer oder das Zeichen nicht darstellbar, erscheint der Kreis.

**Offene Fakten** stehen im Ticket [Bilddecoder und Tray-Zeichen rendern](13-bilddecoder-und-tray-zeichen.md).

Verworfen: einheitliche Fenster-Deckkraft, wählbare oder beliebige Schriften, eigene Farben für Dimmung oder Phasenwort, eine eigene Deckkraft für das Bild, Bild strecken oder einpassen, eine Overlay-Form nach dem Bild, ein Tray-Zeichen je Phase, Datei-Watcher, Folgen des Windows-Modus.

## Nachtrag (aus *[SPEC.md zusammenschreiben](16-spec-zusammenschreiben.md)*, 2026-11-26)

Punkt 2 ist geändert: Die Schrift gehört **doch zum Theme**, als Feld `schrift` mit drei eingebauten Werten: `ubuntu` (Ubuntu Light aus egui, Voreinstellung, auch für *Dunkel*/*Hell*), `inter` (Inter mit eingefrorenem `tnum`) und `jetbrains_mono`. Beliebige Schriftdateien bleiben ausgeschlossen. Grund: Ubuntu Light ist bei kleiner Skalierung recht zart. Ein Theme wie Pip-Boy soll eine passende Schrift mitbringen können.

Punkt 7 ist ebenfalls geändert: Der Theme-Ordner wird **im Sekundentakt** neu gelesen (nur Dateinamen und Änderungszeitpunkte, geparst wird nur Geändertes), nicht bei jedem Öffnen des Menüs. Grund: Das Tray-Menü öffnet `tray-icon` selbst, und solange es offen ist, steht die Event-Loop. Das Untermenü ist beim Öffnen also höchstens 1 s alt.
