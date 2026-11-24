# Rusty Pomodoro Overlay

Ein persönlicher Pomodoro-Timer für Windows, der die Restzeit der aktuellen Phase in einem kleinen, randlosen Fenster über dem Desktop anzeigt.

## Sprache

**Overlay**:
Das kleine, randlose Fenster ohne Titelleiste, das Restzeit und aktuelle Phase zeigt; die einzige sichtbare Oberfläche der App.
_Vermeiden_: Widget, Popup, Hauptfenster

**Immer-im-Vordergrund**:
Ein an- und abschaltbarer Modus, in dem das Overlay über allen anderen Fenstern liegt; ausgeschaltet verhält es sich wie ein normales randloses Fenster.
_Vermeiden_: Always-on-top-Fenster, Pin-Modus

**Phase**:
Ein zeitlich begrenzter Abschnitt des Pomodoro-Ablaufs; entweder Fokusphase, kurze Pause oder lange Pause.
_Vermeiden_: Session, Intervall, Runde

**Fokusphase**:
Die Phase konzentrierten Arbeitens (klassisch 25 Minuten).
_Vermeiden_: Pomodoro (als Phasenname), Arbeitsphase, Sprint

**Theme**:
Ein benannter Satz visueller Einstellungen für das Overlay (z. B. Farben, Transparenz).
_Vermeiden_: Skin, Stil

**Läuft**:
Zustand der aktuellen Phase, in dem ihre Restzeit abläuft.

**Angehalten**:
Zustand der aktuellen Phase, in dem ihre Restzeit stillsteht und erhalten bleibt; man kommt durch *Anhalten* hinein und durch *Fortsetzen* wieder heraus.
_Vermeiden_: pausiert, Pausieren (kollidiert mit der Phase *Pause*)

**Bereit**:
Zustand, in dem die nächste Phase feststeht, aber noch nicht gestartet wurde; von ihr ist noch keine Zeit verstrichen.
_Vermeiden_: wartend, gestoppt

**Zyklus**:
Die Folge von Fokusphasen mit kurzen Pausen dazwischen, die mit der langen Pause endet; danach beginnt ein neuer Zyklus.
_Vermeiden_: Runde, Satz, Session

**Zyklus-Fortschritt**:
Die wievielte Fokusphase des aktuellen Zyklus gerade dran ist (z. B. 2 von 4).
_Vermeiden_: Zähler

**Überspringen**:
Die aktuelle Phase sofort beenden und zur nächsten übergehen; eine übersprungene Fokusphase zählt für den Zyklus-Fortschritt wie eine abgelaufene.

**Phase zurücksetzen**:
Die aktuelle Phase erhält wieder ihre volle Dauer und ist bereit; der Zyklus-Fortschritt bleibt.

**Neu beginnen**:
Den aktuellen Zyklus verwerfen und einen neuen mit der ersten Fokusphase im Zustand bereit beginnen.
_Vermeiden_: Reset (mehrdeutig)
