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
