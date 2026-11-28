//! Hintergrundbild dekodieren und Tray-Icon rastern (SPEC 7.5, 7.6).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use eframe::egui::{self, ColorImage, TextureFilter, TextureHandle, TextureOptions};

use crate::theme::Farbe;

/// Das Hintergrundbild des aktiven Themes. Dekodiert wird in einem kurzen Thread;
/// bis das Ergebnis da ist, gilt die Farbe `hintergrund`.
#[derive(Default)]
pub struct Hintergrund {
    /// Bild und Zielfläche, für die zuletzt dekodiert wurde
    auftrag: Option<(PathBuf, (u32, u32))>,
    empfang: Option<Receiver<Option<ColorImage>>>,
    textur: Option<TextureHandle>,
    /// Bild, das sich nicht laden ließ
    fehlgeschlagen: Option<PathBuf>,
}

impl Hintergrund {
    /// Startet das Dekodieren, wenn sich Bild oder Zielfläche geändert haben.
    /// `neu_laden` erzwingt es (Klick auf das aktive Theme).
    pub fn setzen(
        &mut self,
        ctx: &egui::Context,
        bild: Option<&Path>,
        flaeche: (u32, u32),
        neu_laden: bool,
    ) {
        let auftrag = bild.map(|b| (b.to_owned(), flaeche));
        if auftrag == self.auftrag && !neu_laden {
            return;
        }
        self.auftrag = auftrag.clone();
        self.textur = None;
        self.fehlgeschlagen = None;
        self.empfang = auftrag.map(|(pfad, flaeche)| {
            let (sender, empfang) = channel();
            let wecken = ctx.clone();
            std::thread::spawn(move || {
                let _ = sender.send(dekodieren(&pfad, flaeche));
                wecken.request_repaint();
            });
            empfang
        });
    }

    /// Holt ein fertig dekodiertes Bild ab. Liefert, ob sich etwas geändert hat.
    pub fn abholen(&mut self, ctx: &egui::Context) -> bool {
        let Some(ergebnis) = self.empfang.as_ref().and_then(|e| e.try_recv().ok()) else {
            return false;
        };
        self.empfang = None;
        match ergebnis {
            Some(bild) => {
                let optionen = TextureOptions {
                    mipmap_mode: Some(TextureFilter::Linear),
                    ..TextureOptions::LINEAR
                };
                self.textur = Some(ctx.load_texture("hintergrund", bild, optionen));
            }
            None => self.fehlgeschlagen = self.auftrag.as_ref().map(|(p, _)| p.clone()),
        }
        true
    }

    pub fn textur(&self) -> Option<&TextureHandle> {
        self.textur.as_ref()
    }

    pub fn fehlgeschlagen(&self) -> Option<&Path> {
        self.fehlgeschlagen.as_deref()
    }
}

/// Liest das Bild, dreht es nach EXIF und verkleinert es auf die Zielfläche. Jeder Fehler
/// ergibt `None` (Menütext „Bild fehlt“).
fn dekodieren(pfad: &Path, flaeche: (u32, u32)) -> Option<ColorImage> {
    use image::ImageDecoder as _;

    let mut decoder = image::ImageReader::open(pfad)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;
    let ausrichtung = decoder.orientation().ok()?;
    let mut bild = image::DynamicImage::from_decoder(decoder).ok()?;
    bild.apply_orientation(ausrichtung);
    if let Some((b, h)) = zielgroesse((bild.width(), bild.height()), flaeche) {
        bild = bild.thumbnail_exact(b, h);
    }
    let rgba = bild.into_rgba8();
    let groesse = [rgba.width() as usize, rgba.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(groesse, rgba.as_raw()))
}

/// Größe, auf die ein Bild verkleinert wird, damit es die Fläche gerade noch ganz bedeckt
/// (proportional, „füllen“). `None`: das Bild ist nicht größer, es bleibt, wie es ist.
pub fn zielgroesse(bild: (u32, u32), flaeche: (u32, u32)) -> Option<(u32, u32)> {
    let (bb, bh) = (f64::from(bild.0), f64::from(bild.1));
    let skala = (f64::from(flaeche.0) / bb).max(f64::from(flaeche.1) / bh);
    if skala >= 1.0 {
        return None;
    }
    let seite = |n: f64| ((n * skala).round() as u32).max(1);
    Some((seite(bb), seite(bh)))
}

/// Das eine Unicode-Zeichen eines `tray_zeichen` (SPEC 7.6). Variantenselektoren am Ende
/// fallen weg; leer oder mehr als ein Zeichen ergibt `None` (dann der Kreis).
pub fn zeichen_pruefen(text: &str) -> Option<char> {
    let mut zeichen = text.trim_end_matches(['\u{FE0E}', '\u{FE0F}']).chars();
    match (zeichen.next(), zeichen.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

/// Zeichnet das Tray-Icon. Die Maske (Kreis oder Zeichen) wird je Zeichen und Größe einmal
/// gerastert, danach nur umgefärbt.
pub struct Trayzeichner {
    /// Schriftdateien in Suchreihenfolge
    schriften: Vec<Vec<u8>>,
    /// zuletzt gerasterte Maske mit Zeichen und Größe
    maske: Option<(Option<char>, u32, Vec<u8>)>,
}

impl Trayzeichner {
    pub fn neu(schriften: Vec<Vec<u8>>) -> Self {
        Self {
            schriften,
            maske: None,
        }
    }

    /// RGBA, nicht vormultipliziert, `groesse` × `groesse`.
    pub fn icon(&mut self, tray_zeichen: Option<&str>, farbe: Farbe, groesse: u32) -> Vec<u8> {
        let zeichen = tray_zeichen.and_then(zeichen_pruefen);
        let aktuell = self
            .maske
            .as_ref()
            .is_some_and(|(z, g, _)| *z == zeichen && *g == groesse);
        if !aktuell {
            let maske = zeichen
                .and_then(|z| self.schriften.iter().find_map(|s| glyphe(s, z, groesse)))
                .unwrap_or_else(|| kreis(groesse));
            self.maske = Some((zeichen, groesse, maske));
        }
        let (_, _, maske) = self.maske.as_ref().expect("eben gesetzt");
        maske
            .iter()
            .flat_map(|&a| [farbe.r, farbe.g, farbe.b, a])
            .collect()
    }
}

/// *Segoe UI Symbol*, dann *Segoe UI Emoji* aus `%WINDIR%\Fonts`; fehlende fallen weg.
pub fn systemschriften() -> Vec<Vec<u8>> {
    let windows = std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into());
    let ordner = std::path::Path::new(&windows).join("Fonts");
    ["seguisym.ttf", "seguiemj.ttf"]
        .iter()
        .filter_map(|datei| std::fs::read(ordner.join(datei)).ok())
        .collect()
}

/// Die Kontur des Zeichens aus einer Schrift, auf ihren Umriss eingepasst und mit 1 px Rand
/// zentriert. `None`, wenn die Schrift das Zeichen nicht kennt oder die Kontur leer ist.
fn glyphe(schrift: &[u8], zeichen: char, groesse: u32) -> Option<Vec<u8>> {
    use skrifa::MetadataProvider as _;
    use vello_cpu::kurbo::{Affine, BezPath, Shape as _};

    let font = skrifa::FontRef::new(schrift).ok()?;
    let id = font.charmap().map(zeichen)?;
    let kontur = font.outline_glyphs().get(id)?;
    let mut pfad = BezPath::new();
    let einstellungen = skrifa::outline::DrawSettings::unhinted(
        skrifa::instance::Size::unscaled(),
        skrifa::instance::LocationRef::default(),
    );
    kontur.draw(einstellungen, &mut Stift(&mut pfad)).ok()?;
    let umriss = pfad.bounding_box();
    if umriss.width() <= 0.0 || umriss.height() <= 0.0 {
        return None;
    }
    let n = f64::from(groesse);
    let innen = n - 2.0;
    let skala = (innen / umriss.width()).min(innen / umriss.height());
    let versatz = (
        (n - umriss.width() * skala) / 2.0,
        (n - umriss.height() * skala) / 2.0,
    );
    let seite = u16::try_from(groesse).ok()?;
    let mut ctx = vello_cpu::RenderContext::new(seite, seite);
    ctx.set_transform(
        Affine::translate(versatz)
            * Affine::scale(skala)
            * Affine::translate((-umriss.x0, -umriss.y0)),
    );
    ctx.set_paint(vello_cpu::color::OpaqueColor::<vello_cpu::color::Srgb>::WHITE);
    ctx.fill_path(&pfad);
    let mut bild = vello_cpu::Pixmap::new(seite, seite);
    ctx.render(&mut bild, &mut vello_cpu::Resources::new());
    let maske: Vec<u8> = bild
        .data_as_u8_slice()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|px| px[3])
        .collect();
    maske.iter().any(|&a| a > 0).then_some(maske)
}

/// Sammelt die Kontur in einen Pfad; y zeigt in Schriften nach oben, im Bild nach unten.
struct Stift<'a>(&'a mut vello_cpu::kurbo::BezPath);

impl skrifa::outline::OutlinePen for Stift<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((f64::from(x), -f64::from(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((f64::from(x), -f64::from(y)));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0.quad_to(
            (f64::from(cx0), -f64::from(cy0)),
            (f64::from(x), -f64::from(y)),
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.curve_to(
            (f64::from(cx0), -f64::from(cy0)),
            (f64::from(cx1), -f64::from(cy1)),
            (f64::from(x), -f64::from(y)),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

/// Gefüllter Kreis mit 1 px Rand, geglättet über die Abdeckung je Pixel.
fn kreis(groesse: u32) -> Vec<u8> {
    let mitte = f64::from(groesse) / 2.0;
    let radius = mitte - 1.0;
    (0..groesse * groesse)
        .map(|i| {
            let x = f64::from(i % groesse) + 0.5 - mitte;
            let y = f64::from(i / groesse) + 0.5 - mitte;
            let abdeckung = (radius + 0.5 - x.hypot(y)).clamp(0.0, 1.0);
            (abdeckung * 255.0).round() as u8
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROT: Farbe = Farbe::rgb(0xE5, 0x53, 0x4B);

    fn alpha(icon: &[u8], groesse: u32, x: u32, y: u32) -> u8 {
        icon[((y * groesse + x) * 4 + 3) as usize]
    }

    fn pixel(icon: &[u8], groesse: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * groesse + x) * 4) as usize;
        icon[i..i + 4].try_into().unwrap()
    }

    #[test]
    fn ohne_zeichen_ein_kreis_in_der_farbe_mit_rand() {
        let mut z = Trayzeichner::neu(Vec::new());
        let n = 16;
        let icon = z.icon(None, ROT, n);
        assert_eq!(icon.len(), 16 * 16 * 4);
        assert_eq!(pixel(&icon, n, 8, 8), [0xE5, 0x53, 0x4B, 255]);
        for (x, y) in [(0, 0), (15, 0), (0, 15), (15, 15), (1, 1)] {
            assert_eq!(alpha(&icon, n, x, y), 0, "Ecke {x},{y}");
        }
        // 1 px Rand bleibt frei, der Kreis reicht bis daran
        for i in 0..n {
            for (x, y) in [(i, 0), (i, n - 1), (0, i), (n - 1, i)] {
                assert_eq!(alpha(&icon, n, x, y), 0, "Rand {x},{y}");
            }
        }
        assert!(alpha(&icon, n, 1, 8) > 0);
        assert!(alpha(&icon, n, 8, 14) > 0);
    }

    /// Kleinstes Rechteck mit sichtbaren Pixeln: (links, oben, rechts, unten), einschließlich.
    fn umriss(icon: &[u8], n: u32) -> (u32, u32, u32, u32) {
        let sichtbar: Vec<(u32, u32)> = (0..n * n)
            .filter(|i| icon[(i * 4 + 3) as usize] > 0)
            .map(|i| (i % n, i / n))
            .collect();
        assert!(!sichtbar.is_empty(), "Icon ist leer");
        let xs = sichtbar.iter().map(|p| p.0);
        let ys = sichtbar.iter().map(|p| p.1);
        (
            xs.clone().min().unwrap(),
            ys.clone().min().unwrap(),
            xs.max().unwrap(),
            ys.max().unwrap(),
        )
    }

    #[test]
    fn zeichen_aus_der_systemschrift_eingepasst_und_zentriert() {
        let mut z = Trayzeichner::neu(systemschriften());
        let n = 24;
        let icon = z.icon(Some("☢"), ROT, n);
        assert!(icon != z.icon(None, ROT, n), "kein Rückfall auf den Kreis");
        let (l, o, r, u) = umriss(&icon, n);
        // auf den Umriss eingepasst: die längere Seite füllt bis auf 1 px Rand
        assert!(r - l >= n - 4 || u - o >= n - 4, "Umriss {l},{o}–{r},{u}");
        assert!(l >= 1 && o >= 1 && r <= n - 2 && u <= n - 2, "Rand frei");
        // zentriert
        assert!(
            (l as i32 - (n - 1 - r) as i32).abs() <= 1,
            "waagrecht {l} / {r}"
        );
        assert!(
            (o as i32 - (n - 1 - u) as i32).abs() <= 1,
            "senkrecht {o} / {u}"
        );
        // einfarbig in der Farbe
        let mitte = pixel(&icon, n, 12, 12);
        assert_eq!(mitte[..3], [0xE5, 0x53, 0x4B]);
    }

    #[test]
    fn nicht_darstellbare_zeichen_ergeben_den_kreis() {
        let mut z = Trayzeichner::neu(systemschriften());
        let kreis = z.icon(None, ROT, 20);
        for zeichen in ["\u{F0000}", "", "👨\u{200D}💻", "👍🏽"] {
            assert!(z.icon(Some(zeichen), ROT, 20) == kreis, "{zeichen:?}");
        }
    }

    #[test]
    fn emoji_kommt_aus_der_zweiten_schrift_einfarbig() {
        let mut z = Trayzeichner::neu(systemschriften());
        let icon = z.icon(Some("🍅"), ROT, 32);
        assert!(icon != z.icon(None, ROT, 32), "kein Rückfall auf den Kreis");
        assert!(
            icon.as_chunks::<4>()
                .0
                .iter()
                .all(|px| px[..3] == [0xE5, 0x53, 0x4B])
        );
    }

    #[test]
    fn grosses_foto_wird_auf_die_flaeche_verkleinert() {
        // 300 % von 180 × 90 bei 100 % DPI
        assert_eq!(zielgroesse((4000, 3000), (540, 270)), Some((540, 405)));
    }

    #[test]
    fn hochkant_bestimmt_die_breite() {
        assert_eq!(zielgroesse((3000, 4000), (540, 270)), Some((540, 720)));
    }

    #[test]
    fn kleines_bild_wird_nie_vergroessert() {
        assert_eq!(zielgroesse((200, 100), (540, 270)), None);
        assert_eq!(zielgroesse((540, 270), (540, 270)), None);
    }

    #[test]
    fn breit_genug_aber_zu_hoch_wird_trotzdem_nicht_verkleinert() {
        // die Breite füllt gerade, kleiner ginge nur mit Lücke
        assert_eq!(zielgroesse((540, 2000), (540, 270)), None);
    }

    #[test]
    fn ein_zeichen_mit_variantenselektor_bleibt_ein_zeichen() {
        assert_eq!(zeichen_pruefen("☢"), Some('☢'));
        assert_eq!(zeichen_pruefen("☢\u{FE0F}"), Some('☢'));
        assert_eq!(zeichen_pruefen("🍅\u{FE0E}"), Some('🍅'));
    }

    #[test]
    fn leeres_feld_und_zusammengesetzte_zeichen_sind_nicht_darstellbar() {
        assert_eq!(zeichen_pruefen(""), None);
        assert_eq!(zeichen_pruefen("\u{FE0F}"), None);
        // ZWJ-Folge, Hautfarbe, Flagge, Tastenkappe
        assert_eq!(zeichen_pruefen("👨\u{200D}💻"), None);
        assert_eq!(zeichen_pruefen("👍🏽"), None);
        assert_eq!(zeichen_pruefen("🇩🇪"), None);
        assert_eq!(zeichen_pruefen("1\u{FE0F}\u{20E3}"), None);
    }
}
