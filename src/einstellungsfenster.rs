//! Das Einstellungsfenster (SPEC 8.5): ein immediate Viewport, nur solange es offen ist.
//! Kein eigener Zwischenstand: Es zeigt bei jedem Zeichnen den aktuellen Stand und meldet
//! jede Änderung sofort als Aktion.

use eframe::egui::{self, Color32, DragValue, ViewportBuilder, ViewportCommand, ViewportId};

use crate::einstellungsdatei::{Aenderung, Einstellungen};

const TITEL: &str = "Rusty Pomodoro Overlay – Einstellungen";
const GROESSE: [f32; 2] = [380.0, 470.0];

/// Was im Fenster ausgelöst wurde; die Schale führt es im nächsten `logic` aus.
#[derive(Debug, Clone, PartialEq)]
pub enum Fensteraktion {
    Aendern(Aenderung),
    Autostart(bool),
    DateiOeffnen,
    ThemeOrdnerOeffnen,
}

/// Was das Fenster anzeigt.
pub struct Formular<'a> {
    pub einstellungen: &'a Einstellungen,
    /// fertige Texte, eine Zeile je Fehler
    pub fehler: &'a [String],
    /// wählbare Themes: (Schlüssel, Name) in Menüreihenfolge
    pub themes: &'a [(String, String)],
    pub autostart: bool,
    /// `GL_RENDERER`
    pub grafik: &'a str,
}

#[derive(Default)]
pub struct Einstellungsfenster {
    offen: bool,
    nach_vorn: bool,
    zentrieren: bool,
}

impl Einstellungsfenster {
    /// Öffnet das Fenster oder holt das offene nach vorn.
    pub fn oeffnen(&mut self) {
        if self.offen {
            self.nach_vorn = true;
        } else {
            self.offen = true;
            self.zentrieren = true;
        }
    }

    pub fn ist_offen(&self) -> bool {
        self.offen
    }

    /// Zeichnet das Fenster, wenn es offen ist. Aus `App::ui` aufzurufen.
    pub fn zeigen(&mut self, ctx: &egui::Context, formular: &Formular) -> Vec<Fensteraktion> {
        if !self.offen {
            return Vec::new();
        }
        let builder = ViewportBuilder::default()
            .with_title(TITEL)
            .with_inner_size(GROESSE)
            .with_resizable(false)
            .with_maximize_button(false);
        let mut aktionen = Vec::new();
        ctx.show_viewport_immediate(
            ViewportId::from_hash_of("einstellungen"),
            builder,
            |ui, _klasse| {
                egui::CentralPanel::default().show(ui, |ui| {
                    inhalt(ui, formular, &mut aktionen);
                });
                let ctx = ui.ctx().clone();
                let (schliessen, esc) = ctx.input(|i| {
                    (
                        i.viewport().close_requested(),
                        i.key_pressed(egui::Key::Escape),
                    )
                });
                if schliessen || esc {
                    self.offen = false;
                }
                if self.zentrieren
                    && let Some(befehl) = ViewportCommand::center_on_screen(&ctx)
                {
                    ctx.send_viewport_cmd(befehl);
                    self.zentrieren = false;
                }
                if std::mem::take(&mut self.nach_vorn) {
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
            },
        );
        aktionen
    }
}

fn inhalt(ui: &mut egui::Ui, f: &Formular, aktionen: &mut Vec<Fensteraktion>) {
    let e = f.einstellungen;
    let mut aendern = |a: Aenderung| aktionen.push(Fensteraktion::Aendern(a));

    if !f.fehler.is_empty() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            for fehler in f.fehler {
                ui.colored_label(Color32::from_rgb(0xE0, 0x80, 0x20), format!("⚠ {fehler}"));
            }
        });
        ui.add_space(6.0);
    }

    ui.heading("Ablauf");
    egui::Grid::new("ablauf").num_columns(2).show(ui, |ui| {
        let mut minuten = |ui: &mut egui::Ui, text: &str, wert: u8, neu: fn(u8) -> Aenderung| {
            ui.label(text);
            let mut v = wert;
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut v).range(1..=180));
                ui.label("min");
            });
            ui.end_row();
            if v != wert {
                aendern(neu(v));
            }
        };
        minuten(ui, "Fokusphase", e.fokus_minuten, Aenderung::FokusMinuten);
        minuten(
            ui,
            "Kurze Pause",
            e.kurze_pause_minuten,
            Aenderung::KurzePauseMinuten,
        );
        minuten(
            ui,
            "Lange Pause",
            e.lange_pause_minuten,
            Aenderung::LangePauseMinuten,
        );
        ui.label("Fokusphasen pro Zyklus");
        let mut n = e.fokusphasen_pro_zyklus;
        ui.add(DragValue::new(&mut n).range(1..=12));
        ui.end_row();
        if n != e.fokusphasen_pro_zyklus {
            aendern(Aenderung::FokusphasenProZyklus(n));
        }
    });
    schalter(
        ui,
        "Pause automatisch starten",
        e.pause_automatisch_starten,
        |v| {
            aendern(Aenderung::PauseAutomatischStarten(v));
        },
    );
    schalter(
        ui,
        "Fokus automatisch starten",
        e.fokus_automatisch_starten,
        |v| {
            aendern(Aenderung::FokusAutomatischStarten(v));
        },
    );

    ui.add_space(8.0);
    ui.heading("Hinweis");
    schalter(ui, "Ton", e.ton, |v| aendern(Aenderung::Ton(v)));
    schalter(ui, "Signal im Overlay", e.signal_im_overlay, |v| {
        aendern(Aenderung::SignalImOverlay(v));
    });
    schalter(
        ui,
        "Overlay bei Phasenwechsel einblenden",
        e.overlay_bei_phasenwechsel_einblenden,
        |v| aendern(Aenderung::OverlayBeiPhasenwechselEinblenden(v)),
    );

    ui.add_space(8.0);
    ui.heading("Aussehen");
    ui.horizontal(|ui| {
        ui.label("Theme");
        let aktuell = f
            .themes
            .iter()
            .find(|(s, _)| *s == e.theme)
            .map_or(e.theme.as_str(), |(_, n)| n.as_str());
        egui::ComboBox::from_id_salt("theme")
            .selected_text(aktuell)
            .show_ui(ui, |ui| {
                for (schluessel, name) in f.themes {
                    if ui.selectable_label(*schluessel == e.theme, name).clicked()
                        && *schluessel != e.theme
                    {
                        aendern(Aenderung::Theme(schluessel.clone()));
                    }
                }
            });
    });

    ui.add_space(8.0);
    ui.heading("System");
    let mut autostart = f.autostart;
    if ui.checkbox(&mut autostart, "Mit Windows starten").changed() {
        aktionen.push(Fensteraktion::Autostart(autostart));
    }

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Datei öffnen").clicked() {
            aktionen.push(Fensteraktion::DateiOeffnen);
        }
        if ui.button("Theme-Ordner öffnen").clicked() {
            aktionen.push(Fensteraktion::ThemeOrdnerOeffnen);
        }
    });
    ui.label(format!(
        "Grafik: {} · Version {}",
        f.grafik,
        env!("CARGO_PKG_VERSION")
    ));
}

fn schalter(ui: &mut egui::Ui, text: &str, wert: bool, mut neu: impl FnMut(bool)) {
    let mut v = wert;
    if ui.checkbox(&mut v, text).changed() {
        neu(v);
    }
}
