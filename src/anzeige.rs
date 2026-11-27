//! Was Overlay und Tray zeigen (SPEC 4.5, 5.3, 5.4): ein fertiger, rein darstellender Datensatz.

use std::time::Duration;

use crate::ablauf::{Phase, Stand, Zustand};
use crate::theme::{Farbe, Theme};

/// Anteil, zu dem eine gedimmte Farbe mit dem Hintergrund gemischt wird (fest, SPEC 5.4).
const DIMMEN: f32 = 0.5;
/// Wie stark der Hintergrund bei voller Blinkstärke zur Akzentfarbe geht (Richtwert).
const BLINKSTAERKE: f32 = 0.6;

/// Woran das Tray-Icon hängt; es wird nur bei Änderung neu gesetzt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trayschluessel {
    pub phase: Phase,
    pub gedimmt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Keins,
    Pause,
    Start,
}

/// Ein Zyklus-Punkt je Fokusphase des aktuellen Zyklus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punkt {
    /// erledigt, in `vordergrund`
    Voll,
    /// aktuelle Fokusphase: Ring in der Akzentfarbe
    Hervorgehoben,
    /// offen, in `gedaempft`
    Leer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Anzeige {
    /// `24:13` bzw. `+01:05`
    pub zeit: String,
    pub symbol: Symbol,
    pub phasenwort: &'static str,
    /// Zeit, Symbole, volle Zyklus-Punkte; schon gedimmt
    pub vordergrund: Farbe,
    /// Akzentfarbe der aktuellen Phase; schon gedimmt
    pub akzent: Farbe,
    /// leere Zyklus-Punkte, Spur des Balkens; schon gedimmt
    pub gedaempft: Farbe,
    /// verstrichener Anteil `0.0..=1.0`
    pub fortschritt: f32,
    pub punkte: Vec<Punkt>,
    /// Fläche des Overlays mit Deckkraft, beim Aufblinken schon gemischt
    pub hintergrund: Farbe,
    /// Bei einem Hintergrundbild darüber zu zeichnen; Alpha 0 außerhalb des Aufblinkens
    pub blinkschicht: Farbe,
    /// Tray-Tooltip, ggf. mit zweiter Zeile für Fehler
    pub tooltip: String,
    pub tray: Trayschluessel,
}

pub fn anzeige(stand: &Stand, theme: &Theme, aufblinken: f32, einstellungsfehler: bool) -> Anzeige {
    let f = &theme.farben;
    let gedimmt = match stand.zustand {
        Zustand::Laeuft => false,
        Zustand::Angehalten => true,
        Zustand::Bereit => stand.ueberzeit.is_none(),
    };
    let dimmen = |farbe: Farbe| {
        if gedimmt {
            mischen(farbe, f.hintergrund, DIMMEN)
        } else {
            farbe
        }
    };
    let (phasenwort, akzent) = match stand.phase {
        Phase::Fokus => ("Fokus", f.fokus),
        Phase::KurzePause => ("Pause", f.kurze_pause),
        Phase::LangePause => ("Lange Pause", f.lange_pause),
    };
    let zeit = zeittext(stand);
    let blinken = aufblinken.clamp(0.0, 1.0) * BLINKSTAERKE;
    Anzeige {
        hintergrund: mischen(f.hintergrund, akzent, blinken),
        blinkschicht: Farbe {
            a: (blinken * 255.0).round() as u8,
            ..akzent
        },
        tray: Trayschluessel {
            phase: stand.phase,
            gedimmt,
        },
        tooltip: tooltip(stand, &zeit, einstellungsfehler),
        zeit,
        symbol: match stand.zustand {
            Zustand::Laeuft => Symbol::Keins,
            Zustand::Angehalten => Symbol::Pause,
            Zustand::Bereit => Symbol::Start,
        },
        phasenwort,
        vordergrund: dimmen(f.vordergrund),
        akzent: dimmen(akzent),
        gedaempft: dimmen(f.gedaempft),
        fortschritt: stand.anteil,
        punkte: (1..=stand.fokusphasen_im_zyklus)
            .map(|i| {
                if i <= stand.erledigt {
                    Punkt::Voll
                } else if stand.phase == Phase::Fokus && i == stand.fokusphase {
                    Punkt::Hervorgehoben
                } else {
                    Punkt::Leer
                }
            })
            .collect(),
    }
}

/// Mischt die RGB-Werte von `zu` mit dem Anteil `anteil` in `farbe`; Alpha bleibt.
fn mischen(farbe: Farbe, zu: Farbe, anteil: f32) -> Farbe {
    let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * anteil).round() as u8;
    Farbe {
        r: m(farbe.r, zu.r),
        g: m(farbe.g, zu.g),
        b: m(farbe.b, zu.b),
        a: farbe.a,
    }
}

fn tooltip(stand: &Stand, zeit: &str, einstellungsfehler: bool) -> String {
    let n = stand.fokusphasen_im_zyklus;
    let phase = match stand.phase {
        Phase::Fokus => format!("Fokus {}/{n}", stand.fokusphase),
        Phase::KurzePause => format!("Pause nach {}/{n}", stand.fokusphase),
        Phase::LangePause => "Lange Pause".to_owned(),
    };
    let zustand = match stand.zustand {
        Zustand::Laeuft => "läuft",
        Zustand::Angehalten => "angehalten",
        Zustand::Bereit => "bereit",
    };
    let fehler = if einstellungsfehler {
        "
⚠ Fehler in den Einstellungen"
    } else {
        ""
    };
    format!("{phase} · {zustand} · {zeit}{fehler}")
}

fn zeittext(stand: &Stand) -> String {
    let (vorzeichen, sekunden) = match stand.ueberzeit {
        Some(ueberzeit) => ("+", ueberzeit.as_secs()),
        None => ("", aufgerundet(stand.rest)),
    };
    format!("{vorzeichen}{:02}:{:02}", sekunden / 60, sekunden % 60)
}

fn aufgerundet(dauer: Duration) -> u64 {
    dauer.as_secs() + u64::from(dauer.subsec_nanos() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::dunkel;

    fn sekunden(s: f64) -> Duration {
        Duration::from_secs_f64(s)
    }

    fn laeuft(rest: Duration) -> Stand {
        Stand {
            phase: Phase::Fokus,
            zustand: Zustand::Laeuft,
            fokusphase: 1,
            erledigt: 0,
            fokusphasen_im_zyklus: 4,
            rest,
            ueberzeit: None,
            anteil: 0.0,
        }
    }

    fn bereit_mit_ueberzeit(ueberzeit: Duration) -> Stand {
        Stand {
            zustand: Zustand::Bereit,
            rest: sekunden(1500.0),
            ueberzeit: Some(ueberzeit),
            ..laeuft(Duration::ZERO)
        }
    }

    fn zeit(stand: Stand) -> String {
        anzeige(&stand, &dunkel(), 0.0, false).zeit
    }

    #[test]
    fn restzeit_wird_aufgerundet() {
        assert_eq!(zeit(laeuft(sekunden(1500.0))), "25:00");
        assert_eq!(zeit(laeuft(sekunden(1499.1))), "25:00");
        assert_eq!(zeit(laeuft(sekunden(1499.0))), "24:59");
        assert_eq!(zeit(laeuft(sekunden(0.001))), "00:01");
        assert_eq!(zeit(laeuft(Duration::ZERO)), "00:00");
    }

    #[test]
    fn minuten_duerfen_mehr_als_zwei_stellen_haben() {
        assert_eq!(zeit(laeuft(sekunden(180.0 * 60.0))), "180:00");
    }

    #[test]
    fn ueberzeit_wird_abgerundet_und_mit_plus_angezeigt() {
        assert_eq!(zeit(bereit_mit_ueberzeit(Duration::ZERO)), "+00:00");
        assert_eq!(zeit(bereit_mit_ueberzeit(sekunden(0.999))), "+00:00");
        assert_eq!(zeit(bereit_mit_ueberzeit(sekunden(65.5))), "+01:05");
        assert_eq!(
            zeit(bereit_mit_ueberzeit(sekunden(125.0 * 60.0 + 30.0))),
            "+125:30"
        );
    }

    // Dunkel, normal und zu 50 % mit #1E1E1E gemischt
    const VORDERGRUND: Farbe = Farbe::rgb(0xF0, 0xF0, 0xF0);
    const VORDERGRUND_GEDIMMT: Farbe = Farbe::rgb(0x87, 0x87, 0x87);
    const FOKUS: Farbe = Farbe::rgb(0xE5, 0x53, 0x4B);
    const FOKUS_GEDIMMT: Farbe = Farbe::rgb(0x82, 0x39, 0x35);

    #[test]
    fn laeuft_zeigt_normal_ohne_symbol_mit_fortschritt() {
        let stand = Stand {
            anteil: 0.4,
            ..laeuft(sekunden(900.0))
        };
        let a = anzeige(&stand, &dunkel(), 0.0, false);

        assert_eq!(a.symbol, Symbol::Keins);
        assert_eq!(a.phasenwort, "Fokus");
        assert_eq!((a.vordergrund, a.akzent), (VORDERGRUND, FOKUS));
        assert_eq!(a.fortschritt, 0.4);
    }

    #[test]
    fn angehalten_ist_gedimmt_mit_pausensymbol_und_fortschritt() {
        let stand = Stand {
            zustand: Zustand::Angehalten,
            anteil: 0.4,
            ..laeuft(sekunden(900.0))
        };
        let a = anzeige(&stand, &dunkel(), 0.0, false);

        assert_eq!(a.symbol, Symbol::Pause);
        assert_eq!(
            (a.vordergrund, a.akzent),
            (VORDERGRUND_GEDIMMT, FOKUS_GEDIMMT)
        );
        assert_eq!(a.fortschritt, 0.4);
    }

    #[test]
    fn bereit_ohne_ueberzeit_ist_gedimmt_mit_startsymbol_und_leerem_balken() {
        let stand = Stand {
            zustand: Zustand::Bereit,
            ..laeuft(sekunden(1500.0))
        };
        let a = anzeige(&stand, &dunkel(), 0.0, false);

        assert_eq!(a.zeit, "25:00");
        assert_eq!(a.symbol, Symbol::Start);
        assert_eq!(
            (a.vordergrund, a.akzent),
            (VORDERGRUND_GEDIMMT, FOKUS_GEDIMMT)
        );
        assert_eq!(a.fortschritt, 0.0);
    }

    #[test]
    fn bereit_mit_ueberzeit_ist_nicht_gedimmt() {
        let stand = Stand {
            phase: Phase::LangePause,
            ..bereit_mit_ueberzeit(sekunden(7.0))
        };
        let a = anzeige(&stand, &dunkel(), 0.0, false);

        assert_eq!(a.symbol, Symbol::Start);
        assert_eq!(a.phasenwort, "Lange Pause");
        assert_eq!(
            (a.vordergrund, a.akzent),
            (VORDERGRUND, Farbe::rgb(0x42, 0xA5, 0xF5))
        );
        assert_eq!(a.fortschritt, 0.0);
    }

    #[test]
    fn kurze_pause_heisst_pause_in_ihrer_akzentfarbe() {
        let stand = Stand {
            phase: Phase::KurzePause,
            ..laeuft(sekunden(60.0))
        };
        let a = anzeige(&stand, &dunkel(), 0.0, false);

        assert_eq!(
            (a.phasenwort, a.akzent),
            ("Pause", Farbe::rgb(0x4C, 0xAF, 0x50))
        );
    }

    /// Schreibweise aus SPEC 5.4: ● voll, ◐ hervorgehoben, ○ leer
    fn punkte(phase: Phase, fokusphase: u8, erledigt: u8, im_zyklus: u8) -> String {
        let stand = Stand {
            phase,
            fokusphase,
            erledigt,
            fokusphasen_im_zyklus: im_zyklus,
            ..laeuft(sekunden(60.0))
        };
        anzeige(&stand, &dunkel(), 0.0, false)
            .punkte
            .iter()
            .map(|p| match p {
                Punkt::Voll => '●',
                Punkt::Hervorgehoben => '◐',
                Punkt::Leer => '○',
            })
            .collect()
    }

    #[test]
    fn zyklus_punkte_wie_in_den_beispielen_der_spec() {
        assert_eq!(punkte(Phase::Fokus, 1, 0, 4), "◐○○○");
        assert_eq!(punkte(Phase::KurzePause, 2, 2, 4), "●●○○");
        assert_eq!(punkte(Phase::Fokus, 3, 2, 4), "●●◐○");
        assert_eq!(punkte(Phase::LangePause, 4, 4, 4), "●●●●");
        assert_eq!(punkte(Phase::Fokus, 12, 11, 12), "●●●●●●●●●●●◐");
    }

    fn tooltip(stand: Stand, einstellungsfehler: bool) -> String {
        anzeige(&stand, &dunkel(), 0.0, einstellungsfehler).tooltip
    }

    #[test]
    fn tooltip_texte_wie_in_der_spec() {
        let fokus_3 = Stand {
            fokusphase: 3,
            erledigt: 2,
            ..laeuft(sekunden(754.0))
        };
        let pause_nach_2 = Stand {
            phase: Phase::KurzePause,
            zustand: Zustand::Angehalten,
            fokusphase: 2,
            erledigt: 2,
            ..laeuft(sekunden(190.0))
        };
        let lange_pause = Stand {
            phase: Phase::LangePause,
            ..bereit_mit_ueberzeit(sekunden(420.0))
        };
        let start = Stand {
            zustand: Zustand::Bereit,
            ..laeuft(sekunden(1500.0))
        };

        assert_eq!(tooltip(fokus_3, false), "Fokus 3/4 · läuft · 12:34");
        assert_eq!(
            tooltip(pause_nach_2, false),
            "Pause nach 2/4 · angehalten · 03:10"
        );
        assert_eq!(tooltip(lange_pause, false), "Lange Pause · bereit · +07:00");
        assert_eq!(tooltip(start, false), "Fokus 1/4 · bereit · 25:00");
    }

    #[test]
    fn tooltip_hat_bei_fehlern_eine_zweite_zeile_und_bleibt_unter_128_zeichen() {
        let stand = Stand {
            fokusphase: 12,
            fokusphasen_im_zyklus: 12,
            ..laeuft(sekunden(180.0 * 60.0))
        };

        let text = tooltip(stand, true);

        assert_eq!(
            text,
            "Fokus 12/12 · läuft · 180:00\n⚠ Fehler in den Einstellungen"
        );
        assert!(text.encode_utf16().count() < 128);
    }

    #[test]
    fn aufblinken_mischt_den_hintergrund_zur_akzentfarbe_der_neuen_phase() {
        let stand = Stand {
            phase: Phase::KurzePause,
            ..laeuft(sekunden(300.0))
        };

        let ruhig = anzeige(&stand, &dunkel(), 0.0, false);
        let voll = anzeige(&stand, &dunkel(), 1.0, false);

        assert_eq!(ruhig.hintergrund, Farbe::rgba(0x1E, 0x1E, 0x1E, 0xD9));
        assert_eq!(ruhig.blinkschicht.a, 0);
        // 60 % von #1E1E1E zu #4CAF50, Deckkraft bleibt
        assert_eq!(voll.hintergrund, Farbe::rgba(0x3A, 0x75, 0x3C, 0xD9));
        // für ein Hintergrundbild: dieselbe Mischung als Schicht darüber
        assert_eq!(voll.blinkschicht, Farbe::rgba(0x4C, 0xAF, 0x50, 153));
    }

    #[test]
    fn aufblinken_nimmt_die_volle_akzentfarbe_auch_wenn_gedimmt() {
        let stand = Stand {
            zustand: Zustand::Bereit,
            ..laeuft(sekunden(1500.0))
        };

        let a = anzeige(&stand, &dunkel(), 1.0, false);

        assert_eq!(a.blinkschicht, Farbe::rgba(0xE5, 0x53, 0x4B, 153));
    }

    #[test]
    fn tray_schluessel_aus_phase_und_gedimmt() {
        let laufend = anzeige(&laeuft(sekunden(60.0)), &dunkel(), 0.0, false);
        let angehalten = Stand {
            zustand: Zustand::Angehalten,
            ..laeuft(sekunden(60.0))
        };
        let ueberzeit = Stand {
            phase: Phase::LangePause,
            ..bereit_mit_ueberzeit(sekunden(1.0))
        };

        assert_eq!(
            laufend.tray,
            Trayschluessel {
                phase: Phase::Fokus,
                gedimmt: false
            }
        );
        assert!(anzeige(&angehalten, &dunkel(), 0.0, false).tray.gedimmt);
        assert_eq!(
            anzeige(&ueberzeit, &dunkel(), 0.0, false).tray,
            Trayschluessel {
                phase: Phase::LangePause,
                gedimmt: false
            }
        );
    }
}
