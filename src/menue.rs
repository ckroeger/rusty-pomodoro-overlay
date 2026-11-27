//! Das gemeinsame Menü (SPEC 4.4, 7.4, 9) als reines Modell und die Aktionen aller Eingaben.

use crate::ablauf::{Phase, Stand, Zustand};
use crate::theme::{self, Themeeintrag, Themezustand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schalter {
    ImmerImVordergrund,
    ClickThrough,
    OverlayAnzeigen,
    Ton,
}

/// Alles, was eine Eingabe auslösen kann: Menü, Tray, Overlay, Tasten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aktion {
    Primaeraktion,
    Ueberspringen,
    PhaseZuruecksetzen,
    NeuBeginnen,
    Umschalten(Schalter),
    /// Schlüssel des Themes
    ThemeWaehlen(String),
    ThemeOrdnerOeffnen,
    EinstellungsfensterOeffnen,
    EinstellungsdateiOeffnen,
    /// Kennung eines Laufzeitfehlers
    FehlerAusblenden(u32),
    Beenden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schalterstand {
    pub immer_im_vordergrund: bool,
    pub click_through: bool,
    pub overlay_anzeigen: bool,
    pub ton: bool,
}

/// Ein Laufzeitfehler, den man per Klick ausblendet (SPEC 9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laufzeitfehler {
    pub id: u32,
    /// z. B. `Autostart: Eintrag konnte nicht gesetzt werden`
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eintrag {
    pub text: String,
    pub aktion: Aktion,
    pub aktiv: bool,
    /// `None`: kein Häkchen-Eintrag
    pub haekchen: Option<bool>,
}

impl Eintrag {
    fn neu(text: impl Into<String>, aktion: Aktion) -> Self {
        Self {
            text: text.into(),
            aktion,
            aktiv: true,
            haekchen: None,
        }
    }

    fn aktiv(mut self, aktiv: bool) -> Self {
        self.aktiv = aktiv;
        self
    }

    fn haekchen(mut self, an: bool) -> Self {
        self.haekchen = Some(an);
        self
    }
}

/// Das ganze Menü in fester Reihenfolge; Trenner setzt `tray.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menuemodell {
    /// ganz oben, nur bei Fehlern
    pub fehler: Vec<Eintrag>,
    pub primaeraktion: Eintrag,
    pub ueberspringen: Eintrag,
    pub phase_zuruecksetzen: Eintrag,
    pub neu_beginnen: Eintrag,
    pub immer_im_vordergrund: Eintrag,
    pub click_through: Eintrag,
    pub overlay_anzeigen: Eintrag,
    pub ton: Eintrag,
    /// *Dunkel*, *Hell*
    pub eingebaute_themes: Vec<Eintrag>,
    /// alphabetisch nach Name
    pub eigene_themes: Vec<Eintrag>,
    pub theme_ordner_oeffnen: Eintrag,
    pub einstellungen: Eintrag,
    pub beenden: Eintrag,
}

/// Baut das Menü aus dem Stand. `einstellungsfehler` sind fertige Texte aus der
/// Einstellungsdatei, `aktives_theme` ist der Schlüssel des tatsächlich verwendeten Themes.
pub fn menue(
    stand: &Stand,
    schalter: Schalterstand,
    einstellungsfehler: &[String],
    laufzeitfehler: &[Laufzeitfehler],
    themes: &[Themeeintrag],
    aktives_theme: &str,
) -> Menuemodell {
    let bereit_ohne_ueberzeit = stand.zustand == Zustand::Bereit && stand.ueberzeit.is_none();
    let am_zyklusanfang = bereit_ohne_ueberzeit
        && stand.phase == Phase::Fokus
        && stand.fokusphase == 1
        && stand.erledigt == 0;
    let primaer = match stand.zustand {
        Zustand::Bereit => "Starten",
        Zustand::Laeuft => "Anhalten",
        Zustand::Angehalten => "Fortsetzen",
    };
    let umschalten =
        |text: &str, s: Schalter, an: bool| Eintrag::neu(text, Aktion::Umschalten(s)).haekchen(an);
    let mut fehler = Vec::new();
    if let Some((erster, weitere)) = einstellungsfehler.split_first() {
        let zusatz = if weitere.is_empty() {
            String::new()
        } else {
            format!(" (+{} weitere)", weitere.len())
        };
        fehler.push(Eintrag::neu(
            format!("⚠ Einstellungen: {erster}{zusatz}"),
            Aktion::EinstellungsdateiOeffnen,
        ));
    }
    fehler.extend(
        laufzeitfehler
            .iter()
            .map(|f| Eintrag::neu(format!("⚠ {}", f.text), Aktion::FehlerAusblenden(f.id))),
    );
    Menuemodell {
        fehler,
        primaeraktion: Eintrag::neu(primaer, Aktion::Primaeraktion),
        ueberspringen: Eintrag::neu("Überspringen", Aktion::Ueberspringen),
        phase_zuruecksetzen: Eintrag::neu("Phase zurücksetzen", Aktion::PhaseZuruecksetzen)
            .aktiv(!bereit_ohne_ueberzeit),
        neu_beginnen: Eintrag::neu("Neu beginnen", Aktion::NeuBeginnen).aktiv(!am_zyklusanfang),
        immer_im_vordergrund: umschalten(
            "Immer im Vordergrund",
            Schalter::ImmerImVordergrund,
            schalter.immer_im_vordergrund,
        ),
        click_through: umschalten(
            "Click-through",
            Schalter::ClickThrough,
            schalter.click_through,
        ),
        overlay_anzeigen: umschalten(
            "Overlay anzeigen",
            Schalter::OverlayAnzeigen,
            schalter.overlay_anzeigen,
        ),
        ton: umschalten("Ton", Schalter::Ton, schalter.ton),
        eingebaute_themes: theme::eingebaute()
            .into_iter()
            .map(|(schluessel, t)| {
                Eintrag::neu(t.name, Aktion::ThemeWaehlen(schluessel.into()))
                    .haekchen(schluessel == aktives_theme)
            })
            .collect(),
        eigene_themes: themes
            .iter()
            .map(|t| theme_eintrag(t, aktives_theme))
            .collect(),
        theme_ordner_oeffnen: Eintrag::neu("Theme-Ordner öffnen", Aktion::ThemeOrdnerOeffnen),
        einstellungen: Eintrag::neu("Einstellungen …", Aktion::EinstellungsfensterOeffnen),
        beenden: Eintrag::neu("Beenden", Aktion::Beenden),
    }
}

fn theme_eintrag(t: &Themeeintrag, aktives_theme: &str) -> Eintrag {
    let (zusatz, waehlbar) = match t.zustand {
        Themezustand::Syntaxfehler { zeile } => (format!(" (Fehler in Zeile {zeile})"), false),
        Themezustand::Reserviert => (" (Name reserviert)".to_owned(), false),
        Themezustand::Gueltig if t.bild_fehlt => (" (Bild fehlt)".to_owned(), true),
        Themezustand::Gueltig => (String::new(), true),
    };
    Eintrag::neu(
        format!("{}{zusatz}", t.name),
        Aktion::ThemeWaehlen(t.schluessel.clone()),
    )
    .aktiv(waehlbar)
    .haekchen(waehlbar && t.schluessel == aktives_theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn bereit_am_anfang() -> Stand {
        Stand {
            phase: Phase::Fokus,
            zustand: Zustand::Bereit,
            fokusphase: 1,
            erledigt: 0,
            fokusphasen_im_zyklus: 4,
            rest: Duration::from_secs(1500),
            ueberzeit: None,
            anteil: 0.0,
        }
    }

    fn schalter() -> Schalterstand {
        Schalterstand {
            immer_im_vordergrund: true,
            click_through: false,
            overlay_anzeigen: true,
            ton: true,
        }
    }

    fn modell(stand: &Stand) -> Menuemodell {
        menue(stand, schalter(), &[], &[], &[], "dunkel")
    }

    #[test]
    fn primaeraktion_heisst_je_nach_zustand_anders_und_ist_nie_ausgegraut() {
        let laeuft = Stand {
            zustand: Zustand::Laeuft,
            ..bereit_am_anfang()
        };
        let angehalten = Stand {
            zustand: Zustand::Angehalten,
            ..bereit_am_anfang()
        };

        for (stand, text) in [
            (bereit_am_anfang(), "Starten"),
            (laeuft, "Anhalten"),
            (angehalten, "Fortsetzen"),
        ] {
            let m = modell(&stand);
            assert_eq!(
                (m.primaeraktion.text.as_str(), m.primaeraktion.aktiv),
                (text, true)
            );
            assert_eq!(m.primaeraktion.aktion, Aktion::Primaeraktion);
            assert!(m.ueberspringen.aktiv);
        }
    }

    #[test]
    fn am_zyklusanfang_sind_zuruecksetzen_und_neu_beginnen_ausgegraut() {
        let m = modell(&bereit_am_anfang());

        assert!(!m.phase_zuruecksetzen.aktiv);
        assert!(!m.neu_beginnen.aktiv);
    }

    #[test]
    fn bereit_mit_ueberzeit_laesst_beides_zu() {
        let stand = Stand {
            ueberzeit: Some(Duration::from_secs(3)),
            ..bereit_am_anfang()
        };
        let m = modell(&stand);

        assert!(m.phase_zuruecksetzen.aktiv);
        assert!(m.neu_beginnen.aktiv);
    }

    #[test]
    fn spaeter_im_zyklus_bereit_graut_nur_zuruecksetzen_aus() {
        let stand = Stand {
            fokusphase: 2,
            erledigt: 1,
            ..bereit_am_anfang()
        };
        let m = modell(&stand);

        assert!(!m.phase_zuruecksetzen.aktiv);
        assert!(m.neu_beginnen.aktiv);
    }

    #[test]
    fn laufend_oder_angehalten_ist_beides_aktiv() {
        for zustand in [Zustand::Laeuft, Zustand::Angehalten] {
            let m = modell(&Stand {
                zustand,
                ..bereit_am_anfang()
            });
            assert!(m.phase_zuruecksetzen.aktiv);
            assert!(m.neu_beginnen.aktiv);
        }
    }

    #[test]
    fn haekchen_folgen_dem_schalterstand() {
        let s = Schalterstand {
            immer_im_vordergrund: false,
            click_through: true,
            overlay_anzeigen: false,
            ton: true,
        };
        let m = menue(&bereit_am_anfang(), s, &[], &[], &[], "dunkel");

        assert_eq!(m.immer_im_vordergrund.haekchen, Some(false));
        assert_eq!(m.click_through.haekchen, Some(true));
        assert_eq!(m.overlay_anzeigen.haekchen, Some(false));
        assert_eq!(m.ton.haekchen, Some(true));
        assert_eq!(m.ton.aktion, Aktion::Umschalten(Schalter::Ton));
        assert_eq!(m.primaeraktion.haekchen, None);
    }

    fn fehlertexte(einstellungen: &[&str], laufzeit: &[Laufzeitfehler]) -> Vec<(String, Aktion)> {
        let einstellungen: Vec<String> = einstellungen.iter().map(|s| s.to_string()).collect();
        menue(
            &bereit_am_anfang(),
            schalter(),
            &einstellungen,
            laufzeit,
            &[],
            "dunkel",
        )
        .fehler
        .into_iter()
        .map(|e| (e.text, e.aktion))
        .collect()
    }

    #[test]
    fn ohne_fehler_gibt_es_keine_fehlerzeile() {
        assert_eq!(fehlertexte(&[], &[]), vec![]);
    }

    #[test]
    fn ein_fehler_der_einstellungsdatei_oeffnet_die_datei() {
        assert_eq!(
            fehlertexte(&["fokus_minuten = 500 ungültig, 25 verwendet"], &[]),
            vec![(
                "⚠ Einstellungen: fokus_minuten = 500 ungültig, 25 verwendet".to_string(),
                Aktion::EinstellungsdateiOeffnen
            )]
        );
    }

    #[test]
    fn mehrere_fehler_der_einstellungsdatei_zeigen_den_ersten_mit_anzahl() {
        assert_eq!(
            fehlertexte(
                &[
                    "Fehler in Zeile 4",
                    "unbekannter Schlüssel farbe",
                    "ton ungültig"
                ],
                &[]
            ),
            vec![(
                "⚠ Einstellungen: Fehler in Zeile 4 (+2 weitere)".to_string(),
                Aktion::EinstellungsdateiOeffnen
            )]
        );
    }

    #[test]
    fn laufzeitfehler_stehen_als_eigene_zeilen_und_lassen_sich_ausblenden() {
        let laufzeit = [
            Laufzeitfehler {
                id: 7,
                text: "Autostart: Eintrag konnte nicht gesetzt werden".into(),
            },
            Laufzeitfehler {
                id: 9,
                text: "Einstellungen: Datei konnte nicht geschrieben werden".into(),
            },
        ];

        assert_eq!(
            fehlertexte(&["Fehler in Zeile 4"], &laufzeit),
            vec![
                (
                    "⚠ Einstellungen: Fehler in Zeile 4".to_string(),
                    Aktion::EinstellungsdateiOeffnen
                ),
                (
                    "⚠ Autostart: Eintrag konnte nicht gesetzt werden".to_string(),
                    Aktion::FehlerAusblenden(7)
                ),
                (
                    "⚠ Einstellungen: Datei konnte nicht geschrieben werden".to_string(),
                    Aktion::FehlerAusblenden(9)
                ),
            ]
        );
    }

    fn eigenes(
        schluessel: &str,
        name: &str,
        zustand: Themezustand,
        bild_fehlt: bool,
    ) -> Themeeintrag {
        Themeeintrag {
            schluessel: schluessel.into(),
            name: name.into(),
            zustand,
            bild_fehlt,
        }
    }

    /// (Text, aktiv, Häkchen, gewählter Schlüssel)
    fn kurz(eintraege: &[Eintrag]) -> Vec<(&str, bool, Option<bool>, &str)> {
        eintraege
            .iter()
            .map(|e| match &e.aktion {
                Aktion::ThemeWaehlen(s) => (e.text.as_str(), e.aktiv, e.haekchen, s.as_str()),
                andere => panic!("unerwartete Aktion {andere:?}"),
            })
            .collect()
    }

    #[test]
    fn theme_untermenue_mit_haekchen_und_zusaetzen_in_der_reihenfolge_der_liste() {
        let themes = [
            eigenes("zebra", "aqua", Themezustand::Gueltig, false),
            eigenes(
                "kaputt",
                "kaputt",
                Themezustand::Syntaxfehler { zeile: 4 },
                false,
            ),
            eigenes("hell", "Mein Hell", Themezustand::Reserviert, false),
            eigenes("pipboy", "Pip-Boy", Themezustand::Gueltig, true),
        ];

        let m = menue(&bereit_am_anfang(), schalter(), &[], &[], &themes, "pipboy");

        assert_eq!(
            kurz(&m.eingebaute_themes),
            vec![
                ("Dunkel", true, Some(false), "dunkel"),
                ("Hell", true, Some(false), "hell")
            ]
        );
        assert_eq!(
            kurz(&m.eigene_themes),
            vec![
                ("aqua", true, Some(false), "zebra"),
                ("kaputt (Fehler in Zeile 4)", false, Some(false), "kaputt"),
                ("Mein Hell (Name reserviert)", false, Some(false), "hell"),
                ("Pip-Boy (Bild fehlt)", true, Some(true), "pipboy"),
            ]
        );
    }

    #[test]
    fn das_aktive_eingebaute_theme_hat_das_haekchen() {
        let m = menue(&bereit_am_anfang(), schalter(), &[], &[], &[], "hell");

        assert_eq!(
            kurz(&m.eingebaute_themes),
            vec![
                ("Dunkel", true, Some(false), "dunkel"),
                ("Hell", true, Some(true), "hell")
            ]
        );
    }
}
