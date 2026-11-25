---
title: Bilddecoder und Tray-Zeichen rendern
labels: [wayfinder:research]
status: closed
assignee: ckr
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

## Resolution

**Hintergrundbild: `image` 0.25 mit nur `jpeg` + `png` (dieselbe Version, die eframe schon mitbringt), vor dem Hochladen verkleinert, in einem kurzen Hintergrund-Thread dekodiert.**
- Ablauf: `ImageReader::open` → `with_guessed_format` → `into_decoder` → `orientation` → `DynamicImage::from_decoder` → `apply_orientation` (EXIF-Drehung von Handyfotos) → bei Bedarf `thumbnail_exact` → `into_rgba8`. Das Ergebnis geht per Kanal an die UI, danach `request_repaint()`. Bis dahin gilt die Farbe `hintergrund`, jeder Fehler wird zu „(Bild fehlt)“.
- **Verkleinern ist Pflicht:** `egui_glow` bricht bei Texturen über `GL_MAX_TEXTURE_SIZE` mit `assert!` ab, bei `panic = "abort"` ist das ein Absturz. Ziel ist die Fläche, die das Overlay bei 300 % Zoom mal DPI-Faktor bedeckt. Größere Bilder werden nur verkleinert, nie vergrößert. Kleinere Zoomstufen übernimmt die GPU mit Mipmaps (`TextureOptions { mipmap_mode: Some(Linear), ..LINEAR }`).
- Zeichnen: `ColorImage::from_rgba_unmultiplied` behält den Alphakanal. Mittig beschnitten wird über das UV-Rechteck, die Deckkraft kommt über den Farbton `Color32::from_white_alpha(deckkraft)` dazu, der wegen vormultipliziertem Alpha genau Deckkraft × Bild-Alpha ergibt.
- Gemessen (Spike, Release/LTO/strip, Basis 4,69 MB): **+336 KB**, neu sind nur `zune-jpeg` und `zune-core`. Ein Foto mit 6000 × 4000 braucht 0,12 s (PNG) bis 0,19 s (JPEG) inkl. Verkleinern, kaputte und fehlende Dateien geben sauber einen Fehler.
- **Verworfen:**
  - WIC: +9,5 KB und am schnellsten (69 ms), geht aber nicht über `windows-sys`. Es bräuchte das `windows`-Crate (10 Crates mehr), `unsafe`-COM und eine eigene EXIF-Drehung. Es ist die Rückfalloption, falls die Größe wichtiger wird.
  - `zune-jpeg` + `png` direkt: spart nur 168 KB, dafür fehlen Formaterkennung, Verkleinern und EXIF-Drehung.
  - `egui_extras`-Loader: +441 KB, ignoriert `SizeHint` und lädt immer in voller Auflösung, das führt zum Absturz bei großen Bildern.
  - GDI+ über `windows-sys`: +5,5 KB, aber alt, langsamer, mit groben Fehlercodes und ohne Verkleinern.

**Tray-Zeichen: Kontur mit `skrifa` 0.44 + `vello_cpu` 0.2 rastern (dieselben Crates, Versionen und Features wie `epaint` 0.37), aus den Systemschriften *Segoe UI Symbol* und dann *Segoe UI Emoji* (`%WINDIR%\Fonts`), zur Laufzeit gelesen.**
- **Nicht darstellbar** = keine der beiden `cmap` kennt das Zeichen (`charmap().map(c)` ist `None` oder 0) oder die Kontur ist leer. Dann gilt der Kreis.
- **Farbige Emoji werden einfarbig**, weil nur die `glyf`-Kontur gezeichnet wird und `COLR`/`CPAL` nicht beachtet werden. Beide Schriften haben Konturen für ihre Emoji. Geprüft mit `☢ ⌛ ★ ⏰ ✓ ❤ 🍅 🦀 🥑 🫠 🪿`, alle in 16 px erkennbar. U+F0000 wurde richtig als nicht darstellbar erkannt.
- Die Kontur wird auf ihren Umriss eingepasst und zentriert. Größe ist `GetSystemMetricsForDpi(SM_CXSMICON, dpi)` (16/20/24/32 px), wie es `NOTIFYICONDATAW` empfiehlt. `Icon::from_rgba` erwartet nicht vormultipliziertes RGBA. Die App rastert einmal pro Theme und Größe (2 bis 15 ms) und färbt danach nur für jede Phase und *gedimmt* um.
- Ein Zeichen ist ein Unicode-Skalar. Variantenselektoren U+FE0E/U+FE0F am Ende werden abgeschnitten. ZWJ-Folgen, Hautfarben, Flaggen und Tastenkappen gelten als nicht darstellbar (kein Shaping).
- Gemessen: **+61 KB, kein neues Crate.** Die Microsoft-Schriften dürfen Apps zum Zeichnen auf den Bildschirm nutzen, einbetten dürfen sie sie nicht.
- **Verworfen:**
  - GDI (+3 KB): `GetGlyphIndicesW` meldet jedes Zeichen außerhalb der BMP (also fast alle Emoji wie `🍅`) als fehlend.
  - DirectWrite: fachlich sauber, aber nicht in `windows-sys`.
  - `ab_glyph`: +110 KB und 4 Crates, gleiches Ergebnis. Es ist der Ersatz, falls die junge `vello_cpu`-API bricht.
  - Eingebettete freie Symbolschrift: Megabytes und weniger Zeichen.
  - Rastern über egui selbst: Die Rasterfunktion in `epaint` ist privat, Pixel aus dem Font-Atlas zurückzulesen ist umständlich.

**Risiken / nicht geprüft:** `skrifa`/`vello_cpu` müssen bei jedem egui-Update auf die Versionen von `epaint` nachgezogen werden, sonst landen sie doppelt im Baum. Nicht gemessen habe ich die Speicherspitze beim Dekodieren (berechnet 72 bis 96 MB kurzzeitig). Mit echten Dateien nicht geprüft habe ich EXIF-gedrehte JPEGs, CMYK-JPEGs und 16-bit- oder Paletten-PNGs, ebenso die Icongrößen 20/24 px und das Icon in der echten Taskleiste. ICC-Farbprofile wendet kein Weg an.

Details: [Recherche](../research/13-bilddecoder-und-tray-zeichen.md)
