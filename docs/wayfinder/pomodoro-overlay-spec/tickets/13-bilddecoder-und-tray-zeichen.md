---
title: Bilddecoder und Tray-Zeichen rendern
labels: [wayfinder:research]
status: open
assignee:
blocked_by: [07]
---

# Bilddecoder und Tray-Zeichen rendern

## Question

Zwei Faktenfragen für eine Rust-App mit `eframe`/`egui` 0.37 (+ `glow`, winit, `tray-icon` 0.26, `windows-sys` 0.61) unter Windows 11:

1. **Hintergrundbild:** Womit dekodiert die App zur Laufzeit ein vom Nutzer gewähltes **JPG oder PNG** (PNG mit Alphakanal) und zeigt es als egui-Textur? Das Bild wird proportional skaliert und mittig beschnitten, bis es das Overlay füllt, und mit einer Deckkraft multipliziert. Kandidaten: `image` mit nur den Features `jpeg` + `png`, `egui_extras`-Loader, `zune-jpeg` + `png` direkt, WIC über `windows-sys`. Bewertet nach Binärgröße (zuletzt 4,68 MB, Release/LTO/strip), Aufwand und Verhalten bei großen Bildern (vorab herunterskalieren?).
2. **Tray-Zeichen:** Wie zeichnet die App ein beliebiges **Unicode-Zeichen** (z. B. `☢`, `⌛`, auch Emoji wie `🍅`) **einfarbig in einer gegebenen Farbe** in ein RGBA-Icon von 16 × 16 bzw. 32 × 32 (je nach DPI) für `tray-icon`? Kandidaten: GDI/DirectWrite über `windows-sys` mit *Segoe UI Symbol* bzw. Schriftfallback, `ab_glyph`/`fontdue` mit einer eingebetteten Symbolschrift, egui-Rasterung. Zu prüfen: Wie erkennt man ein nicht darstellbares Zeichen (Rückfall auf den Kreis)? Lassen sich farbige Emoji (*Segoe UI Emoji*) einfarbig rendern?

Ergebnis: Empfehlung mit Begründung für beide Teile.

## Kontext

- Aus *[Theming](07-theming.md)*: Ist das Bild nicht dekodierbar, gilt die Hintergrundfarbe mit dem Hinweis „(Bild fehlt)“ im Menü. Ein Tray-Zeichen für alle Phasen, eingefärbt in der Akzentfarbe, gedimmt bei *angehalten*/*bereit*. Rückfall ist der gefüllte Kreis.
- Spike-Code zum Ausprobieren: Branch `prototype/02-overlay-spike`.
