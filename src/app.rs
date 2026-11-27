//! Die Schale (SPEC 11.6): verbindet Kern, Einstellungsdatei, Overlay und Windows.
//! `logic` läuft jede Sekunde, auch bei ausgeblendetem Overlay, und ist beliebig oft aufrufbar.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, FontFamily, ViewportCommand, WindowLevel};
use windows_sys::Win32::Foundation::{HANDLE, HWND};

use crate::ablauf::{Ablauf, Ereignis};
use crate::anzeige::{self, Anzeige};
use crate::einstellungsdatei::{Aenderung, Einstellungen, Einstellungsdatei};
use crate::hinweis::{Hinweis, Klang};
use crate::menue::{Aktion, Laufzeitfehler, Schalter};
use crate::overlay::{self, Eingabe, GRUNDGROESSE, Zeiger};
use crate::plattform::{self, Rechteck};
use crate::theme::{self, Schrift, Theme};
use crate::themeordner::Themeordner;

static FOKUS_ENDET: &[u8] = include_bytes!("../assets/fokus_endet.wav");
static PAUSE_ENDET: &[u8] = include_bytes!("../assets/pause_endet.wav");

/// Abstand der Startposition zum Rand des Arbeitsbereichs (logische Pixel).
const STARTABSTAND: f32 = 24.0;
const SEKUNDE: Duration = Duration::from_secs(1);

/// Der Ordner `%USERPROFILE%\.rusty_pomodoro_overlay`.
pub fn app_ordner() -> PathBuf {
    let heim = std::env::var_os("USERPROFILE").map_or_else(|| PathBuf::from("."), PathBuf::from);
    heim.join(".rusty_pomodoro_overlay")
}

pub struct App {
    ablauf: Ablauf,
    hinweis: Hinweis,
    datei: Einstellungsdatei,
    themeordner: Themeordner,
    theme: Theme,
    /// Schlüssel des tatsächlich verwendeten Themes (Rückfall: `dunkel`)
    aktives_theme: String,
    zeigen_event: HANDLE,

    hwnd: Option<HWND>,
    /// Click-through wird nie gespeichert und startet aus.
    click_through: bool,
    /// was zuletzt aufs Fenster angewandt wurde
    angewandt: Option<Fensterzustand>,
    laufzeitfehler: Vec<Laufzeitfehler>,
    naechste_fehler_id: u32,

    anzeige: Anzeige,
    eingaben: Vec<Eingabe>,
    zeiger: Zeiger,
    /// nach Ziehen oder Skalieren: Position bald prüfen
    position_pruefen: bool,
    naechste_sekundenpruefung: Instant,
    /// Nur Beenden über das Menü schließt; sonst wird Schließen verworfen.
    beenden: bool,
    /// `GL_RENDERER`, einmal beim Start gelesen
    grafik: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Fensterzustand {
    immer_im_vordergrund: bool,
    click_through: bool,
    sichtbar: bool,
    skalierung: u16,
}

impl App {
    pub fn neu(zeigen_event: HANDLE) -> Self {
        let ordner = app_ordner();
        let datei = Einstellungsdatei::oeffnen(&ordner);
        let mut themeordner = Themeordner::neu(&ordner.join("themes"));
        themeordner.takt();
        let einstellungen = datei.aktuell().0.clone();
        let (theme, aktives_theme) = theme_aufloesen(&themeordner, &einstellungen.theme);
        let ablauf = Ablauf::neu(einstellungen.ablauf());
        let jetzt = Instant::now();
        let anzeige = anzeige::anzeige(&ablauf.stand(jetzt), &theme, 0.0, false);
        Self {
            ablauf,
            hinweis: Hinweis::default(),
            datei,
            themeordner,
            theme,
            aktives_theme,
            zeigen_event,
            hwnd: None,
            click_through: false,
            angewandt: None,
            laufzeitfehler: Vec::new(),
            naechste_fehler_id: 1,
            anzeige,
            eingaben: Vec::new(),
            zeiger: Zeiger::default(),
            position_pruefen: false,
            naechste_sekundenpruefung: jetzt,
            beenden: false,
            grafik: String::new(),
        }
    }

    fn einstellungen(&self) -> &Einstellungen {
        self.datei.aktuell().0
    }

    /// Erster `logic`-Aufruf: HWND holen, Fensterstil, Position (SPEC 10.1 Punkt 5).
    fn einrichten(&mut self, frame: &eframe::Frame) {
        let Some(hwnd) = plattform::hwnd_von(frame) else {
            return;
        };
        self.hwnd = Some(hwnd);
        if let Some(gl) = frame.gl() {
            use eframe::glow::HasContext as _;
            // für die Zeile „Grafik“ im Einstellungsfenster (SPEC 8.5)
            self.grafik = unsafe { gl.get_parameter_string(eframe::glow::RENDERER) };
        }
        plattform::unterklasse_anhaengen(hwnd, self.datei.sicherung());
        plattform::als_werkzeugfenster(hwnd);
    }

    /// Die eine Verteilstelle für alle Eingaben (SPEC 11.6).
    fn ausfuehren(&mut self, aktion: Aktion, jetzt: Instant) {
        let e = self.einstellungen().clone();
        match aktion {
            Aktion::Primaeraktion => self.befehl(crate::ablauf::Befehl::Primaeraktion, jetzt),
            Aktion::Ueberspringen => self.befehl(crate::ablauf::Befehl::Ueberspringen, jetzt),
            Aktion::PhaseZuruecksetzen => {
                self.befehl(crate::ablauf::Befehl::PhaseZuruecksetzen, jetzt)
            }
            Aktion::NeuBeginnen => self.befehl(crate::ablauf::Befehl::NeuBeginnen, jetzt),
            Aktion::Umschalten(Schalter::ImmerImVordergrund) => {
                self.aendern(
                    Aenderung::ImmerImVordergrund(!e.immer_im_vordergrund),
                    jetzt,
                );
            }
            Aktion::Umschalten(Schalter::ClickThrough) => self.click_through = !self.click_through,
            Aktion::Umschalten(Schalter::OverlayAnzeigen) => {
                self.aendern(Aenderung::OverlayAnzeigen(!e.overlay_anzeigen), jetzt);
            }
            Aktion::Umschalten(Schalter::Ton) => self.aendern(Aenderung::Ton(!e.ton), jetzt),
            Aktion::ThemeWaehlen(schluessel) => {
                self.aendern(Aenderung::Theme(schluessel), jetzt);
                self.theme_anwenden();
            }
            Aktion::ThemeOrdnerOeffnen => {
                let _ = std::fs::create_dir_all(self.themeordner.pfad());
                plattform::oeffnen(self.themeordner.pfad(), "explore");
            }
            Aktion::EinstellungsdateiOeffnen => {
                let pfad = self.datei.pfad();
                if !plattform::oeffnen(&pfad, "open") {
                    let _ = std::process::Command::new("notepad.exe").arg(&pfad).spawn();
                }
            }
            Aktion::EinstellungsfensterOeffnen => {} // Schritt 6
            Aktion::FehlerAusblenden(id) => self.laufzeitfehler.retain(|f| f.id != id),
            Aktion::Beenden => self.beenden = true,
        }
    }

    fn befehl(&mut self, befehl: crate::ablauf::Befehl, jetzt: Instant) {
        let ereignisse = self.ablauf.befehl(befehl, jetzt);
        self.hinweise(ereignisse, jetzt);
    }

    fn aendern(&mut self, aenderung: Aenderung, jetzt: Instant) {
        self.datei.aendern(aenderung, jetzt);
        self.ablauf.einstellungen(self.einstellungen().ablauf());
    }

    fn laufzeitfehler(&mut self, text: &str) {
        if !self.laufzeitfehler.iter().any(|f| f.text == text) {
            self.laufzeitfehler.push(Laufzeitfehler {
                id: self.naechste_fehler_id,
                text: text.into(),
            });
            self.naechste_fehler_id += 1;
        }
    }

    fn theme_anwenden(&mut self) {
        let schluessel = self.einstellungen().theme.clone();
        (self.theme, self.aktives_theme) = theme_aufloesen(&self.themeordner, &schluessel);
    }

    fn hinweise(&mut self, ereignisse: Vec<Ereignis>, jetzt: Instant) {
        for Ereignis::PhaseAbgelaufen { von, .. } in ereignisse {
            let schalter = self.einstellungen().hinweisschalter();
            let aktionen =
                self.hinweis
                    .ausloesen(von, schalter, plattform::stoeren_erlaubt(), jetzt);
            match aktionen.klang {
                Some(Klang::FokusEndet) => plattform::klang_abspielen(FOKUS_ENDET),
                Some(Klang::PauseEndet) => plattform::klang_abspielen(PAUSE_ENDET),
                None => {}
            }
            if aktionen.einblenden && !self.einstellungen().overlay_anzeigen {
                self.aendern(Aenderung::OverlayAnzeigen(true), jetzt);
            }
        }
    }

    /// Immer-im-Vordergrund, Click-through, Sichtbarkeit und Größe aufs Fenster bringen,
    /// aber nur, was sich geändert hat. Die Sichtbarkeit wird mit dem echten Fenster
    /// verglichen: eframe zeigt das Fenster nach dem ersten Bild selbst an.
    fn fenster_anwenden(&mut self, ctx: &egui::Context) {
        let Some(hwnd) = self.hwnd else { return };
        let e = self.einstellungen();
        let soll = Fensterzustand {
            immer_im_vordergrund: e.immer_im_vordergrund,
            click_through: self.click_through,
            sichtbar: e.overlay_anzeigen,
            skalierung: e.skalierung,
        };
        let ist = self.angewandt;
        if ist.map(|z| z.immer_im_vordergrund) != Some(soll.immer_im_vordergrund) {
            let ebene = if soll.immer_im_vordergrund {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(ebene));
        }
        if ist.map(|z| z.click_through) != Some(soll.click_through) {
            // erst zur Laufzeit, nie im ViewportBuilder (egui #2537)
            ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(soll.click_through));
        }
        if ist.map(|z| z.skalierung) != Some(soll.skalierung) {
            self.groesse_setzen(ctx, hwnd, soll.skalierung, ist.is_none());
        }
        if plattform::ist_sichtbar(hwnd) != soll.sichtbar {
            // immer über plattform, damit es nie den Fokus nimmt (SPEC 5.1)
            if soll.sichtbar {
                plattform::zeigen_ohne_fokus(hwnd);
            } else {
                plattform::ausblenden(hwnd);
            }
        }
        self.angewandt = Some(soll);
    }

    /// Größe aus Skalierung und DPI, um die Mitte herum. Beim ersten Mal an die gespeicherte
    /// bzw. die Startposition.
    fn groesse_setzen(
        &mut self,
        ctx: &egui::Context,
        hwnd: HWND,
        skalierung: u16,
        erstes_mal: bool,
    ) {
        let faktor = f32::from(skalierung) / 100.0;
        ctx.set_zoom_factor(faktor);
        let dpi = plattform::dpi_faktor(hwnd);
        let breite = (GRUNDGROESSE.x * faktor * dpi).round() as i32;
        let hoehe = (GRUNDGROESSE.y * faktor * dpi).round() as i32;
        let (x, y) = if erstes_mal {
            self.einstellungen()
                .position
                .unwrap_or_else(|| self.startposition(breite, hoehe, dpi))
        } else {
            let alt = plattform::fenster_rechteck(hwnd).unwrap_or(Rechteck {
                x: 0,
                y: 0,
                breite,
                hoehe,
            });
            let (mx, my) = alt.mitte();
            (mx - breite / 2, my - hoehe / 2)
        };
        plattform::fenster_setzen(
            hwnd,
            Rechteck {
                x,
                y,
                breite,
                hoehe,
            },
        );
        self.position_pruefen = true;
    }

    fn startposition(&self, breite: i32, hoehe: i32, dpi: f32) -> (i32, i32) {
        plattform::startposition(breite, hoehe, (STARTABSTAND * dpi).round() as i32)
    }

    /// Liegt das Overlay ganz in einem Arbeitsbereich? Sonst hineinschieben; neue Lage speichern.
    fn position_einhalten(&mut self, jetzt: Instant) {
        let Some(hwnd) = self.hwnd else { return };
        if plattform::maustaste_gedrueckt() {
            return;
        }
        let Some(mut r) = plattform::fenster_rechteck(hwnd) else {
            return;
        };
        let start = self.startposition(r.breite, r.hoehe, plattform::dpi_faktor(hwnd));
        if let Some(neu) = plattform::in_arbeitsbereich(r, start) {
            plattform::fenster_setzen(hwnd, neu);
            r = neu;
        }
        if self.einstellungen().position != Some((r.x, r.y)) {
            self.aendern(Aenderung::Position(r.x, r.y), jetzt);
        }
        self.position_pruefen = false;
    }

    fn skalieren(&mut self, rastungen: i32, jetzt: Instant) {
        let alt = i32::from(self.einstellungen().skalierung);
        let neu = (alt + 10 * rastungen).clamp(50, 300) as u16;
        if i32::from(neu) != alt {
            self.aendern(Aenderung::Skalierung(neu), jetzt);
        }
    }
}

fn theme_aufloesen(ordner: &Themeordner, schluessel: &str) -> (Theme, String) {
    if let Some((s, t)) = theme::eingebaute()
        .into_iter()
        .find(|(s, _)| *s == schluessel)
    {
        return (t, s.to_owned());
    }
    match ordner.theme(schluessel) {
        Some(t) => (t.clone(), schluessel.to_owned()),
        None => (theme::dunkel(), "dunkel".to_owned()),
    }
}

fn schriftfamilie(schrift: Schrift) -> FontFamily {
    match schrift {
        // Inter und JetBrains Mono kommen mit den Schriftdateien (Schritt 6)
        Schrift::Ubuntu | Schrift::Inter | Schrift::JetbrainsMono => FontFamily::Proportional,
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Alpha pro Pixel
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let jetzt = Instant::now();
        if self.hwnd.is_none() {
            ctx.options_mut(|o| o.zoom_with_keyboard = false);
            self.einrichten(frame);
        }

        // 1. Eingaben
        for eingabe in std::mem::take(&mut self.eingaben) {
            match eingabe {
                Eingabe::Aktion(aktion) => self.ausfuehren(aktion, jetzt),
                Eingabe::Skalieren(n) => self.skalieren(n, jetzt),
                Eingabe::ZiehenBegonnen => self.position_pruefen = true,
                Eingabe::MenueOeffnen(_) => {} // Schritt 5
            }
        }

        // 2. Einstellungsdatei; 4. fällige Prüfungen im Sekundentakt
        let sekunde_um = jetzt >= self.naechste_sekundenpruefung;
        if sekunde_um {
            self.naechste_sekundenpruefung = jetzt + SEKUNDE;
            if self.datei.takt(jetzt).is_some() {
                self.ablauf.einstellungen(self.einstellungen().ablauf());
                self.theme_anwenden();
            }
            if self.themeordner.takt() {
                self.theme_anwenden();
            }
            if plattform::zweiter_start_angefragt(self.zeigen_event) {
                if !self.einstellungen().overlay_anzeigen {
                    self.aendern(Aenderung::OverlayAnzeigen(true), jetzt);
                }
                self.hinweis.blinken_starten(jetzt);
            }
            self.position_pruefen = true;
        } else {
            // verzögertes Schreiben (Position, Skalierung) braucht den Takt auch dazwischen
            self.datei.takt(jetzt);
        }
        if self.datei.schreibfehler_abholen() {
            self.laufzeitfehler("Einstellungen: Datei konnte nicht geschrieben werden");
        }

        // 3. Ablauf
        let ereignisse = self.ablauf.takt(jetzt);
        self.hinweise(ereignisse, jetzt);

        self.fenster_anwenden(ctx);
        if self.position_pruefen {
            self.position_einhalten(jetzt);
        }

        // 5. Anzeige
        let fehler = !self.datei.aktuell().1.is_empty() || !self.laufzeitfehler.is_empty();
        self.anzeige = anzeige::anzeige(
            &self.ablauf.stand(jetzt),
            &self.theme,
            self.hinweis.aufblinken(jetzt),
            fehler,
        );

        // Alt+F4 verwerfen, nur Beenden schließt (SPEC 5.1, 10.4)
        if ctx.input(|i| i.viewport().close_requested()) && !self.beenden {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if self.beenden {
            self.datei.sichern();
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        // 6. nächster Aufruf
        let mut faellig = jetzt + SEKUNDE;
        for t in [
            self.ablauf.naechste_faelligkeit(jetzt),
            self.hinweis.naechste_faelligkeit(jetzt),
            Some(self.naechste_sekundenpruefung),
        ]
        .into_iter()
        .flatten()
        {
            faellig = faellig.min(t);
        }
        if self.position_pruefen {
            faellig = faellig.min(jetzt + Duration::from_millis(100));
        }
        ctx.request_repaint_after(faellig.saturating_duration_since(jetzt));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let schrift = schriftfamilie(self.theme.schrift);
        let eingaben = if self.click_through {
            // bei Click-through wirken Klicks und Mausrad nicht
            overlay::zeichnen(ui, &self.anzeige, &schrift, None, &mut Zeiger::default());
            Vec::new()
        } else {
            overlay::zeichnen(ui, &self.anzeige, &schrift, None, &mut self.zeiger)
        };
        if !eingaben.is_empty() {
            self.eingaben.extend(eingaben);
            ui.ctx().request_repaint();
        }
    }
}
