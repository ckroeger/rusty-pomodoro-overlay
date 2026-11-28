//! Das Overlay in egui (SPEC 4.2, 4.3, 5.2): zeichnet die fertige `Anzeige` und übersetzt
//! Maus und Tasten in Eingaben. Entscheidet selbst nichts über den Ablauf.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Pos2, Rect, Sense, Stroke,
    TextureHandle, Vec2, ViewportCommand, pos2, vec2,
};

use crate::anzeige::{Anzeige, Punkt, Symbol};
use crate::menue::Aktion;
use crate::theme::Farbe;

/// Grundgröße bei 100 % in logischen Pixeln (Richtwert).
pub const GRUNDGROESSE: Vec2 = vec2(180.0, 90.0);
/// Die Ecken rundet Windows (siehe `plattform::als_werkzeugfenster`); der Balken hält
/// trotzdem Abstand zu den Ecken.
const ECKENABSTAND: f32 = 10.0;
const ZEIT_GROESSE: f32 = 40.0;
const WORT_GROESSE: f32 = 13.0;
const BALKEN_HOEHE: f32 = 3.0;
/// Ab dieser Bewegung wird aus einem Klick ein Ziehen (logische Pixel, Richtwert).
const ZIEHSCHWELLE: f32 = 4.0;

/// Was das Overlay aus Maus und Tasten gemacht hat.
#[derive(Debug, Clone, PartialEq)]
pub enum Eingabe {
    Aktion(Aktion),
    /// Rastungen des Mausrads mit `Strg`, positiv = größer
    Skalieren(i32),
    /// Das Fenster wird gerade vom System verschoben.
    ZiehenBegonnen,
    /// Menü öffnen; `None` an der Mausposition, sonst an dieser Stelle im Overlay
    MenueOeffnen(Option<Pos2>),
}

/// Was zwischen zwei Bildern über die Maus gemerkt werden muss.
#[derive(Debug, Default)]
pub struct Zeiger {
    gedrueckt_bei: Option<Pos2>,
    zieht: bool,
    /// Bruchteile von Rastungen (Präzisions-Mausräder)
    rad_rest: f32,
}

pub fn farbe(f: Farbe) -> Color32 {
    Color32::from_rgba_unmultiplied(f.r, f.g, f.b, f.a)
}

/// Zeichnet die Anzeige auf die ganze Fläche und liefert die Eingaben dieses Bildes.
/// `skala` ist die Skalierung des Overlays (1.0 = 100 %). Das Overlay skaliert selbst statt
/// über `set_zoom_factor`, weil der Zoom auch das Einstellungsfenster träfe.
pub fn zeichnen(
    ui: &mut egui::Ui,
    anzeige: &Anzeige,
    schrift: &FontFamily,
    skala: f32,
    bild: Option<&TextureHandle>,
    zeiger: &mut Zeiger,
) -> Vec<Eingabe> {
    let s = skala;
    let flaeche = Rect::from_min_size(Pos2::ZERO, GRUNDGROESSE * s);
    let antwort = ui.interact(
        flaeche,
        egui::Id::unique("overlay"),
        Sense::click_and_drag(),
    );
    let maler = ui.painter();

    hintergrund(maler, flaeche, anzeige, bild);
    zeit(maler, flaeche, anzeige, schrift, s);
    maler.text(
        pos2(flaeche.center().x, 62.0 * s),
        Align2::CENTER_CENTER,
        anzeige.phasenwort,
        FontId::new(WORT_GROESSE * s, schrift.clone()),
        farbe(anzeige.akzent),
    );
    zyklus_punkte(maler, flaeche, anzeige, s);
    balken(maler, flaeche, anzeige, s);

    eingaben(ui, &antwort, zeiger)
}

fn hintergrund(
    maler: &egui::Painter,
    flaeche: Rect,
    anzeige: &Anzeige,
    bild: Option<&TextureHandle>,
) {
    let rund = CornerRadius::ZERO;
    match bild {
        None => {
            maler.rect_filled(flaeche, rund, farbe(anzeige.hintergrund));
        }
        Some(textur) => {
            // „füllen“: proportional bis die Fläche bedeckt ist, mittig beschnitten
            let [b, h] = textur.size().map(|n| n as f32);
            let (sx, sy) = (flaeche.width() / b, flaeche.height() / h);
            let skala = sx.max(sy);
            let (uv_b, uv_h) = (sx / skala, sy / skala);
            let uv = Rect::from_center_size(pos2(0.5, 0.5), vec2(uv_b, uv_h));
            // Die Deckkraft von `hintergrund` gilt fürs Bild (SPEC 7.5)
            let farbton = Color32::from_white_alpha(anzeige.hintergrund.a);
            maler.add(
                egui::epaint::RectShape::filled(flaeche, rund, farbton)
                    .with_texture(textur.id(), uv),
            );
            maler.rect_filled(flaeche, rund, farbe(anzeige.blinkschicht));
        }
    }
}

fn zeit(maler: &egui::Painter, flaeche: Rect, anzeige: &Anzeige, schrift: &FontFamily, s: f32) {
    let farbe_zeit = farbe(anzeige.vordergrund);
    let mitte = pos2(flaeche.center().x, 30.0 * s);
    let text = maler.text(
        mitte,
        Align2::CENTER_CENTER,
        &anzeige.zeit,
        FontId::new(ZEIT_GROESSE * s, schrift.clone()),
        farbe_zeit,
    );
    // Symbol links neben der Zeit, als Form gezeichnet
    let groesse = 12.0 * s;
    let symbol = Rect::from_center_size(
        pos2(text.left() - 6.0 * s - groesse / 2.0, mitte.y),
        Vec2::splat(groesse),
    );
    match anzeige.symbol {
        Symbol::Keins => {}
        Symbol::Pause => {
            let balken = groesse * 0.32;
            for x in [symbol.left(), symbol.right() - balken] {
                let r = Rect::from_min_size(pos2(x, symbol.top()), vec2(balken, groesse));
                maler.rect_filled(r, CornerRadius::same(1), farbe_zeit);
            }
        }
        Symbol::Start => {
            let ecken = vec![
                symbol.left_top(),
                pos2(symbol.right(), symbol.center().y),
                symbol.left_bottom(),
            ];
            maler.add(egui::Shape::convex_polygon(ecken, farbe_zeit, Stroke::NONE));
        }
    }
}

fn zyklus_punkte(maler: &egui::Painter, flaeche: Rect, anzeige: &Anzeige, s: f32) {
    let anzahl = anzeige.punkte.len() as f32;
    if anzahl == 0.0 {
        return;
    }
    // bei vielen Punkten kleiner, damit sie in eine Zeile passen
    let abstand = (140.0 / anzahl).min(14.0) * s;
    let radius = (abstand * 0.3).min(3.5 * s);
    let y = 76.0 * s;
    let start = flaeche.center().x - abstand * (anzahl - 1.0) / 2.0;
    for (i, punkt) in anzeige.punkte.iter().enumerate() {
        let mitte = pos2(start + abstand * i as f32, y);
        let _ = match punkt {
            Punkt::Voll => maler.circle_filled(mitte, radius, farbe(anzeige.vordergrund)),
            Punkt::Leer => maler.circle_filled(mitte, radius, farbe(anzeige.gedaempft)),
            Punkt::Hervorgehoben => maler.circle_stroke(
                mitte,
                radius - 0.6 * s,
                Stroke::new(1.2 * s, farbe(anzeige.akzent)),
            ),
        };
    }
}

fn balken(maler: &egui::Painter, flaeche: Rect, anzeige: &Anzeige, s: f32) {
    // Abstand zu den von Windows gerundeten Ecken; die skalieren nicht mit dem Overlay
    let rand = ECKENABSTAND;
    let spur = Rect::from_min_max(
        pos2(
            flaeche.left() + rand,
            flaeche.bottom() - (BALKEN_HOEHE + 3.0) * s,
        ),
        pos2(flaeche.right() - rand, flaeche.bottom() - 3.0 * s),
    );
    let rund = CornerRadius::same(1);
    maler.rect_filled(spur, rund, farbe(anzeige.gedaempft));
    if anzeige.fortschritt > 0.0 {
        let mut fuellung = spur;
        fuellung.set_width(spur.width() * anzeige.fortschritt.clamp(0.0, 1.0));
        maler.rect_filled(fuellung, rund, farbe(anzeige.akzent));
    }
}

fn eingaben(ui: &egui::Ui, antwort: &egui::Response, zeiger: &mut Zeiger) -> Vec<Eingabe> {
    let mut eingaben = Vec::new();
    let mut ziehen_starten = false;
    // Achtung: In `ui.input` hält egui eine Lesesperre auf den Kontext. Viewport-Befehle
    // erst danach senden, sonst verklemmt es sich.
    ui.input(|i| {
        // Linksklick ohne Bewegung = Primäraktion, sonst Ziehen
        if i.pointer.primary_pressed() && antwort.hovered() {
            zeiger.gedrueckt_bei = i.pointer.interact_pos();
            zeiger.zieht = false;
        }
        if let (Some(start), Some(jetzt)) = (zeiger.gedrueckt_bei, i.pointer.interact_pos())
            && i.pointer.primary_down()
            && !zeiger.zieht
            && start.distance(jetzt) > ZIEHSCHWELLE
        {
            zeiger.zieht = true;
            ziehen_starten = true;
        }
        if i.pointer.primary_released() {
            if zeiger.gedrueckt_bei.is_some() && !zeiger.zieht {
                eingaben.push(Eingabe::Aktion(Aktion::Primaeraktion));
            }
            zeiger.gedrueckt_bei = None;
            zeiger.zieht = false;
        }

        // Strg+Mausrad: 10 % pro Rastung
        for ereignis in &i.events {
            if let egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } = ereignis
                && modifiers.command
            {
                zeiger.rad_rest += match unit {
                    egui::MouseWheelUnit::Line => delta.y,
                    egui::MouseWheelUnit::Point => delta.y / 50.0,
                    egui::MouseWheelUnit::Page => delta.y * 3.0,
                };
            }
        }
        let rastungen = zeiger.rad_rest.trunc();
        if rastungen != 0.0 {
            zeiger.rad_rest -= rastungen;
            eingaben.push(Eingabe::Skalieren(rastungen as i32));
        }

        // lokale Tasten (nur mit Fokus, SPEC 4.3)
        if i.key_pressed(Key::Space) {
            eingaben.push(Eingabe::Aktion(Aktion::Primaeraktion));
        }
        let umschalt_f10 = i.key_released(Key::F10) && i.modifiers.shift;
        if umschalt_f10 || i.key_pressed(Key::Menu) {
            eingaben.push(Eingabe::MenueOeffnen(Some(Pos2::ZERO)));
        }
    });
    if ziehen_starten {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        eingaben.push(Eingabe::ZiehenBegonnen);
    }
    if antwort.secondary_clicked() {
        eingaben.push(Eingabe::MenueOeffnen(None));
    }
    eingaben
}
