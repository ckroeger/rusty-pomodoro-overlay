//! Die Schale (SPEC 11.6): verbindet Kern, Einstellungsdatei, Overlay und Windows.
//! `logic` läuft jede Sekunde, auch bei ausgeblendetem Overlay, und ist beliebig oft aufrufbar.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, ViewportCommand, WindowLevel};
use windows_sys::Win32::Foundation::{HANDLE, HWND};

use crate::ablauf::{Ablauf, Ereignis};
use crate::anzeige::Trayschluessel;
use crate::anzeige::{self, Anzeige};
use crate::einstellungsdatei::{Aenderung, Einstellungen, Einstellungsdatei};
use crate::einstellungsfenster::{Einstellungsfenster, Fensteraktion, Formular};
use crate::grafik::{self, Hintergrund, Trayzeichner};
use crate::hinweis::{Hinweis, Klang};
use crate::menue::{self, Aktion, Laufzeitfehler, Menuemodell, Schalter, Schalterstand};
use crate::overlay::{self, Eingabe, GRUNDGROESSE, Zeiger};
use crate::plattform::{self, Rechteck};
use crate::theme::{self, Farben, Theme};
use crate::themeordner::Themeordner;
use crate::tray::Tray;

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

    /// `None`, wenn sich das Tray-Icon nicht anlegen ließ
    tray: Option<Tray>,
    trayzeichner: Trayzeichner,
    /// woraus das Tray-Icon zuletzt gezeichnet wurde
    tray_gezeichnet: Option<(Trayschluessel, Farben, Option<String>, u32)>,
    hintergrund: Hintergrund,
    /// Klick auf das aktive Theme: Bild neu dekodieren
    bild_neu_laden: bool,

    fenster: Einstellungsfenster,
    fensteraktionen: Vec<Fensteraktion>,
    /// ob der Autostart-Eintrag existiert; gelesen beim Öffnen und nach dem Umschalten
    autostart: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Fensterzustand {
    immer_im_vordergrund: bool,
    click_through: bool,
    sichtbar: bool,
    skalierung: u16,
}

impl App {
    pub fn neu(ctx: &egui::Context, zeigen_event: HANDLE) -> Self {
        let ordner = app_ordner();
        let datei = Einstellungsdatei::oeffnen(&ordner);
        let mut themeordner = Themeordner::neu(&ordner.join("themes"));
        themeordner.takt();
        let einstellungen = datei.aktuell().0.clone();
        let (theme, aktives_theme) = theme_aufloesen(&themeordner, &einstellungen.theme);
        let ablauf = Ablauf::neu(einstellungen.ablauf());
        let jetzt = Instant::now();
        let anzeige = anzeige::anzeige(&ablauf.stand(jetzt), &theme, 0.0, false);
        let mut app = Self {
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
            tray: None,
            trayzeichner: Trayzeichner::neu(grafik::systemschriften()),
            tray_gezeichnet: None,
            hintergrund: Hintergrund::default(),
            bild_neu_laden: false,
            fenster: Einstellungsfenster::default(),
            fensteraktionen: Vec::new(),
            autostart: false,
        };
        let modell = app.menuemodell(jetzt);
        let icon = app.tray_icon();
        app.tray = icon.and_then(|icon| Tray::neu(ctx, &modell, icon));
        app
    }

    /// Das gemeinsame Menü aus dem aktuellen Stand (SPEC 4.4).
    fn menuemodell(&self, jetzt: Instant) -> Menuemodell {
        let (e, fehler) = self.datei.aktuell();
        let einstellungsfehler: Vec<String> = fehler.iter().map(|f| f.text()).collect();
        let mut themes = self.themeordner.liste();
        if let Some(kaputt) = self.hintergrund.fehlgeschlagen() {
            for t in &mut themes {
                let bild = self
                    .themeordner
                    .theme(&t.schluessel)
                    .and_then(|t| t.bild.as_deref());
                t.bild_fehlt |= bild == Some(kaputt);
            }
        }
        menue::menue(
            &self.ablauf.stand(jetzt),
            Schalterstand {
                immer_im_vordergrund: e.immer_im_vordergrund,
                click_through: self.click_through,
                overlay_anzeigen: e.overlay_anzeigen,
                ton: e.ton,
            },
            &einstellungsfehler,
            &self.laufzeitfehler,
            &themes,
            &self.aktives_theme,
        )
    }

    /// Zeichnet das Tray-Icon neu, wenn sich Phase, Dimmen, Theme oder Größe geändert haben.
    fn tray_icon(&mut self) -> Option<tray_icon::Icon> {
        let groesse = plattform::tray_icon_groesse();
        let schluessel = (
            self.anzeige.tray,
            self.theme.farben,
            self.theme.tray_zeichen.clone(),
            groesse,
        );
        if self.tray_gezeichnet.as_ref() == Some(&schluessel) {
            return None;
        }
        let rgba = self.trayzeichner.icon(
            self.theme.tray_zeichen.as_deref(),
            self.anzeige.akzent,
            groesse,
        );
        self.tray_gezeichnet = Some(schluessel);
        tray_icon::Icon::from_rgba(rgba, groesse, groesse).ok()
    }

    /// Bildschirmpixel der Zielfläche fürs Hintergrundbild: 300 % × DPI (SPEC 7.5).
    fn bildflaeche(&self) -> (u32, u32) {
        let dpi = self.hwnd.map_or(1.0, plattform::dpi_faktor);
        let seite = |n: f32| (n * 3.0 * dpi).round() as u32;
        (seite(GRUNDGROESSE.x), seite(GRUNDGROESSE.y))
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
                self.bild_neu_laden |= schluessel == self.aktives_theme;
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
            Aktion::EinstellungsfensterOeffnen => {
                self.autostart = plattform::autostart_an();
                self.fenster.oeffnen();
            }
            Aktion::FehlerAusblenden(id) => self.laufzeitfehler.retain(|f| f.id != id),
            Aktion::Beenden => self.beenden = true,
        }
    }

    fn fensteraktion(&mut self, aktion: Fensteraktion, jetzt: Instant) {
        match aktion {
            Fensteraktion::Aendern(aenderung) => {
                let theme = matches!(aenderung, Aenderung::Theme(_));
                self.aendern(aenderung, jetzt);
                if theme {
                    self.theme_anwenden();
                }
            }
            Fensteraktion::Autostart(an) => {
                if !plattform::autostart_setzen(an) {
                    self.laufzeitfehler("Autostart: Eintrag konnte nicht gesetzt werden");
                }
                self.autostart = plattform::autostart_an();
            }
            Fensteraktion::DateiOeffnen => self.ausfuehren(Aktion::EinstellungsdateiOeffnen, jetzt),
            Fensteraktion::ThemeOrdnerOeffnen => self.ausfuehren(Aktion::ThemeOrdnerOeffnen, jetzt),
        }
    }

    /// Die wählbaren Themes fürs Einstellungsfenster: wie im Menü, ohne kaputte.
    fn waehlbare_themes(&self) -> Vec<(String, String)> {
        let mut liste: Vec<(String, String)> = theme::eingebaute()
            .into_iter()
            .map(|(s, t)| (s.to_owned(), t.name))
            .collect();
        liste.extend(
            self.themeordner
                .liste()
                .into_iter()
                .filter(|t| t.zustand == theme::Themezustand::Gueltig)
                .map(|t| (t.schluessel, t.name)),
        );
        liste
    }

    /// Öffnet das gemeinsame Menü am Overlay; die Auswahl kommt als Menü-Ereignis.
    fn menue_zeigen(&mut self, stelle: Option<egui::Pos2>, jetzt: Instant) {
        let modell = self.menuemodell(jetzt);
        if let (Some(tray), Some(hwnd)) = (&mut self.tray, self.hwnd) {
            tray.menue_anwenden(&modell);
            tray.menue_zeigen(hwnd, stelle.map(|p| (f64::from(p.x), f64::from(p.y))));
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
            self.groesse_setzen(hwnd, soll.skalierung, ist.is_none());
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
    fn groesse_setzen(&mut self, hwnd: HWND, skalierung: u16, erstes_mal: bool) {
        let faktor = f32::from(skalierung) / 100.0;
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

        // 1. Eingaben: Menü und Tray, dann das Overlay
        for aktion in self.tray.as_ref().map(Tray::aktionen).unwrap_or_default() {
            self.ausfuehren(aktion, jetzt);
        }
        for aktion in std::mem::take(&mut self.fensteraktionen) {
            self.fensteraktion(aktion, jetzt);
        }
        for eingabe in std::mem::take(&mut self.eingaben) {
            match eingabe {
                Eingabe::Aktion(aktion) => self.ausfuehren(aktion, jetzt),
                Eingabe::Skalieren(n) => self.skalieren(n, jetzt),
                Eingabe::ZiehenBegonnen => self.position_pruefen = true,
                Eingabe::MenueOeffnen(stelle) => self.menue_zeigen(stelle, jetzt),
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
        let flaeche = self.bildflaeche();
        let neu_laden = std::mem::take(&mut self.bild_neu_laden);
        self.hintergrund
            .setzen(ctx, self.theme.bild.as_deref(), flaeche, neu_laden);
        self.hintergrund.abholen(ctx);

        // 5. Anzeige
        let fehler = !self.datei.aktuell().1.is_empty() || !self.laufzeitfehler.is_empty();
        self.anzeige = anzeige::anzeige(
            &self.ablauf.stand(jetzt),
            &self.theme,
            self.hinweis.aufblinken(jetzt),
            fehler,
        );
        let modell = self.menuemodell(jetzt);
        let icon = self.tray_icon();
        if let Some(tray) = &mut self.tray {
            tray.tooltip_setzen(&self.anzeige.tooltip);
            if let Some(icon) = icon {
                tray.icon_setzen(icon);
            }
            tray.menue_anwenden(&modell);
        }

        // ausgeblendet zeichnet eframe sonst nicht, und mit dem Overlay das Fenster nicht
        if self.fenster.ist_offen() && !self.einstellungen().overlay_anzeigen {
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
        }

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
        let schrift = crate::schriften::familie(self.theme.schrift);
        let bild = self.hintergrund.textur();
        // die angewandte Größe, damit Zeichnung und Fenster zusammenpassen
        let skala = self
            .angewandt
            .map_or(1.0, |z| f32::from(z.skalierung) / 100.0);
        let eingaben = if self.click_through {
            // bei Click-through wirken Klicks und Mausrad nicht
            overlay::zeichnen(
                ui,
                &self.anzeige,
                &schrift,
                skala,
                bild,
                &mut Zeiger::default(),
            );
            Vec::new()
        } else {
            overlay::zeichnen(ui, &self.anzeige, &schrift, skala, bild, &mut self.zeiger)
        };
        if !eingaben.is_empty() {
            self.eingaben.extend(eingaben);
            ui.ctx().request_repaint();
        }

        if self.fenster.ist_offen() {
            let fehler: Vec<String> = self.datei.aktuell().1.iter().map(|f| f.text()).collect();
            let themes = self.waehlbare_themes();
            let formular = Formular {
                einstellungen: self.datei.aktuell().0,
                fehler: &fehler,
                themes: &themes,
                autostart: self.autostart,
                grafik: &self.grafik,
            };
            let aktionen = self.fenster.zeigen(ui.ctx(), &formular);
            if !aktionen.is_empty() || !self.fenster.ist_offen() {
                self.fensteraktionen.extend(aktionen);
                ui.ctx().request_repaint();
            }
        }
    }
}
