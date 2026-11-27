//! Hinweis bei Phasenwechsel (SPEC 6): welcher Klang, ob eingeblendet wird, Aufblinken.

use std::time::{Duration, Instant};

use crate::ablauf::Phase;

/// Gesamtdauer des Aufblinkens (drei gleich lange Pulse).
const BLINKDAUER: Duration = Duration::from_secs(2);
const PULSE: f32 = 3.0;
/// Bildabstand, solange es blinkt.
const BILDABSTAND: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Klang {
    FokusEndet,
    PauseEndet,
}

/// Die Schalter aus den Einstellungen, die den Hinweis steuern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hinweisschalter {
    pub ton: bool,
    pub signal_im_overlay: bool,
    /// *Overlay bei Phasenwechsel einblenden*
    pub einblenden: bool,
}

/// Was die Schale beim Hinweis tun soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hinweisaktionen {
    pub klang: Option<Klang>,
    pub einblenden: bool,
}

#[derive(Debug, Default)]
pub struct Hinweis {
    blinken_seit: Option<Instant>,
}

impl Hinweis {
    /// Beim Ereignis `PhaseAbgelaufen`. `beendet` ist die abgelaufene Phase,
    /// `stoeren_erlaubt` kommt aus `SHQueryUserNotificationState`.
    pub fn ausloesen(
        &mut self,
        beendet: Phase,
        schalter: Hinweisschalter,
        stoeren_erlaubt: bool,
        jetzt: Instant,
    ) -> Hinweisaktionen {
        if schalter.signal_im_overlay {
            self.blinken_starten(jetzt);
        }
        let klang = match beendet {
            Phase::Fokus => Klang::FokusEndet,
            Phase::KurzePause | Phase::LangePause => Klang::PauseEndet,
        };
        Hinweisaktionen {
            klang: (schalter.ton && stoeren_erlaubt).then_some(klang),
            einblenden: schalter.einblenden,
        }
    }

    /// Für den zweiten Start: blinkt unabhängig von *Signal im Overlay*.
    pub fn blinken_starten(&mut self, jetzt: Instant) {
        self.blinken_seit = Some(jetzt);
    }

    /// Stärke des Aufblinkens `0.0..=1.0`, außerhalb der 2 s genau 0.
    pub fn aufblinken(&self, jetzt: Instant) -> f32 {
        let Some(verstrichen) = self.verstrichen(jetzt) else {
            return 0.0;
        };
        let im_puls = (verstrichen.as_secs_f32() / BLINKDAUER.as_secs_f32() * PULSE).fract();
        (im_puls * std::f32::consts::PI).sin().powi(2)
    }

    pub fn naechste_faelligkeit(&self, jetzt: Instant) -> Option<Instant> {
        let verstrichen = self.verstrichen(jetzt)?;
        Some(jetzt + BILDABSTAND.min(BLINKDAUER - verstrichen))
    }

    /// Zeit seit Beginn des Aufblinkens, solange es noch blinkt.
    fn verstrichen(&self, jetzt: Instant) -> Option<Duration> {
        let verstrichen = jetzt.saturating_duration_since(self.blinken_seit?);
        (verstrichen < BLINKDAUER).then_some(verstrichen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alles_an() -> Hinweisschalter {
        Hinweisschalter {
            ton: true,
            signal_im_overlay: true,
            einblenden: true,
        }
    }

    #[test]
    fn klang_richtet_sich_nach_der_beendeten_phase() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();

        let fokus = hinweis.ausloesen(Phase::Fokus, alles_an(), true, t0);
        let kurz = hinweis.ausloesen(Phase::KurzePause, alles_an(), true, t0);
        let lang = hinweis.ausloesen(Phase::LangePause, alles_an(), true, t0);

        assert_eq!(fokus.klang, Some(Klang::FokusEndet));
        assert_eq!(kurz.klang, Some(Klang::PauseEndet));
        assert_eq!(lang.klang, Some(Klang::PauseEndet));
    }

    #[test]
    fn kein_klang_bei_ton_aus_oder_wenn_stoeren_nicht_erlaubt() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();
        let ton_aus = Hinweisschalter {
            ton: false,
            ..alles_an()
        };

        assert_eq!(
            hinweis.ausloesen(Phase::Fokus, ton_aus, true, t0).klang,
            None
        );
        assert_eq!(
            hinweis.ausloesen(Phase::Fokus, alles_an(), false, t0).klang,
            None
        );
    }

    #[test]
    fn eingeblendet_wird_nur_mit_schalter() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();
        let aus = Hinweisschalter {
            einblenden: false,
            ..alles_an()
        };

        assert!(
            hinweis
                .ausloesen(Phase::Fokus, alles_an(), true, t0)
                .einblenden
        );
        assert!(!hinweis.ausloesen(Phase::Fokus, aus, true, t0).einblenden);
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn nahe(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn aufblinken_dauert_zwei_sekunden_mit_drei_pulsen() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();
        assert_eq!(hinweis.aufblinken(t0), 0.0);

        hinweis.ausloesen(Phase::Fokus, alles_an(), true, t0);

        // Spitzen in der Mitte jedes Pulses (Pulslänge 2/3 s)
        for spitze in [333, 1000, 1667] {
            assert!(
                nahe(hinweis.aufblinken(t0 + ms(spitze)), 1.0),
                "Spitze bei {spitze} ms"
            );
        }
        // Täler zwischen den Pulsen
        for tal in [0, 667, 1333] {
            assert!(hinweis.aufblinken(t0 + ms(tal)) < 0.01, "Tal bei {tal} ms");
        }
        assert!((hinweis.aufblinken(t0 + ms(167)) - 0.5).abs() < 0.01);
        assert_eq!(hinweis.aufblinken(t0 + ms(2000)), 0.0);
        assert_eq!(hinweis.aufblinken(t0 + ms(5000)), 0.0);
    }

    #[test]
    fn ohne_signal_im_overlay_blinkt_nichts() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();
        let aus = Hinweisschalter {
            signal_im_overlay: false,
            ..alles_an()
        };

        hinweis.ausloesen(Phase::Fokus, aus, true, t0);

        assert_eq!(hinweis.aufblinken(t0 + ms(333)), 0.0);
        assert_eq!(hinweis.naechste_faelligkeit(t0), None);
    }

    #[test]
    fn blinken_starten_blinkt_auch_ohne_schalter() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();

        hinweis.blinken_starten(t0);

        assert!(nahe(hinweis.aufblinken(t0 + ms(333)), 1.0));
    }

    #[test]
    fn waehrend_des_blinkens_ist_etwa_alle_16_ms_etwas_faellig() {
        let t0 = Instant::now();
        let mut hinweis = Hinweis::default();
        assert_eq!(hinweis.naechste_faelligkeit(t0), None);

        hinweis.blinken_starten(t0);

        assert_eq!(
            hinweis.naechste_faelligkeit(t0 + ms(100)),
            Some(t0 + ms(116))
        );
        // zum Schluss genau das Ende, damit das letzte Bild wieder normal ist
        assert_eq!(
            hinweis.naechste_faelligkeit(t0 + ms(1990)),
            Some(t0 + ms(2000))
        );
        assert_eq!(hinweis.naechste_faelligkeit(t0 + ms(2000)), None);
    }
}
