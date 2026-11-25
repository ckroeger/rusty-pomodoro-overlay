# Recherche: Bilddecoder und Tray-Zeichen rendern

Ticket: [13 – Bilddecoder und Tray-Zeichen rendern](../tickets/13-bilddecoder-und-tray-zeichen.md) · Stand: 2026-11-25

## Frage

1. **Hintergrundbild:** Womit dekodiert die App (eframe/egui 0.37 + glow, `windows-sys` 0.61, nur Windows 11) zur Laufzeit ein vom Nutzer gewähltes JPG oder PNG (mit Alphakanal) und zeigt es als egui-Textur, gefüllt, mittig beschnitten und mit einer Deckkraft multipliziert? Bewertet nach Binärgröße, Aufwand und Verhalten bei großen Bildern.
2. **Tray-Zeichen:** Wie zeichnet die App ein beliebiges Unicode-Zeichen (`☢`, `⌛`, auch `🍅`) einfarbig in einer gegebenen Farbe in ein RGBA-Icon für `tray-icon`? Wie erkennt sie ein nicht darstellbares Zeichen, und lassen sich farbige Emoji einfarbig zeichnen?

## Kurzantwort

**Bild: `image` 0.25 mit nur `jpeg` + `png`, vor dem Hochladen auf Cover-Größe verkleinert (`thumbnail_exact`), EXIF-Drehung angewendet, Dekodieren in einem kurzen Hintergrund-Thread.**
- `image` 0.25.10 mit `png` steckt schon im Baum (eframe braucht es für das Fenster-Icon). Neu kommen nur `zune-jpeg` und `zune-core` dazu. Gemessen: **+336 KB** (4,69 → 5,04 MB).
- WIC wäre mit +9,5 KB viel kleiner, geht aber **nicht über `windows-sys`** (keine COM-Schnittstellen), sondern braucht das `windows`-Crate (10 Crates mehr, darunter Proc-Macros) und `unsafe`-COM-Code. Die EXIF-Drehung müsste man selbst auslesen. Es ist die Alternative, falls die Größe doch wichtiger wird.
- Herunterskalieren ist **Pflicht, nicht Kür**: `egui_glow` bricht mit `assert!` ab, wenn eine Textur größer als `GL_MAX_TEXTURE_SIZE` ist. Mit `panic = "abort"` beendet das die App. Ein 6000 × 4000-Foto würde außerdem 96 MB RGBA belegen.

**Tray-Zeichen: Systemschriften zur Laufzeit aus `%WINDIR%\Fonts` lesen (*Segoe UI Symbol*, dann *Segoe UI Emoji*) und die Kontur mit `skrifa` + `vello_cpu` rastern, also den Crates, mit denen egui 0.37 selbst Text rastert.**
- Kein neues Crate, gemessen **+61 KB**. Pro Zeichen und Größe 2 bis 3 ms, mit Nachladen von *Segoe UI Emoji* (12,9 MB) 10 bis 15 ms.
- **Nicht darstellbar** heißt: Die `cmap` keiner der beiden Schriften kennt das Zeichen (`charmap().map(c)` liefert `None` oder Glyphe 0) oder die Kontur ist leer. Dann gilt der Kreis.
- **Farbige Emoji werden einfarbig**, weil nur die Kontur (`glyf`) gezeichnet und die Farbebene (`COLR`/`CPAL`) nicht beachtet wird. Beide Schriften haben für ihre Emoji Konturen. Probiert mit `🍅`, `🦀`, `❤`, `🫠`, `🪿`, `🥑`: alle erkennbar.
- GDI (`windows-sys`, +3 KB) scheidet aus: `GetGlyphIndicesW` meldet **jedes Zeichen außerhalb der BMP** (U+10000 und höher, also fast alle Emoji wie `🍅`) als nicht vorhanden.

## Geprüfte Versionen

| Crate | Version | Bemerkung |
|---|---|---|
| `eframe` / `egui` / `egui_glow` / `epaint` | 0.37.0 | `eframe` hängt an `image` 0.25.10 mit `default-features = false, features = ["png"]` (`Cargo.toml` von eframe); `epaint` rastert Text mit `skrifa` 0.44 + `vello_cpu` 0.2 (`src/text/font_face.rs`) |
| `image` | 0.25.10 | Feature `jpeg` = `zune-jpeg` 0.5 + `zune-core` 0.5; `moxcms` und `png` 0.18 sind immer dabei |
| `egui_extras` | 0.37.0 | Feature `image` + `file` |
| `windows-sys` | 0.61.2 | hat `Win32/Graphics/{Gdi, GdiPlus, Dwm, …}`, aber **kein** `Imaging` (WIC) und kein `DirectWrite` (geprüft im Quellbaum des Crates) |
| `windows` | 0.62.2 | für WIC (`Win32_Graphics_Imaging`, `Win32_System_Com`) |
| `skrifa` / `vello_cpu` | 0.44.0 / 0.2.0 | direkt, mit denselben Features wie `epaint` (`std`; bei `vello_cpu` `std` + `u8_pipeline`, ohne Default-Features) |
| `ab_glyph` | 0.2.32 | zieht `ttf-parser` 0.25, `owned_ttf_parser`, `ab_glyph_rasterizer` |
| `tray-icon` | 0.26.1 | `Icon::from_rgba(rgba, w, h)` → intern `CreateIcon` mit BGRA, also **nicht vormultipliziertes** RGBA (`src/platform_impl/windows/icon.rs`) |

## Messaufbau

Kopie des Spikes von `prototype/02-overlay-spike` im Temp-Ordner (nicht im Repo), umgestellt auf `windows-sys` 0.61. Release, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`. Je Kandidat ein Cargo-Feature, das echten Code einbindet: Das Overlay lädt beim Start das Bild aus der Umgebungsvariable `BG` als Textur und zeichnet es gefüllt und beschnitten mit der Deckkraft des Hintergrund-Reglers. Das Tray-Zeichen aus `GLYPH` wird gerendert und als Fenster-Icon gesetzt, damit der Linker nichts wegwirft. Ein zweites Programm `bench` misst Zeiten und schreibt die gerenderten Icons heraus.

Testbilder (Python/PIL): `gross.jpg` 6000 × 4000 (1,6 MB, Qualität 90), `gross.png` 6000 × 4000 RGBA mit radialem Alpha (1,05 MB), `klein.jpg`/`klein.png` 1200 × 600, ein progressives JPEG, eine kaputte Datei (`FF D8 FF E0` + Müll), eine fehlende Datei. Zielgröße der Verkleinerung: Cover für 1800 × 900 Pixel.

## Teil 1: Hintergrundbild

### Binärgröße

| Variante | Exe | Δ zur Basis | neue Crates |
|---|---|---|---|
| Basis (Spike, `windows-sys` 0.61) | 4.694.528 B | – | – |
| `image` 0.25 `jpeg` + `png`, `thumbnail_exact` + `apply_orientation` | 5.038.592 B | **+336 KB** | `zune-jpeg`, `zune-core` |
| ebenso, mit `resize_exact(…, Triangle)` | 5.030.912 B | +328 KB | ebenso |
| `zune-jpeg` + `png` direkt, eigener Box-Filter | 4.866.560 B | +168 KB | `zune-jpeg`, `zune-core` |
| `egui_extras` 0.37 Loader (`image` + `file`) | 5.146.624 B | +441 KB | `egui_extras`, `zune-*`, `mime`, `mime_guess2`, `enum-map`, `unicase` |
| WIC über `windows` 0.62, mit `IWICBitmapScaler` | 4.704.256 B | **+9,5 KB** | `windows`, `windows-core`, `-implement`, `-interface`, `-result`, `-strings`, `-numerics`, `-collections`, `-future`, `-threading`, `syn` |
| GDI+ (flache API) über `windows-sys` 0.61, eigener Box-Filter | 4.700.160 B | +5,5 KB | – |

### Dauer: Dekodieren + Verkleinern (jeweils zwei Läufe, ms)

| Datei | `image` (`thumbnail_exact`) | `image` (Triangle) | `zune` + `png` | WIC | GDI+ |
|---|---|---|---|---|---|
| `gross.jpg` 6000 × 4000 | 197 / 189 | 320 / 340 | 149 / 148 | **69 / 69** | 203 / 203 |
| `gross.png` 6000 × 4000 | 118 / 117 | 277 / 278 | 130 / 135 | 97 / 101 | 181 / 184 |
| `klein.jpg` 1200 × 600 | – | 12 / 5 | 9 / 4 | 5 / 7 | 12 / 11 |
| `klein.png` 1200 × 600 | – | 7 / 5 | 5 / 5 | 5 / 6 | 10 / 12 |
| progressives JPEG | – | 4 / 1 | 2 / 4 | 2 / 2 | 6 / 6 |
| kaputt / fehlt | Fehler, ~2 ms | ebenso | ebenso | Fehler, ~1–3 ms | Fehler (`Status 2` für beides) |

Alle Wege lieferten bei kaputter oder fehlender Datei einen Fehler und keinen Absturz. Das reicht für „(Bild fehlt)“ aus *Theming*. WIC ist beim großen JPEG am schnellsten, weil der Skalierer die Verkleinerung bis in den JPEG-Decoder durchreicht. Die übrigen Wege dekodieren erst die volle Größe (6000 × 4000 × 4 B = 96 MB zwischendurch, nicht gemessen).

### Bewertung der Kandidaten

- **`image` (`jpeg` + `png`)**: etwa 15 Zeilen sicherer Code. `ImageReader::open(p)?.with_guessed_format()?.into_decoder()?`, dann `decoder.orientation()` und `DynamicImage::apply_orientation` (EXIF-Drehung von Handyfotos, so macht es auch `egui_extras::image::load_image_bytes`). `ImageReader` hat ab Werk `Limits { max_alloc: Some(512 MiB) }` (`src/io/limits.rs`) und bricht riesige oder präparierte Dateien mit einem Fehler ab, statt den Speicher zu sprengen. Das Format erkennt es am Inhalt, nicht an der Endung. `png` und `image` sind schon im Baum. Nachteil: +336 KB, also ~7 %.
- **`zune-jpeg` + `png` direkt**: spart gegenüber `image` nur 168 KB, dafür muss man selbst unterscheiden (Magic Bytes), Farbtypen umwandeln, verkleinern und die EXIF-Drehung auslesen (`zune-jpeg` liefert nur die rohen EXIF-Bytes). Lohnt nicht.
- **`egui_extras`-Loader**: am größten (+441 KB). Der Bild-Loader ignoriert die Größenangabe (`fn load(&self, ctx, uri, _: SizeHint)` in `src/loaders/image_loader.rs`) und lädt das Bild immer in voller Auflösung als Textur. Ein Bild über `GL_MAX_TEXTURE_SIZE` bringt damit die App zum Absturz (siehe unten). Er dekodiert zwar im Hintergrund-Thread (`background_decode.rs`), cacht aber pro URI. Beim erneuten Laden des Themes müsste man `forget` aufrufen. Scheidet aus.
- **WIC (`windows`-Crate)**: kleinster Zuwachs und schnellster Decoder, mit eingebautem Skalierer (`IWICBitmapScaler`, `WICBitmapInterpolationModeFant`) und Umwandlung nach `GUID_WICPixelFormat32bppRGBA`. Dagegen spricht: `windows-sys` hat keine COM-Schnittstellen, also braucht es ein zweites Windows-Crate mit Proc-Macros (längere Kompilierzeit), etwa 40 Zeilen `unsafe` mit `CoInitializeEx`/`CoCreateInstance`, und die EXIF-Drehung wendet WIC nicht selbst an (Metadaten-Abfrage + `IWICBitmapFlipRotator`). Fehlermeldungen kommen als HRESULT-Text, z. B. „Der Imageheader ist unbekannt. (0x88982F61)“.
- **GDI+ über `windows-sys`**: als Notlösung ohne neues Crate (+5,5 KB): `GdiplusStartup`, `GdipCreateBitmapFromFile`, `GdipBitmapLockBits` (32bpp ARGB, BGRA → RGBA tauschen). Langsamer als `image` mit `thumbnail_exact`, Fehler nur als grober `Status`, Skalieren und EXIF-Drehung müsste man selbst machen. Nicht empfohlen, aber gut zu wissen.

### Große Bilder: Herunterskalieren ist nötig

- `egui::Context::load_texture` prüft die Größe nur mit `debug_assert!`. `egui_glow` prüft beim Hochladen mit **`assert!`** („Got a texture image of size {}x{}, but the maximum supported texture side is only {}“, `src/painter.rs` Z. 552). Der Wert kommt aus `GL_MAX_TEXTURE_SIZE` (typisch 16384). Mit `panic = "abort"` beendet ein Panorama mit 20000 px Breite also die App.
- **Ziel der Verkleinerung:** Das Bild wird so weit verkleinert, dass es die größte mögliche Overlay-Fläche gerade noch füllt, also die physische Größe des Overlays bei 300 % Zoom (*Inhalt und Verhalten des Overlays*) mal Skalierungsfaktor des Monitors. Nur wenn das Bild größer ist, wird verkleinert, nie vergrößert. Bei 2:1 und etwa 1800 × 900 px sind das ~6,5 MB Textur statt 96 MB.
- **Kleinere Zoomstufen** übernimmt die GPU. Damit feine Motive beim Verkleinern um den Faktor 3 nicht flimmern, wird die Textur mit `TextureOptions { mipmap_mode: Some(TextureFilter::Linear), ..TextureOptions::LINEAR }` angelegt. `egui_glow` erzeugt dann Mipmaps (`generate_mipmap`, `src/painter.rs` Z. 632).
- `thumbnail_exact` (schnelle Flächenmittelung) war doppelt so schnell wie `resize_exact` mit `Triangle` und reicht für einen Hintergrund.

### Zeichnen: füllen, beschneiden, Deckkraft

- `egui::ColorImage::from_rgba_unmultiplied` wandelt in vormultipliziertes Alpha um. Der Alphakanal eines PNG bleibt also erhalten.
- Mittiges Beschneiden geht über das UV-Rechteck statt über Pixel: Ist das Bild breiter als die Fläche (`bild_seitenverhältnis > fläche_seitenverhältnis`), gilt `u = fläche/bild`, UV = `[(1−u)/2, 0] … [(1+u)/2, 1]`, sonst entsprechend für `v`. Gezeichnet wird mit `painter.image(tex_id, rect, uv, Color32::from_white_alpha(deckkraft))`. Weil `Color32` vormultipliziert ist, multipliziert dieser Farbton alle vier Kanäle mit der Deckkraft, also genau „Deckkraft × Alpha des Bildes“.
- Im Messprogramm funktionierte das mit dem 6000 × 4000-PNG auf einem 300 × 230-Overlay: Das Bild war beschnitten, die Ränder mit Alpha 0 ließen den Desktop durch (Screenshot, nicht im Repo).

### Wann dekodieren

0,1 bis 0,2 s für ein großes Foto fallen beim Start und beim Wechsel des Themes an. Damit das Overlay nicht einfriert, dekodiert ein kurzlebiger `std::thread::spawn` und schickt das fertige `ColorImage` per Kanal zurück, gefolgt von `ctx.request_repaint()`. Bis dahin gilt die Farbe `hintergrund`. Bei einem Fehler wird es „(Bild fehlt)“. Das Original wird nicht aufgehoben. Ändert sich der Skalierungsfaktor (anderer Monitor), wird die Datei neu gelesen.

## Teil 2: Tray-Zeichen

### Binärgröße

| Variante | Exe | Δ zur Basis | neue Crates |
|---|---|---|---|
| `skrifa` + `vello_cpu` (wie `epaint`), Systemschriften | 4.757.504 B | **+61 KB** | – |
| GDI (`windows-sys`, `Win32_Graphics_Gdi`), `DrawTextW` in DIB | 4.697.600 B | +3 KB | – |
| `ab_glyph` 0.2, Systemschriften | 4.807.680 B | +110 KB | `ab_glyph`, `ab_glyph_rasterizer`, `owned_ttf_parser`, `ttf-parser` |

### Probe mit zehn Zeichen (16 und 32 px)

| Zeichen | `skrifa` + `vello_cpu` | GDI | `ab_glyph` |
|---|---|---|---|
| `☢` U+2622, `⌛` U+231B, `★` U+2605, `⏰` U+23F0, `✓` U+2713, `A`, `❤` U+2764 | *Segoe UI Symbol*, 2–5 ms | *Segoe UI Symbol*, 1–12 ms | *Segoe UI Symbol*, 2–4 ms |
| `🍅` U+1F345, `🦀` U+1F980, `🥑` U+1F951 | *Segoe UI Symbol*, ~2 ms | **„nicht darstellbar“** | *Segoe UI Symbol*, ~2 ms |
| `🫠` U+1FAE0, `🪿` U+1FABF (neuere Emoji) | *Segoe UI Emoji*, 10–15 ms | – (nicht geprüft, s. o.) | *Segoe UI Emoji*, 9–15 ms |
| U+F0000 (Private Use) | nicht darstellbar, 8–14 ms | nicht darstellbar | nicht darstellbar |

Bei allen drei Wegen sind die Bilder in 16 px gut erkennbar (vergrößerte Kontaktabzüge, nicht im Repo). `skrifa`/`ab_glyph` füllen die Fläche, weil sie auf den Umriss der Glyphe einpassen. GDI zeichnet in der Em-Höhe und damit kleiner. Emoji erscheinen als einfarbige Silhouetten mit ausgesparten Innenlinien, z. B. Tomate mit Stielansatz und Krebs mit Scheren.

### Warum GDI ausscheidet

- [`GetGlyphIndicesW`](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getglyphindicesw) mit `GGI_MARK_NONEXISTING_GLYPHS` „Marks unsupported glyphs with the hexadecimal value 0xffff“. Die Funktion arbeitet aber pro UTF-16-Einheit („attempts to identify a single-glyph representation for each character“) und verweist für alles Weitere auf Uniscribe. Ein Ersatzpaar (Surrogate) wie `🍅` ergab im Versuch zweimal 0xFFFF, obwohl *Segoe UI Symbol* die Glyphe hat. Für eine verlässliche Erkennung bräuchte es Uniscribe (`ScriptItemize`/`ScriptShape`) oder DirectWrite.
- DirectWrite (`IDWriteFontFallback::MapCharacters`, Graustufen über `IDWriteGlyphRunAnalysis`) wäre fachlich sauber, ist in `windows-sys` aber nicht enthalten (COM). Es bräuchte wie WIC das `windows`-Crate. Für ein einziges Zeichen lohnt das nicht.

### Warum `skrifa` + `vello_cpu`

- **Schon im Baum:** `epaint` 0.37 nutzt genau diese beiden Crates, um Glyphen zu rastern: `skrifa::FontRef`, `charmap`, `outline_glyphs().get(gid)`, `outline.draw(DrawSettings::…, &mut pen)` in einen `kurbo::BezPath`, dann `vello_cpu::RenderContext::fill_path` in eine `Pixmap` (`src/text/font_face.rs`, `rasterize_glyph`). Diese Funktion ist privat. Der Weg über egui selbst (Text in den Font-Atlas, dann Pixel zurücklesen) ist umständlich und hängt an Interna. Deshalb nimmt die App dieselben Crates direkt, mit denselben Versionen und Features, damit Cargo keine Duplikate baut.
- **Schriften vom System, nicht eingebettet:** *Segoe UI Symbol* (`seguisym.ttf`, 2,5 MB) und *Segoe UI Emoji* (`seguiemj.ttf`, 12,9 MB) liegen auf jedem Windows 11 unter `%WINDIR%\Fonts`. Die App liest sie nur beim Wechsel des Themes. Laut [Font redistribution FAQ](https://learn.microsoft.com/en-us/typography/fonts/font-faq) darf jede Windows-App sie benutzen („Any application installed on your Windows computer has access to these fonts. A Windows application can use the fonts to render content to a screen“, ausdrücklich auch „the symbol and emoji fonts“). Einbetten ist dagegen verboten („you may not redistribute the Windows fonts … you may not convert them to other formats, including bitmap formats“; das Einbetten in Dokumente erlaubt „not embedding fonts in games, apps and devices“). Eine freie Ersatzschrift (z. B. Noto Sans Symbols 2 oder Noto Emoji) würde die Exe um Megabytes vergrößern und deckt nicht mehr ab.
- **Erkennen:** Erst *Segoe UI Symbol*, dann *Segoe UI Emoji*. Für jede Schrift: `font.charmap().map(c)` liefert `Some(gid)` mit `gid ≠ 0`, die Kontur existiert und ist nicht leer. Sonst die nächste Schrift. Findet sich nichts, wird der Kreis gezeichnet. Das ist eine reine `cmap`-Abfrage ohne Heuristik.
- **Einfarbig:** Gezeichnet wird nur die Kontur (`glyf`) mit Deckung als Alpha, die Farbebenen (`COLR`/`CPAL`) werden nicht gelesen. Die App färbt die Deckung dann ein (`[r, g, b, deckung]`). *Segoe UI Emoji* hat für jede Emoji-Glyphe auch eine einfarbige Kontur, sonst könnte GDI sie nicht schwarz-weiß zeichnen. Die Probe mit `🫠` und `🪿` bestätigt das. Für den Fall einer Glyphe ohne Kontur greift die Prüfung „Kontur leer“.
- **Einpassen:** Die Kontur wird in Schrifteinheiten gezeichnet (`Size::unscaled()`), ihr Umriss (`bounding_box`) wird proportional auf die Icongröße skaliert und zentriert. So füllen schmale (`⌛`) und breite Zeichen (`🦀`) die Fläche gleich gut. Hinting gibt es dabei nicht, bei 16 px sehen die Ränder etwas weicher aus als bei GDI.

### Was nicht geht (und so in die Spec gehört)

- **Ein Zeichen = ein Unicode-Skalar.** Ohne Shaping (GSUB) gibt es keine Sequenzen: Emoji mit Hautfarbe, ZWJ-Folgen (`👩‍💻`), Flaggen und Tastenkappen ergeben nur ihr erstes Zeichen oder gar nichts. Vorschlag: Variantenselektoren U+FE0E/U+FE0F am Ende abschneiden (`❤️` → `❤`), alles andere mit mehr als einem Skalar gilt als nicht darstellbar → Kreis.
- Zeichen, die nur in anderen Systemschriften stecken (CJK, Segoe Fluent Icons, Private Use), werden nicht gefunden. Wer das will, müsste die Schriftliste erweitern. Für v1 reichen Symbol und Emoji.

### Icongröße und Farbe

- Laut [`NOTIFYICONDATAW`](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw) wird ein 16 × 16-Icon „scaled to a larger size in a system set to a high dpi value. This can lead to an unattractive result.“ Microsoft empfiehlt `LoadIconMetric` mit `LIM_SMALL`, also die Größe `SM_CXSMICON`. Weil die App selbst rendert, nimmt sie die Größe `GetSystemMetricsForDpi(SM_CXSMICON, dpi)`: 16 px bei 100 %, 20 bei 125 %, 24 bei 150 %, 32 bei 200 %. Gemessen habe ich nur 16 und 32.
- `tray-icon::Icon::from_rgba` erwartet nicht vormultipliziertes RGBA. Die App rastert die Deckung einmal pro Theme und Größe und färbt sie für jede Phase und *gedimmt* neu ein (Mikrosekunden). Gerastert wird also nicht bei jedem Phasenwechsel.

## Empfehlung

1. **Hintergrundbild:** `image = { version = "0.25", default-features = false, features = ["jpeg", "png"] }` (dieselbe Version wie eframe). Ablauf in einem kurzen Hintergrund-Thread: `ImageReader::open` → `with_guessed_format` → `into_decoder` → `orientation` → `DynamicImage::from_decoder` → `apply_orientation` → bei Bedarf `thumbnail_exact` auf Cover der Overlay-Fläche bei 300 % mal DPI-Faktor → `into_rgba8`. Als Textur mit Mipmaps hochladen, mit UV-Beschnitt und `Color32::from_white_alpha(deckkraft)` zeichnen. Jeder Fehler gilt als „(Bild fehlt)“. Kosten +336 KB.
   - Ist die Größe wichtiger als der zusätzliche Code und das zweite Windows-Crate: WIC über `windows` (+9,5 KB, schnellster Weg), dann mit eigener Auswertung der EXIF-Drehung.
2. **Tray-Zeichen:** `skrifa` 0.44 + `vello_cpu` 0.2 direkt (gleiche Versionen und Features wie `epaint` 0.37), Schriften `seguisym.ttf` und dann `seguiemj.ttf` aus `%WINDIR%\Fonts` zur Laufzeit. Nicht darstellbar = in keiner `cmap` oder leere Kontur → Kreis. Nur die Kontur zeichnen (einfarbig, auch bei Emoji), auf den Umriss einpassen, Größe `SM_CXSMICON` zur aktuellen DPI. Kosten +61 KB, kein neues Crate.
   - Ersatz, falls die `vello_cpu`-API beim nächsten egui-Update bricht: `ab_glyph` (+110 KB, gleiches Verhalten in der Probe).

## Nicht verifiziert

- Speicherspitze beim Dekodieren eines 6000 × 4000-Fotos. Berechnet sind 72 bis 96 MB kurzzeitig, gemessen habe ich das nicht.
- JPEGs mit EXIF-Drehung, CMYK-JPEGs und PNGs mit 16 bit oder Palette mit Transparenz habe ich nicht mit echten Dateien geprüft. `image` behandelt sie laut Code (`apply_orientation`, `into_rgba8`), WIC/GDI+ nicht ausprobiert.
- Farbprofile (ICC) werden bei keinem Weg angewendet. Bilder mit Adobe-RGB-Profil wirken dadurch etwas blass.
- Das Tray-Icon selbst habe ich nicht in der Taskleiste angesehen, nur die gerenderten RGBA-Puffer (vergrößert). Größen 20 und 24 px (125 %/150 %) wurden nicht gerendert.
- Ob alle Emoji in *Segoe UI Emoji* eine nicht leere Kontur haben, ist nur stichprobenhaft geprüft.
- Das Messprojekt nutzte `windows-sys` 0.61 statt 0.59 wie der Spike, deshalb liegt die Basis bei 4,69 statt 4,68 MB.
