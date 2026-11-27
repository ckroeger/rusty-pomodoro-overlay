//! Die Einstellungsdatei (SPEC 8): einzige Quelle der Einstellungen, liest und schreibt
//! `einstellungen.toml` mit `toml_edit`, damit Kommentare erhalten bleiben.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use toml_edit::{DocumentMut, Item, Value};

const DATEINAME: &str = "einstellungen.toml";

/// Kommentierte Vorlage, die beim ersten Start angelegt wird.
pub const VORLAGE: &str = include_str!("../assets/einstellungen.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Einstellungen {
    pub fokus_minuten: u8,
    pub kurze_pause_minuten: u8,
    pub lange_pause_minuten: u8,
    pub fokusphasen_pro_zyklus: u8,
    pub pause_automatisch_starten: bool,
    pub fokus_automatisch_starten: bool,
    pub ton: bool,
    pub signal_im_overlay: bool,
    pub overlay_bei_phasenwechsel_einblenden: bool,
    /// Schlüssel: `dunkel`, `hell` oder Dateiname ohne Endung
    pub theme: String,
    pub immer_im_vordergrund: bool,
    pub overlay_anzeigen: bool,
    /// Prozent, 50 bis 300
    pub skalierung: u16,
    /// obere linke Ecke in physischen Pixeln des virtuellen Bildschirms
    pub position: Option<(i32, i32)>,
}

impl Default for Einstellungen {
    fn default() -> Self {
        Self {
            fokus_minuten: 25,
            kurze_pause_minuten: 5,
            lange_pause_minuten: 15,
            fokusphasen_pro_zyklus: 4,
            pause_automatisch_starten: true,
            fokus_automatisch_starten: false,
            ton: true,
            signal_im_overlay: true,
            overlay_bei_phasenwechsel_einblenden: false,
            theme: "dunkel".into(),
            immer_im_vordergrund: true,
            overlay_anzeigen: true,
            skalierung: 100,
            position: None,
        }
    }
}

/// Ein Fehler in der Einstellungsdatei; `text()` ist die Fassung fürs Menü.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fehler {
    /// kein gültiges TOML; die zuletzt gültigen Werte bleiben
    Syntax {
        zeile: usize,
    },
    /// falscher Typ oder außerhalb des Bereichs; es gilt die Voreinstellung
    Ungueltig {
        schluessel: String,
        wert: String,
        verwendet: String,
    },
    Unbekannt {
        schluessel: String,
    },
}

impl Fehler {
    pub fn text(&self) -> String {
        match self {
            Fehler::Syntax { zeile } => format!("Fehler in Zeile {zeile}"),
            Fehler::Ungueltig {
                schluessel,
                wert,
                verwendet,
            } => {
                format!("{schluessel} = {wert} ungültig, {verwendet} verwendet")
            }
            Fehler::Unbekannt { schluessel } => format!("unbekannter Schlüssel {schluessel}"),
        }
    }
}

/// Der Wert eines Schlüssels als TOML, so wie er in die Datei geschrieben wird.
/// `None` für einen unbekannten Schlüssel oder eine fehlende Position.
fn wert_von(e: &Einstellungen, schluessel: &str) -> Option<Value> {
    Some(match schluessel {
        "fokus_minuten" => i64::from(e.fokus_minuten).into(),
        "kurze_pause_minuten" => i64::from(e.kurze_pause_minuten).into(),
        "lange_pause_minuten" => i64::from(e.lange_pause_minuten).into(),
        "fokusphasen_pro_zyklus" => i64::from(e.fokusphasen_pro_zyklus).into(),
        "pause_automatisch_starten" => e.pause_automatisch_starten.into(),
        "fokus_automatisch_starten" => e.fokus_automatisch_starten.into(),
        "ton" => e.ton.into(),
        "signal_im_overlay" => e.signal_im_overlay.into(),
        "overlay_bei_phasenwechsel_einblenden" => e.overlay_bei_phasenwechsel_einblenden.into(),
        "theme" => e.theme.as_str().into(),
        "immer_im_vordergrund" => e.immer_im_vordergrund.into(),
        "overlay_anzeigen" => e.overlay_anzeigen.into(),
        "skalierung" => i64::from(e.skalierung).into(),
        "position" => {
            let (x, y) = e.position?;
            Value::Array([i64::from(x), i64::from(y)].into_iter().collect())
        }
        _ => return None,
    })
}

/// Wert ohne die umgebenden Leerzeichen aus der Datei.
fn als_text(wert: &Value) -> String {
    wert.clone().decorated("", "").to_string()
}

/// Eine Änderung aus Menü, Maus oder Einstellungsfenster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aenderung {
    FokusMinuten(u8),
    KurzePauseMinuten(u8),
    LangePauseMinuten(u8),
    FokusphasenProZyklus(u8),
    PauseAutomatischStarten(bool),
    FokusAutomatischStarten(bool),
    Ton(bool),
    SignalImOverlay(bool),
    OverlayBeiPhasenwechselEinblenden(bool),
    Theme(String),
    ImmerImVordergrund(bool),
    OverlayAnzeigen(bool),
    /// wird erst nach 1 s ohne weitere Änderung geschrieben
    Skalierung(u16),
    /// wird erst nach 1 s ohne weitere Änderung geschrieben
    Position(i32, i32),
}

impl Aenderung {
    /// Setzt den Wert und liefert den Schlüssel in der Datei.
    fn anwenden(self, e: &mut Einstellungen) -> &'static str {
        match self {
            Aenderung::FokusMinuten(m) => {
                e.fokus_minuten = m;
                "fokus_minuten"
            }
            Aenderung::KurzePauseMinuten(m) => {
                e.kurze_pause_minuten = m;
                "kurze_pause_minuten"
            }
            Aenderung::LangePauseMinuten(m) => {
                e.lange_pause_minuten = m;
                "lange_pause_minuten"
            }
            Aenderung::FokusphasenProZyklus(n) => {
                e.fokusphasen_pro_zyklus = n;
                "fokusphasen_pro_zyklus"
            }
            Aenderung::PauseAutomatischStarten(b) => {
                e.pause_automatisch_starten = b;
                "pause_automatisch_starten"
            }
            Aenderung::FokusAutomatischStarten(b) => {
                e.fokus_automatisch_starten = b;
                "fokus_automatisch_starten"
            }
            Aenderung::Ton(b) => {
                e.ton = b;
                "ton"
            }
            Aenderung::SignalImOverlay(b) => {
                e.signal_im_overlay = b;
                "signal_im_overlay"
            }
            Aenderung::OverlayBeiPhasenwechselEinblenden(b) => {
                (
                    e.overlay_bei_phasenwechsel_einblenden = b,
                    "overlay_bei_phasenwechsel_einblenden",
                )
                    .1
            }
            Aenderung::Theme(t) => {
                e.theme = t;
                "theme"
            }
            Aenderung::ImmerImVordergrund(b) => {
                e.immer_im_vordergrund = b;
                "immer_im_vordergrund"
            }
            Aenderung::OverlayAnzeigen(b) => {
                e.overlay_anzeigen = b;
                "overlay_anzeigen"
            }
            Aenderung::Skalierung(p) => {
                e.skalierung = p;
                "skalierung"
            }
            Aenderung::Position(x, y) => {
                e.position = Some((x, y));
                "position"
            }
        }
    }
}

/// Was Schreiben und Sichern brauchen; geteilt mit [`Sicherung`] für `WM_ENDSESSION`.
#[derive(Debug)]
struct Inneres {
    pfad: PathBuf,
    /// Änderungszeitpunkt der Datei, wie die App sie zuletzt gelesen oder geschrieben hat
    gesehen: Option<SystemTime>,
    /// noch nicht geschriebene Werte (Schlüssel, Wert)
    ausstehend: Vec<(&'static str, Value)>,
    /// wann verzögerte Werte (Position, Skalierung) geschrieben werden
    faellig: Option<Instant>,
    /// ein Schreibvorgang ist fehlgeschlagen und noch nicht abgeholt
    schreibfehler: bool,
}

/// Klonbarer Griff, um ausstehende Änderungen aus der Fensterprozedur zu schreiben
/// (`WM_ENDSESSION`, SPEC 10.4).
#[derive(Debug, Clone)]
pub struct Sicherung(Arc<Mutex<Inneres>>);

impl Sicherung {
    /// Schreibt alles Ausstehende. Hält gerade jemand die Sperre (Aufruf mitten aus
    /// einem eigenen Schreibvorgang), passiert nichts, statt zu blockieren.
    pub fn sichern(&self) {
        if let Ok(mut inneres) = self.0.try_lock() {
            inneres.schreiben();
        }
    }
}

/// Ruhezeit, nach der Position und Skalierung geschrieben werden.
const VERZOEGERUNG: Duration = Duration::from_secs(1);

#[derive(Debug, PartialEq, Eq)]
enum Schreibergebnis {
    Geschrieben,
    /// Die Datei hat einen Syntaxfehler; die Werte bleiben ausstehend.
    Gesperrt,
    Fehlgeschlagen,
}

impl Inneres {
    fn vormerken(&mut self, schluessel: &'static str, wert: Value) {
        self.ausstehend.retain(|(s, _)| *s != schluessel);
        self.ausstehend.push((schluessel, wert));
    }

    /// Liest die Datei frisch, setzt nur die ausstehenden Werte und schreibt atomar.
    fn schreiben(&mut self) -> Schreibergebnis {
        if self.ausstehend.is_empty() {
            return Schreibergebnis::Geschrieben;
        }
        let text = fs::read_to_string(&self.pfad).unwrap_or_else(|_| VORLAGE.to_owned());
        let Ok(mut dokument) = text.parse::<DocumentMut>() else {
            return Schreibergebnis::Gesperrt;
        };
        for (schluessel, wert) in &self.ausstehend {
            let mut neu = wert.clone();
            match dokument.get_mut(schluessel).and_then(Item::as_value_mut) {
                Some(alt) => {
                    *neu.decor_mut() = alt.decor().clone();
                    *alt = neu;
                }
                None => {
                    // Kommentare am Dateiende (wie `# position = …` der Vorlage) gehören
                    // über den neuen Schlüssel, nicht unter ihn.
                    let anhang = dokument.trailing().as_str().unwrap_or("").to_owned();
                    dokument.set_trailing("");
                    dokument.insert(schluessel, Item::Value(neu));
                    if let Some(mut schluessel) = dokument.as_table_mut().key_mut(schluessel) {
                        schluessel.leaf_decor_mut().set_prefix(anhang);
                    }
                }
            }
        }
        let tmp = self.pfad.with_extension("toml.tmp");
        let ok = fs::write(&tmp, dokument.to_string()).and_then(|()| fs::rename(&tmp, &self.pfad));
        if ok.is_err() {
            let _ = fs::remove_file(&tmp);
            self.schreibfehler = true;
            return Schreibergebnis::Fehlgeschlagen;
        }
        self.ausstehend.clear();
        self.faellig = None;
        self.gesehen = aenderungszeit(&self.pfad);
        Schreibergebnis::Geschrieben
    }
}

pub struct Einstellungsdatei {
    aktuell: Einstellungen,
    fehler: Vec<Fehler>,
    inneres: Arc<Mutex<Inneres>>,
}

impl Einstellungsdatei {
    /// Öffnet `<ordner>/einstellungen.toml` und legt Ordner und Vorlage bei Bedarf an.
    pub fn oeffnen(ordner: &Path) -> Self {
        let mut datei = Self {
            aktuell: Einstellungen::default(),
            fehler: Vec::new(),
            inneres: Arc::new(Mutex::new(Inneres {
                pfad: ordner.join(DATEINAME),
                gesehen: None,
                ausstehend: Vec::new(),
                faellig: None,
                schreibfehler: false,
            })),
        };
        datei.einlesen();
        datei
    }

    pub fn aktuell(&self) -> (&Einstellungen, &[Fehler]) {
        (&self.aktuell, &self.fehler)
    }

    /// Gilt sofort im Speicher; geschrieben wird sofort bzw. bei Position und
    /// Skalierung nach 1 s Ruhe (SPEC 8.3).
    pub fn aendern(&mut self, aenderung: Aenderung, jetzt: Instant) {
        let verzoegert = matches!(
            aenderung,
            Aenderung::Position(..) | Aenderung::Skalierung(_)
        );
        let schluessel = aenderung.anwenden(&mut self.aktuell);
        let mut inneres = self.inneres.lock().unwrap();
        if let Some(wert) = wert_von(&self.aktuell, schluessel) {
            inneres.vormerken(schluessel, wert);
        }
        if verzoegert {
            inneres.faellig = Some(jetzt + VERZOEGERUNG);
        } else {
            inneres.schreiben();
        }
    }

    /// Schreibt alles Ausstehende sofort (beim Beenden).
    pub fn sichern(&self) {
        self.sicherung().sichern();
    }

    pub fn sicherung(&self) -> Sicherung {
        Sicherung(Arc::clone(&self.inneres))
    }

    /// Ob seit dem letzten Abholen ein Schreibvorgang fehlgeschlagen ist.
    pub fn schreibfehler_abholen(&mut self) -> bool {
        std::mem::take(&mut self.inneres.lock().unwrap().schreibfehler)
    }

    pub fn pfad(&self) -> PathBuf {
        self.inneres.lock().unwrap().pfad.clone()
    }

    /// Im Sekundentakt: schreibt Fälliges und erkennt Hand-Änderungen.
    /// Liefert die neuen Werte, wenn die Datei von außen geändert wurde.
    pub fn takt(&mut self, jetzt: Instant) -> Option<&Einstellungen> {
        let mut inneres = self.inneres.lock().unwrap();
        if inneres.faellig.is_some_and(|f| f <= jetzt) {
            inneres.schreiben();
        }
        let veraendert = !inneres.pfad.exists() || aenderungszeit(&inneres.pfad) != inneres.gesehen;
        drop(inneres);
        (veraendert && self.einlesen()).then_some(&self.aktuell)
    }

    /// Liest die Datei (legt sie aus der Vorlage an, wenn sie fehlt). Bei einem
    /// Syntaxfehler bleiben die zuletzt gültigen Werte. Liefert, ob neue Werte gelten;
    /// dann gewinnt die Datei über noch nicht geschriebene Änderungen.
    fn einlesen(&mut self) -> bool {
        let mut inneres = self.inneres.lock().unwrap();
        let pfad = inneres.pfad.clone();
        if !pfad.exists() {
            if let Some(ordner) = pfad.parent() {
                let _ = fs::create_dir_all(ordner);
            }
            let _ = fs::write(&pfad, VORLAGE);
        }
        inneres.gesehen = aenderungszeit(&pfad);
        let text = fs::read_to_string(&pfad).unwrap_or_else(|_| VORLAGE.to_owned());
        match lesen(&text) {
            Ok((einstellungen, fehler)) => {
                self.aktuell = einstellungen;
                self.fehler = fehler;
                inneres.ausstehend.clear();
                inneres.faellig = None;
                true
            }
            Err(syntax) => {
                self.fehler = vec![syntax];
                false
            }
        }
    }
}

fn aenderungszeit(pfad: &Path) -> Option<SystemTime> {
    fs::metadata(pfad).and_then(|m| m.modified()).ok()
}

/// Die reine Naht: Text der Datei → Werte und Fehler. Jeder Wert wird einzeln geprüft.
pub fn lesen(text: &str) -> Result<(Einstellungen, Vec<Fehler>), Fehler> {
    let dokument: DocumentMut = text.parse().map_err(|e| Fehler::Syntax {
        zeile: crate::theme::fehlerzeile(text, &e),
    })?;
    let mut e = Einstellungen::default();
    let mut fehler = Vec::new();
    for (schluessel, item) in dokument.iter() {
        let Some(wert) = item.as_value() else {
            fehler.push(Fehler::Unbekannt {
                schluessel: schluessel.to_owned(),
            });
            continue;
        };
        let minuten = || ganzzahl(wert, 1, 180).map(|m| m as u8);
        let gueltig = match schluessel {
            "fokus_minuten" => minuten().map(|m| e.fokus_minuten = m),
            "kurze_pause_minuten" => minuten().map(|m| e.kurze_pause_minuten = m),
            "lange_pause_minuten" => minuten().map(|m| e.lange_pause_minuten = m),
            "fokusphasen_pro_zyklus" => {
                ganzzahl(wert, 1, 12).map(|n| e.fokusphasen_pro_zyklus = n as u8)
            }
            "pause_automatisch_starten" => wert.as_bool().map(|b| e.pause_automatisch_starten = b),
            "fokus_automatisch_starten" => wert.as_bool().map(|b| e.fokus_automatisch_starten = b),
            "ton" => wert.as_bool().map(|b| e.ton = b),
            "signal_im_overlay" => wert.as_bool().map(|b| e.signal_im_overlay = b),
            "overlay_bei_phasenwechsel_einblenden" => wert
                .as_bool()
                .map(|b| e.overlay_bei_phasenwechsel_einblenden = b),
            "theme" => wert.as_str().map(|t| e.theme = t.to_owned()),
            "immer_im_vordergrund" => wert.as_bool().map(|b| e.immer_im_vordergrund = b),
            "overlay_anzeigen" => wert.as_bool().map(|b| e.overlay_anzeigen = b),
            "skalierung" => ganzzahl(wert, 50, 300).map(|p| e.skalierung = p as u16),
            "position" => position(wert).map(|p| e.position = Some(p)),
            _ => {
                fehler.push(Fehler::Unbekannt {
                    schluessel: schluessel.to_owned(),
                });
                continue;
            }
        };
        if gueltig.is_none() {
            let verwendet = wert_von(&Einstellungen::default(), schluessel)
                .map_or_else(|| "unten rechts".to_owned(), |v| als_text(&v));
            fehler.push(Fehler::Ungueltig {
                schluessel: schluessel.to_owned(),
                wert: als_text(wert),
                verwendet,
            });
        }
    }
    Ok((e, fehler))
}

fn ganzzahl(wert: &Value, min: i64, max: i64) -> Option<i64> {
    wert.as_integer().filter(|n| (min..=max).contains(n))
}

fn position(wert: &Value) -> Option<(i32, i32)> {
    let feld = wert.as_array()?;
    let mut zahlen = feld
        .iter()
        .map(|v| v.as_integer().and_then(|n| i32::try_from(n).ok()));
    match (zahlen.next(), zahlen.next(), zahlen.next()) {
        (Some(Some(x)), Some(Some(y)), None) => Some((x, y)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_vorlage_ergibt_genau_die_voreinstellungen() {
        assert_eq!(lesen(VORLAGE), Ok((Einstellungen::default(), vec![])));
    }

    #[test]
    fn voreinstellungen_wie_in_der_spec() {
        let e = Einstellungen::default();

        assert_eq!(
            (
                e.fokus_minuten,
                e.kurze_pause_minuten,
                e.lange_pause_minuten
            ),
            (25, 5, 15)
        );
        assert_eq!(e.fokusphasen_pro_zyklus, 4);
        assert_eq!(
            (e.pause_automatisch_starten, e.fokus_automatisch_starten),
            (true, false)
        );
        assert_eq!(
            (
                e.ton,
                e.signal_im_overlay,
                e.overlay_bei_phasenwechsel_einblenden
            ),
            (true, true, false)
        );
        assert_eq!(e.theme, "dunkel");
        assert_eq!((e.immer_im_vordergrund, e.overlay_anzeigen), (true, true));
        assert_eq!((e.skalierung, e.position), (100, None));
    }

    #[test]
    fn fehlende_schluessel_nehmen_still_die_voreinstellung() {
        let (e, fehler) = lesen("fokus_minuten = 50\n").unwrap();

        assert_eq!(
            e,
            Einstellungen {
                fokus_minuten: 50,
                ..Einstellungen::default()
            }
        );
        assert_eq!(fehler, vec![]);
    }

    fn fehlertexte(text: &str) -> (Einstellungen, Vec<String>) {
        let (e, fehler) = lesen(text).unwrap();
        (e, fehler.iter().map(Fehler::text).collect())
    }

    #[test]
    fn ungueltige_werte_nehmen_die_voreinstellung_und_werden_einzeln_gemeldet() {
        let text = r#"
fokus_minuten = 500
kurze_pause_minuten = 0
lange_pause_minuten = "15"
fokusphasen_pro_zyklus = 13
ton = "ja"
theme = 5
skalierung = 40
position = [1600]
lange_pause_minuten_extra = 1
immer_im_vordergrund = false
"#;
        let (e, fehler) = fehlertexte(text);

        assert_eq!(
            e,
            Einstellungen {
                immer_im_vordergrund: false,
                ..Einstellungen::default()
            }
        );
        assert_eq!(
            fehler,
            vec![
                "fokus_minuten = 500 ungültig, 25 verwendet",
                "kurze_pause_minuten = 0 ungültig, 5 verwendet",
                "lange_pause_minuten = \"15\" ungültig, 15 verwendet",
                "fokusphasen_pro_zyklus = 13 ungültig, 4 verwendet",
                "ton = \"ja\" ungültig, true verwendet",
                "theme = 5 ungültig, \"dunkel\" verwendet",
                "skalierung = 40 ungültig, 100 verwendet",
                "position = [1600] ungültig, unten rechts verwendet",
                "unbekannter Schlüssel lange_pause_minuten_extra",
            ]
        );
    }

    #[test]
    fn position_aus_zwei_ganzzahlen_auch_negativ() {
        let (e, fehler) = fehlertexte("position = [-1200, 300]\n");

        assert_eq!(e.position, Some((-1200, 300)));
        assert_eq!(fehler, Vec::<String>::new());
    }

    #[test]
    fn syntaxfehler_meldet_die_zeile() {
        let fehler = lesen("ton = true\n\n\nfokus_minuten = \n").unwrap_err();

        assert_eq!(fehler.text(), "Fehler in Zeile 4");
    }

    // --- Dateisystem, im temporären Verzeichnis ---

    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Leerer, eindeutiger Ordner je Test.
    fn testordner(name: &str) -> PathBuf {
        let ordner =
            std::env::temp_dir().join(format!("rusty-pomodoro-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&ordner);
        ordner
    }

    fn inhalt(ordner: &std::path::Path) -> String {
        fs::read_to_string(ordner.join("einstellungen.toml")).unwrap()
    }

    #[test]
    fn oeffnen_legt_ordner_und_kommentierte_vorlage_an() {
        let ordner = testordner("anlegen");

        let datei = Einstellungsdatei::oeffnen(&ordner);

        assert_eq!(inhalt(&ordner), VORLAGE);
        assert_eq!(datei.aktuell(), (&Einstellungen::default(), &[][..]));
    }

    #[test]
    fn oeffnen_liest_eine_vorhandene_datei_und_laesst_sie_stehen() {
        let ordner = testordner("vorhanden");
        fs::create_dir_all(&ordner).unwrap();
        fs::write(
            ordner.join("einstellungen.toml"),
            "fokus_minuten = 50\nfarbe = 1\n",
        )
        .unwrap();

        let datei = Einstellungsdatei::oeffnen(&ordner);

        assert_eq!(datei.aktuell().0.fokus_minuten, 50);
        assert_eq!(
            datei.aktuell().1,
            &[Fehler::Unbekannt {
                schluessel: "farbe".into()
            }]
        );
        assert_eq!(inhalt(&ordner), "fokus_minuten = 50\nfarbe = 1\n");
    }

    #[test]
    fn aenderung_wird_sofort_geschrieben_und_kommentare_bleiben() {
        let ordner = testordner("sofort");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);

        datei.aendern(Aenderung::Ton(false), t0);
        datei.aendern(Aenderung::Theme("pipboy".into()), t0);

        let erwartet = VORLAGE
            .replace("ton = true", "ton = false")
            .replace("theme = \"dunkel\"", "theme = \"pipboy\"");
        assert_eq!(inhalt(&ordner), erwartet);
        assert!(!datei.aktuell().0.ton);
        assert!(!ordner.join("einstellungen.toml.tmp").exists());
    }

    #[test]
    fn schreiben_liest_die_datei_frisch_und_aendert_nur_den_eigenen_wert() {
        let ordner = testordner("frisch");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        // jemand ändert die Datei, bevor die App den nächsten Takt hatte
        fs::write(
            ordner.join("einstellungen.toml"),
            "# meins\nfokus_minuten = 40 # lang\n",
        )
        .unwrap();

        datei.aendern(Aenderung::FokusMinuten(45), t0);

        assert_eq!(inhalt(&ordner), "# meins\nfokus_minuten = 45 # lang\n");
    }

    #[test]
    fn position_und_skalierung_werden_erst_nach_einer_sekunde_ruhe_geschrieben() {
        let ordner = testordner("verzoegert");
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut datei = Einstellungsdatei::oeffnen(&ordner);

        datei.aendern(Aenderung::Position(1500, 800), t0);
        datei.aendern(Aenderung::Skalierung(120), t0 + ms(300));
        assert_eq!(datei.aktuell().0.position, Some((1500, 800)));
        datei.takt(t0 + ms(900));
        datei.aendern(Aenderung::Position(1600, 900), t0 + ms(800));
        datei.takt(t0 + ms(1500));
        assert_eq!(inhalt(&ordner), VORLAGE, "noch keine Sekunde Ruhe");

        datei.takt(t0 + ms(1800));

        let erwartet = format!(
            "{}position = [1600, 900]\n",
            VORLAGE.replace("skalierung = 100", "skalierung = 120")
        );
        assert_eq!(inhalt(&ordner), erwartet);
    }

    /// Schreibt wie ein Editor, mit sicher anderem Änderungszeitpunkt.
    fn von_hand(ordner: &std::path::Path, text: &str) {
        let pfad = ordner.join("einstellungen.toml");
        fs::write(&pfad, text).unwrap();
        let spaeter = std::time::SystemTime::now() + Duration::from_secs(2);
        fs::File::options()
            .write(true)
            .open(&pfad)
            .unwrap()
            .set_modified(spaeter)
            .unwrap();
    }

    #[test]
    fn eigene_schreibvorgaenge_loesen_kein_neues_einlesen_aus() {
        let ordner = testordner("eigene");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);

        datei.aendern(Aenderung::Ton(false), t0);

        assert_eq!(datei.takt(t0), None);
    }

    #[test]
    fn hand_aenderung_wird_im_takt_erkannt() {
        let ordner = testordner("hand");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);

        von_hand(&ordner, "fokus_minuten = 500\nton = false\n");

        let neu = datei.takt(t0).cloned();
        assert_eq!(
            neu,
            Some(Einstellungen {
                ton: false,
                ..Einstellungen::default()
            })
        );
        assert_eq!(datei.aktuell().1.len(), 1);
        assert_eq!(datei.takt(t0), None, "nur einmal");
    }

    #[test]
    fn bei_syntaxfehler_bleiben_die_werte_und_die_app_schreibt_nicht() {
        let ordner = testordner("syntax");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        datei.aendern(Aenderung::FokusMinuten(40), t0);

        von_hand(&ordner, "fokus_minuten = \n");
        assert_eq!(datei.takt(t0), None);
        datei.aendern(Aenderung::Ton(false), t0);

        assert_eq!(datei.aktuell().0.fokus_minuten, 40);
        assert!(!datei.aktuell().0.ton, "gilt im Speicher");
        assert_eq!(datei.aktuell().1, &[Fehler::Syntax { zeile: 1 }]);
        assert_eq!(inhalt(&ordner), "fokus_minuten = \n");
    }

    #[test]
    fn nach_reparatur_gewinnt_der_stand_aus_dem_editor() {
        let ordner = testordner("reparatur");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        von_hand(&ordner, "fokus_minuten = \n");
        datei.takt(t0);
        datei.aendern(Aenderung::Ton(false), t0);

        von_hand(&ordner, "fokus_minuten = 30\n");
        let neu = datei.takt(t0).cloned();

        assert_eq!(
            neu,
            Some(Einstellungen {
                fokus_minuten: 30,
                ..Einstellungen::default()
            })
        );
        assert_eq!(datei.aktuell().1, &[]);
        assert_eq!(inhalt(&ordner), "fokus_minuten = 30\n");
    }

    #[test]
    fn geloeschte_datei_wird_im_takt_neu_angelegt_mit_voreinstellungen() {
        let ordner = testordner("geloescht");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        datei.aendern(Aenderung::Ton(false), t0);

        fs::remove_file(ordner.join("einstellungen.toml")).unwrap();
        let neu = datei.takt(t0).cloned();

        assert_eq!(neu, Some(Einstellungen::default()));
        assert_eq!(inhalt(&ordner), VORLAGE);
    }

    #[test]
    fn sicherung_schreibt_ausstehendes_sofort() {
        let ordner = testordner("sicherung");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        let sicherung = datei.sicherung();
        datei.aendern(Aenderung::Skalierung(150), t0);

        sicherung.sichern();

        assert!(inhalt(&ordner).contains("skalierung = 150"));
    }

    #[test]
    fn fehlgeschlagenes_schreiben_wird_einmal_gemeldet_und_spaeter_nachgeholt() {
        let ordner = testordner("schreibfehler");
        let t0 = Instant::now();
        let mut datei = Einstellungsdatei::oeffnen(&ordner);
        let pfad = ordner.join("einstellungen.toml");
        let schreibschutz = |an: bool| {
            let mut rechte = fs::metadata(&pfad).unwrap().permissions();
            rechte.set_readonly(an);
            fs::set_permissions(&pfad, rechte).unwrap();
        };
        schreibschutz(true);

        datei.aendern(Aenderung::Ton(false), t0);

        assert!(datei.schreibfehler_abholen());
        assert!(!datei.schreibfehler_abholen(), "nur einmal");
        assert!(!ordner.join("einstellungen.toml.tmp").exists());
        schreibschutz(false);
        datei.sichern();
        assert!(inhalt(&ordner).contains("ton = false"));
    }
}
