//! Reiner Kern des Phasenablaufs (SPEC 3, 11.2). Zeit kommt nur als Parameter herein.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Fokus,
    KurzePause,
    LangePause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zustand {
    Laeuft,
    Angehalten,
    Bereit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Befehl {
    Primaeraktion,
    Ueberspringen,
    PhaseZuruecksetzen,
    NeuBeginnen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ereignis {
    PhaseAbgelaufen { von: Phase, nach: Phase },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ablaufeinstellungen {
    pub fokus: Duration,
    pub kurze_pause: Duration,
    pub lange_pause: Duration,
    pub fokusphasen_pro_zyklus: u8,
    pub pause_automatisch: bool,
    pub fokus_automatisch: bool,
}

/// Der Stand zu einem Zeitpunkt, fertig zum Anzeigen.
#[derive(Debug, Clone, PartialEq)]
pub struct Stand {
    pub phase: Phase,
    pub zustand: Zustand,
    /// 1-basiert: aktuelle bzw. zuletzt beendete Fokusphase
    pub fokusphase: u8,
    /// erledigte Fokusphasen im Zyklus
    pub erledigt: u8,
    pub fokusphasen_im_zyklus: u8,
    /// bei *bereit*: volle Dauer
    pub rest: Duration,
    pub ueberzeit: Option<Duration>,
    /// verstrichener Anteil der Phasendauer, `0.0..=1.0`
    pub anteil: f32,
}

/// Innerer Zustand mit den Zeitpunkten, aus denen alles berechnet wird.
#[derive(Debug, Clone, Copy)]
enum Lage {
    Laeuft {
        ende: Instant,
    },
    Angehalten {
        rest: Duration,
    },
    /// `abgelaufen_um` ist gesetzt, wenn *bereit* durch Ablauf der vorigen Phase entstand.
    Bereit {
        abgelaufen_um: Option<Instant>,
    },
}

pub struct Ablauf {
    lage: Lage,
    einstellungen: Ablaufeinstellungen,
    phase: Phase,
    fokusphase: u8,
    erledigt: u8,
    fokusphasen_im_zyklus: u8,
    dauer: Duration,
}

impl Ablauf {
    pub fn neu(einstellungen: Ablaufeinstellungen) -> Self {
        Self {
            lage: Lage::Bereit {
                abgelaufen_um: None,
            },
            phase: Phase::Fokus,
            fokusphase: 1,
            erledigt: 0,
            fokusphasen_im_zyklus: einstellungen.fokusphasen_pro_zyklus,
            dauer: einstellungen.fokus,
            einstellungen,
        }
    }

    pub fn befehl(&mut self, befehl: Befehl, jetzt: Instant) -> Vec<Ereignis> {
        let ereignisse = self.takt(jetzt);
        match befehl {
            Befehl::Primaeraktion => {
                self.lage = match self.lage {
                    Lage::Bereit { .. } => Lage::Laeuft {
                        ende: jetzt + self.dauer,
                    },
                    Lage::Laeuft { ende } => Lage::Angehalten {
                        rest: ende.saturating_duration_since(jetzt),
                    },
                    Lage::Angehalten { rest } => Lage::Laeuft { ende: jetzt + rest },
                }
            }
            Befehl::Ueberspringen => {
                let automatisch = self.zur_naechsten_phase();
                self.lage = if automatisch {
                    Lage::Laeuft {
                        ende: jetzt + self.dauer,
                    }
                } else {
                    Lage::Bereit {
                        abgelaufen_um: None,
                    }
                };
            }
            Befehl::PhaseZuruecksetzen => {
                self.dauer = self.dauer_fuer(self.phase);
                self.lage = Lage::Bereit {
                    abgelaufen_um: None,
                };
            }
            Befehl::NeuBeginnen => *self = Self::neu(self.einstellungen.clone()),
        }
        ereignisse
    }

    /// Wann sich die Anzeige als Nächstes ändert: nächste volle Sekunde der Restzeit
    /// bzw. Überzeit, höchstens der Ablaufzeitpunkt. `None`, wenn nichts läuft.
    pub fn naechste_faelligkeit(&self, jetzt: Instant) -> Option<Instant> {
        const SEKUNDE: Duration = Duration::from_secs(1);
        match self.lage {
            Lage::Laeuft { ende } => {
                let rest = ende.saturating_duration_since(jetzt);
                if rest.is_zero() {
                    return Some(ende);
                }
                // Restzeit wird aufgerundet angezeigt: Sie springt, wenn der Bruchteil verbraucht ist.
                let bruchteil = Duration::from_nanos((rest.as_nanos() % SEKUNDE.as_nanos()) as u64);
                Some(
                    jetzt
                        + if bruchteil.is_zero() {
                            SEKUNDE
                        } else {
                            bruchteil
                        },
                )
            }
            Lage::Bereit {
                abgelaufen_um: Some(um),
            } => {
                let ueberzeit = jetzt.saturating_duration_since(um);
                Some(um + Duration::from_secs(ueberzeit.as_secs() + 1))
            }
            _ => None,
        }
    }

    /// Übernimmt geänderte Werte nach SPEC 3.5.
    pub fn einstellungen(&mut self, einstellungen: Ablaufeinstellungen) {
        self.einstellungen = einstellungen;
        // *bereit* hat noch nicht begonnen und nimmt die neue Dauer sofort.
        if let Lage::Bereit { .. } = self.lage {
            self.dauer = self.dauer_fuer(self.phase);
            // Ganz am Zyklusanfang gilt auch die Zykluslänge sofort.
            if self.phase == Phase::Fokus && self.fokusphase == 1 && self.erledigt == 0 {
                self.fokusphasen_im_zyklus = self.einstellungen.fokusphasen_pro_zyklus;
            }
        }
    }

    fn dauer_fuer(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Fokus => self.einstellungen.fokus,
            Phase::KurzePause => self.einstellungen.kurze_pause,
            Phase::LangePause => self.einstellungen.lange_pause,
        }
    }

    /// Beliebig oft aufrufbar; liefert höchstens ein Ereignis.
    pub fn takt(&mut self, jetzt: Instant) -> Vec<Ereignis> {
        let mut letztes = None;
        // Nachholen: über automatische Übergänge hinweg mit den echten Ablaufzeitpunkten.
        while let Lage::Laeuft { ende } = self.lage {
            if ende > jetzt {
                break;
            }
            let von = self.phase;
            let automatisch = self.zur_naechsten_phase();
            self.lage = if automatisch {
                Lage::Laeuft {
                    ende: ende + self.dauer,
                }
            } else {
                Lage::Bereit {
                    abgelaufen_um: Some(ende),
                }
            };
            letztes = Some(Ereignis::PhaseAbgelaufen {
                von,
                nach: self.phase,
            });
        }
        letztes.into_iter().collect()
    }

    /// Schaltet Phase, Zyklus-Fortschritt und Dauer weiter. Liefert, ob die neue Phase
    /// nach dem Auto-Start-Schalter dieser Richtung sofort läuft.
    fn zur_naechsten_phase(&mut self) -> bool {
        let e = &self.einstellungen;
        match self.phase {
            Phase::Fokus => {
                self.erledigt += 1;
                self.phase = if self.erledigt >= self.fokusphasen_im_zyklus {
                    Phase::LangePause
                } else {
                    Phase::KurzePause
                };
            }
            Phase::KurzePause => {
                self.fokusphase += 1;
                self.phase = Phase::Fokus;
            }
            Phase::LangePause => {
                self.fokusphase = 1;
                self.erledigt = 0;
                self.fokusphasen_im_zyklus = e.fokusphasen_pro_zyklus;
                self.phase = Phase::Fokus;
            }
        }
        self.dauer = self.dauer_fuer(self.phase);
        match self.phase {
            Phase::Fokus => self.einstellungen.fokus_automatisch,
            _ => self.einstellungen.pause_automatisch,
        }
    }

    pub fn stand(&self, jetzt: Instant) -> Stand {
        let (zustand, rest) = match self.lage {
            Lage::Laeuft { ende } => (Zustand::Laeuft, ende.saturating_duration_since(jetzt)),
            Lage::Angehalten { rest } => (Zustand::Angehalten, rest),
            Lage::Bereit { .. } => (Zustand::Bereit, self.dauer),
        };
        let anteil = match zustand {
            Zustand::Bereit => 0.0,
            _ => (1.0 - rest.as_secs_f64() / self.dauer.as_secs_f64()).clamp(0.0, 1.0) as f32,
        };
        Stand {
            phase: self.phase,
            zustand,
            fokusphase: self.fokusphase,
            erledigt: self.erledigt,
            fokusphasen_im_zyklus: self.fokusphasen_im_zyklus,
            rest,
            ueberzeit: match self.lage {
                Lage::Bereit {
                    abgelaufen_um: Some(um),
                } => Some(jetzt.saturating_duration_since(um)),
                _ => None,
            },
            anteil,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minuten(m: u64) -> Duration {
        Duration::from_secs(m * 60)
    }

    fn voreinstellungen() -> Ablaufeinstellungen {
        Ablaufeinstellungen {
            fokus: minuten(25),
            kurze_pause: minuten(5),
            lange_pause: minuten(15),
            fokusphasen_pro_zyklus: 4,
            pause_automatisch: true,
            fokus_automatisch: false,
        }
    }

    #[test]
    fn start_ist_fokusphase_1_bereit_mit_voller_dauer_ohne_ueberzeit() {
        let t0 = Instant::now();
        let ablauf = Ablauf::neu(voreinstellungen());

        let stand = ablauf.stand(t0);

        assert_eq!(stand.phase, Phase::Fokus);
        assert_eq!(stand.zustand, Zustand::Bereit);
        assert_eq!(stand.fokusphase, 1);
        assert_eq!(stand.erledigt, 0);
        assert_eq!(stand.fokusphasen_im_zyklus, 4);
        assert_eq!(stand.rest, minuten(25));
        assert_eq!(stand.ueberzeit, None);
        assert_eq!(stand.anteil, 0.0);
    }

    #[test]
    fn starten_anhalten_fortsetzen_haelt_die_restzeit_ueber_takte_und_pausen() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());

        ablauf.befehl(Befehl::Primaeraktion, t0);
        ablauf.takt(t0 + minuten(4));
        let stand = ablauf.stand(t0 + minuten(10));
        assert_eq!(stand.zustand, Zustand::Laeuft);
        assert_eq!(stand.rest, minuten(15));
        assert_eq!(stand.anteil, 0.4);

        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(10));
        ablauf.takt(t0 + minuten(15));
        let stand = ablauf.stand(t0 + minuten(20));
        assert_eq!(stand.zustand, Zustand::Angehalten);
        assert_eq!(stand.rest, minuten(15));
        assert_eq!(stand.anteil, 0.4);

        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(20));
        let stand = ablauf.stand(t0 + minuten(25));
        assert_eq!(stand.zustand, Zustand::Laeuft);
        assert_eq!(stand.rest, minuten(10));
        assert_eq!(stand.phase, Phase::Fokus);
    }

    #[test]
    fn ablauf_der_fokusphase_startet_mit_auto_start_die_kurze_pause() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Primaeraktion, t0);

        assert_eq!(ablauf.takt(t0 + minuten(24)), vec![]);
        let ereignisse = ablauf.takt(t0 + minuten(25));

        assert_eq!(
            ereignisse,
            vec![Ereignis::PhaseAbgelaufen {
                von: Phase::Fokus,
                nach: Phase::KurzePause
            }]
        );
        let stand = ablauf.stand(t0 + minuten(27));
        assert_eq!(stand.phase, Phase::KurzePause);
        assert_eq!(stand.zustand, Zustand::Laeuft);
        assert_eq!(stand.rest, minuten(3));
        assert_eq!(stand.fokusphase, 1);
        assert_eq!(stand.erledigt, 1);
        assert_eq!(ablauf.takt(t0 + minuten(27)), vec![]);
    }

    #[test]
    fn ablauf_der_pause_stellt_ohne_auto_start_die_fokusphase_bereit_mit_ueberzeit() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Primaeraktion, t0);
        ablauf.takt(t0 + minuten(25));

        let ereignisse = ablauf.takt(t0 + minuten(30) + Duration::from_secs(1));

        assert_eq!(
            ereignisse,
            vec![Ereignis::PhaseAbgelaufen {
                von: Phase::KurzePause,
                nach: Phase::Fokus
            }]
        );
        let stand = ablauf.stand(t0 + minuten(37));
        assert_eq!(stand.phase, Phase::Fokus);
        assert_eq!(stand.zustand, Zustand::Bereit);
        assert_eq!(stand.fokusphase, 2);
        assert_eq!(stand.rest, minuten(25));
        assert_eq!(stand.ueberzeit, Some(minuten(7)));
        assert_eq!(stand.anteil, 0.0);
    }

    /// Lässt `schritte` Phasen ablaufen (Auto-Start in beide Richtungen) und
    /// liefert für jede Phase danach (Phase, Fokusphase, erledigt).
    fn phasenfolge(n: u8, schritte: usize) -> Vec<(Phase, u8, u8)> {
        let mut t = Instant::now();
        let mut ablauf = Ablauf::neu(Ablaufeinstellungen {
            fokusphasen_pro_zyklus: n,
            fokus_automatisch: true,
            ..voreinstellungen()
        });
        ablauf.befehl(Befehl::Primaeraktion, t);
        let mut folge = Vec::new();
        for _ in 0..schritte {
            t += ablauf.stand(t).rest;
            assert_eq!(ablauf.takt(t).len(), 1);
            let stand = ablauf.stand(t);
            assert_eq!(stand.zustand, Zustand::Laeuft);
            folge.push((stand.phase, stand.fokusphase, stand.erledigt));
        }
        folge
    }

    #[test]
    fn nach_der_letzten_fokusphase_kommt_die_lange_pause_und_ein_neuer_zyklus() {
        use Phase::*;
        assert_eq!(
            phasenfolge(4, 9),
            vec![
                (KurzePause, 1, 1),
                (Fokus, 2, 1),
                (KurzePause, 2, 2),
                (Fokus, 3, 2),
                (KurzePause, 3, 3),
                (Fokus, 4, 3),
                (LangePause, 4, 4),
                (Fokus, 1, 0),
                (KurzePause, 1, 1),
            ]
        );
    }

    #[test]
    fn bei_einer_fokusphase_pro_zyklus_folgt_immer_die_lange_pause() {
        use Phase::*;
        assert_eq!(
            phasenfolge(1, 4),
            vec![
                (LangePause, 1, 1),
                (Fokus, 1, 0),
                (LangePause, 1, 1),
                (Fokus, 1, 0)
            ]
        );
    }

    #[test]
    fn bei_zwoelf_fokusphasen_pro_zyklus_kommt_die_lange_pause_nach_der_zwoelften() {
        let folge = phasenfolge(12, 24);
        assert_eq!(folge[20], (Phase::KurzePause, 11, 11));
        assert_eq!(folge[21], (Phase::Fokus, 12, 11));
        assert_eq!(folge[22], (Phase::LangePause, 12, 12));
        assert_eq!(folge[23], (Phase::Fokus, 1, 0));
    }

    #[test]
    fn ueberspringen_zaehlt_als_erledigt_ohne_ereignis_und_ohne_ueberzeit() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());

        // aus bereit: Fokus 1 → kurze Pause, läuft (Pause automatisch)
        assert_eq!(ablauf.befehl(Befehl::Ueberspringen, t0), vec![]);
        let stand = ablauf.stand(t0 + minuten(1));
        assert_eq!(
            (stand.phase, stand.zustand, stand.erledigt),
            (Phase::KurzePause, Zustand::Laeuft, 1)
        );
        assert_eq!(stand.rest, minuten(4));

        // aus läuft: kurze Pause → Fokus 2, bereit (Fokus nicht automatisch), ohne Überzeit
        assert_eq!(
            ablauf.befehl(Befehl::Ueberspringen, t0 + minuten(1)),
            vec![]
        );
        let stand = ablauf.stand(t0 + minuten(3));
        assert_eq!(
            (stand.phase, stand.zustand, stand.fokusphase),
            (Phase::Fokus, Zustand::Bereit, 2)
        );
        assert_eq!(stand.ueberzeit, None);

        // aus angehalten: Fokus 2 → kurze Pause
        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(3));
        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(4));
        assert_eq!(
            ablauf.befehl(Befehl::Ueberspringen, t0 + minuten(5)),
            vec![]
        );
        let stand = ablauf.stand(t0 + minuten(5));
        assert_eq!(
            (stand.phase, stand.zustand, stand.erledigt),
            (Phase::KurzePause, Zustand::Laeuft, 2)
        );
        assert_eq!(stand.rest, minuten(5));
    }

    #[test]
    fn ueberspringen_aus_bereit_mit_ueberzeit_beendet_die_ueberzeit() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Ueberspringen, t0);
        ablauf.takt(t0 + minuten(5)); // Pause läuft ab → Fokus 2 bereit mit Überzeit

        ablauf.befehl(Befehl::Ueberspringen, t0 + minuten(6));

        let stand = ablauf.stand(t0 + minuten(6));
        assert_eq!(
            (stand.phase, stand.zustand),
            (Phase::KurzePause, Zustand::Laeuft)
        );
        assert_eq!(stand.ueberzeit, None);
    }

    #[test]
    fn phase_zuruecksetzen_behaelt_den_zyklus_fortschritt_und_uebernimmt_die_neue_dauer() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Ueberspringen, t0); // kurze Pause nach 1
        ablauf.befehl(Befehl::Ueberspringen, t0); // Fokus 2 bereit
        ablauf.befehl(Befehl::Primaeraktion, t0); // Fokus 2 läuft
        ablauf.einstellungen(Ablaufeinstellungen {
            fokus: minuten(50),
            ..voreinstellungen()
        });

        ablauf.befehl(Befehl::PhaseZuruecksetzen, t0 + minuten(10));

        let stand = ablauf.stand(t0 + minuten(20));
        assert_eq!(
            (stand.phase, stand.zustand),
            (Phase::Fokus, Zustand::Bereit)
        );
        assert_eq!((stand.fokusphase, stand.erledigt), (2, 1));
        assert_eq!(stand.rest, minuten(50));
        assert_eq!(stand.ueberzeit, None);
    }

    #[test]
    fn neu_beginnen_setzt_alles_zurueck_mit_allen_neuen_werten() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Ueberspringen, t0);
        ablauf.befehl(Befehl::Ueberspringen, t0);
        ablauf.befehl(Befehl::Ueberspringen, t0); // kurze Pause nach 2, läuft
        ablauf.einstellungen(Ablaufeinstellungen {
            fokus: minuten(40),
            fokusphasen_pro_zyklus: 6,
            ..voreinstellungen()
        });

        ablauf.befehl(Befehl::NeuBeginnen, t0 + minuten(1));

        let stand = ablauf.stand(t0 + minuten(2));
        assert_eq!(
            (stand.phase, stand.zustand),
            (Phase::Fokus, Zustand::Bereit)
        );
        assert_eq!(
            (
                stand.fokusphase,
                stand.erledigt,
                stand.fokusphasen_im_zyklus
            ),
            (1, 0, 6)
        );
        assert_eq!(stand.rest, minuten(40));
        assert_eq!(stand.ueberzeit, None);
    }

    #[test]
    fn geaenderte_dauer_gilt_fuer_bereit_sofort_sonst_ab_der_naechsten_phase_dieses_typs() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());

        // bereit übernimmt sofort
        ablauf.einstellungen(Ablaufeinstellungen {
            fokus: minuten(30),
            ..voreinstellungen()
        });
        assert_eq!(ablauf.stand(t0).rest, minuten(30));

        // laufende Phase behält ihre Dauer, auch angehalten
        ablauf.befehl(Befehl::Primaeraktion, t0);
        ablauf.einstellungen(Ablaufeinstellungen {
            fokus: minuten(10),
            kurze_pause: minuten(7),
            ..voreinstellungen()
        });
        assert_eq!(ablauf.stand(t0 + minuten(5)).rest, minuten(25));
        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(5));
        assert_eq!(ablauf.stand(t0 + minuten(6)).rest, minuten(25));
        ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(6));

        // nächste Phase dieses Typs nimmt den neuen Wert
        ablauf.takt(t0 + minuten(31));
        let stand = ablauf.stand(t0 + minuten(31));
        assert_eq!((stand.phase, stand.rest), (Phase::KurzePause, minuten(7)));
        ablauf.takt(t0 + minuten(38)); // Fokus 2 bereit mit Überzeit
        assert_eq!(ablauf.stand(t0 + minuten(38)).rest, minuten(10));

        // bereit mit Überzeit übernimmt ebenfalls sofort
        ablauf.einstellungen(Ablaufeinstellungen {
            fokus: minuten(20),
            ..voreinstellungen()
        });
        let stand = ablauf.stand(t0 + minuten(39));
        assert_eq!(stand.rest, minuten(20));
        assert_eq!(stand.ueberzeit, Some(minuten(1)));
    }

    #[test]
    fn geaenderte_fokusphasen_pro_zyklus_gelten_ab_dem_naechsten_zyklus() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Ueberspringen, t0); // kurze Pause nach 1

        ablauf.einstellungen(Ablaufeinstellungen {
            fokusphasen_pro_zyklus: 2,
            ..voreinstellungen()
        });

        assert_eq!(ablauf.stand(t0).fokusphasen_im_zyklus, 4);
        for _ in 0..6 {
            ablauf.befehl(Befehl::Ueberspringen, t0); // bis zur langen Pause nach 4
        }
        assert_eq!(ablauf.stand(t0).phase, Phase::LangePause);
        ablauf.befehl(Befehl::Ueberspringen, t0);
        assert_eq!(ablauf.stand(t0).fokusphasen_im_zyklus, 2);
    }

    #[test]
    fn geaenderte_fokusphasen_pro_zyklus_gelten_am_zyklusanfang_sofort() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());

        ablauf.einstellungen(Ablaufeinstellungen {
            fokusphasen_pro_zyklus: 6,
            ..voreinstellungen()
        });

        assert_eq!(ablauf.stand(t0).fokusphasen_im_zyklus, 6);
    }

    #[test]
    fn standby_ueber_mehrere_phasen_ergibt_genau_ein_ereignis_und_den_stand_wie_ohne_schlaf() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(Ablaufeinstellungen {
            fokus_automatisch: true,
            ..voreinstellungen()
        });
        ablauf.befehl(Befehl::Primaeraktion, t0);

        // Fokus 1 (0–25), Pause (25–30), Fokus 2 (30–55), Pause (55–60), Fokus 3 ab 60
        let ereignisse = ablauf.takt(t0 + minuten(70));

        assert_eq!(
            ereignisse,
            vec![Ereignis::PhaseAbgelaufen {
                von: Phase::KurzePause,
                nach: Phase::Fokus
            }]
        );
        let stand = ablauf.stand(t0 + minuten(70));
        assert_eq!(
            (stand.phase, stand.zustand),
            (Phase::Fokus, Zustand::Laeuft)
        );
        assert_eq!((stand.fokusphase, stand.erledigt), (3, 2));
        assert_eq!(stand.rest, minuten(15));
    }

    #[test]
    fn standby_endet_in_bereit_mit_ueberzeit_ab_dem_echten_ablaufzeitpunkt() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Primaeraktion, t0);

        // Fokus 1 (0–25), Pause läuft automatisch (25–30), Fokus 2 bereit ab 30
        let ereignisse = ablauf.takt(t0 + minuten(90));

        assert_eq!(
            ereignisse,
            vec![Ereignis::PhaseAbgelaufen {
                von: Phase::KurzePause,
                nach: Phase::Fokus
            }]
        );
        let stand = ablauf.stand(t0 + minuten(90));
        assert_eq!(
            (stand.phase, stand.zustand, stand.fokusphase),
            (Phase::Fokus, Zustand::Bereit, 2)
        );
        assert_eq!(stand.ueberzeit, Some(minuten(60)));
    }

    #[test]
    fn takt_hundertmal_mit_demselben_zeitpunkt_aendert_nichts() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Primaeraktion, t0);
        let jetzt = t0 + minuten(25);
        assert_eq!(ablauf.takt(jetzt).len(), 1);
        let vorher = ablauf.stand(jetzt);

        for _ in 0..100 {
            assert_eq!(ablauf.takt(jetzt), vec![]);
        }

        assert_eq!(ablauf.stand(jetzt), vorher);
    }

    #[test]
    fn befehl_nach_standby_holt_erst_nach_und_wirkt_dann_auf_den_neuen_stand() {
        let t0 = Instant::now();
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Primaeraktion, t0);

        // ohne takt: Fokus 1 und Pause sind abgelaufen, Fokus 2 ist bereit
        let ereignisse = ablauf.befehl(Befehl::Primaeraktion, t0 + minuten(40));

        assert_eq!(
            ereignisse,
            vec![Ereignis::PhaseAbgelaufen {
                von: Phase::KurzePause,
                nach: Phase::Fokus
            }]
        );
        let stand = ablauf.stand(t0 + minuten(40));
        assert_eq!(
            (stand.phase, stand.zustand, stand.fokusphase),
            (Phase::Fokus, Zustand::Laeuft, 2)
        );
        assert_eq!(stand.rest, minuten(25));
    }

    #[test]
    fn naechste_faelligkeit_faellt_auf_die_naechste_volle_sekunde_oder_den_ablauf() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut ablauf = Ablauf::neu(voreinstellungen());
        assert_eq!(ablauf.naechste_faelligkeit(t0), None);

        ablauf.befehl(Befehl::Primaeraktion, t0);
        assert_eq!(ablauf.naechste_faelligkeit(t0), Some(t0 + ms(1000)));
        assert_eq!(
            ablauf.naechste_faelligkeit(t0 + ms(300)),
            Some(t0 + ms(1000))
        );
        assert_eq!(
            ablauf.naechste_faelligkeit(t0 + minuten(25) - ms(400)),
            Some(t0 + minuten(25))
        );

        ablauf.befehl(Befehl::Primaeraktion, t0 + ms(1500));
        assert_eq!(ablauf.naechste_faelligkeit(t0 + ms(1500)), None);
    }

    #[test]
    fn naechste_faelligkeit_folgt_bei_ueberzeit_den_vollen_sekunden_ab_dem_ablauf() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut ablauf = Ablauf::neu(voreinstellungen());
        ablauf.befehl(Befehl::Ueberspringen, t0); // Pause läuft 0–5
        ablauf.takt(t0 + minuten(5)); // Fokus 2 bereit mit Überzeit ab 5

        assert_eq!(
            ablauf.naechste_faelligkeit(t0 + minuten(5) + ms(2400)),
            Some(t0 + minuten(5) + ms(3000))
        );
        assert_eq!(
            ablauf.naechste_faelligkeit(t0 + minuten(5)),
            Some(t0 + minuten(5) + ms(1000))
        );
    }
}
