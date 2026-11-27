//! Themes lesen (SPEC 7). Rein: bekommt Text und Pfad, liest selbst keine Dateien.

use std::path::{Path, PathBuf};

use toml_edit::DocumentMut;

/// Farbe ohne vormultipliziertes Alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Farbe {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Farbe {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schrift {
    Ubuntu,
    Inter,
    JetbrainsMono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Farben {
    pub fokus: Farbe,
    pub kurze_pause: Farbe,
    pub lange_pause: Farbe,
    pub hintergrund: Farbe,
    pub vordergrund: Farbe,
    pub gedaempft: Farbe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub schrift: Schrift,
    pub farben: Farben,
    /// absoluter Pfad, aufgelöst relativ zur Theme-Datei
    pub bild: Option<PathBuf>,
    /// Rohtext; ob er darstellbar ist, entscheidet `grafik`
    pub tray_zeichen: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warnung {
    /// falscher Typ oder Wert; es gilt der Wert aus *Dunkel*
    Ungueltig(String),
    Unbekannt(String),
}

/// Die Datei ist kein gültiges TOML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fehler {
    /// 1-basiert
    pub zeile: usize,
}

pub fn dunkel() -> Theme {
    Theme {
        name: "Dunkel".into(),
        schrift: Schrift::Ubuntu,
        farben: Farben {
            fokus: Farbe::rgb(0xE5, 0x53, 0x4B),
            kurze_pause: Farbe::rgb(0x4C, 0xAF, 0x50),
            lange_pause: Farbe::rgb(0x42, 0xA5, 0xF5),
            hintergrund: Farbe::rgba(0x1E, 0x1E, 0x1E, 0xD9),
            vordergrund: Farbe::rgb(0xF0, 0xF0, 0xF0),
            gedaempft: Farbe::rgb(0x5A, 0x5A, 0x5A),
        },
        bild: None,
        tray_zeichen: None,
    }
}

pub fn hell() -> Theme {
    Theme {
        name: "Hell".into(),
        schrift: Schrift::Ubuntu,
        farben: Farben {
            fokus: Farbe::rgb(0xC6, 0x28, 0x28),
            kurze_pause: Farbe::rgb(0x2E, 0x7D, 0x32),
            lange_pause: Farbe::rgb(0x15, 0x65, 0xC0),
            hintergrund: Farbe::rgba(0xF5, 0xF5, 0xF5, 0xE6),
            vordergrund: Farbe::rgb(0x20, 0x20, 0x20),
            gedaempft: Farbe::rgb(0xB0, 0xB0, 0xB0),
        },
        bild: None,
        tray_zeichen: None,
    }
}

/// Die eingebauten Themes mit ihrem Schlüssel, in Menüreihenfolge.
pub fn eingebaute() -> [(&'static str, Theme); 2] {
    [("dunkel", dunkel()), ("hell", hell())]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Themezustand {
    Gueltig,
    Syntaxfehler {
        zeile: usize,
    },
    /// Dateiname `dunkel` oder `hell`
    Reserviert,
}

/// Ein eigenes Theme, wie es im Menü erscheint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Themeeintrag {
    /// Dateiname ohne Endung
    pub schluessel: String,
    pub name: String,
    pub zustand: Themezustand,
    /// setzt die Schale, wenn das Bild fehlt oder sich nicht dekodieren lässt
    pub bild_fehlt: bool,
}

/// Ordnet die gelesenen Theme-Dateien (Dateiname ohne Endung, Ergebnis von [`lesen`])
/// fürs Menü: alphabetisch nach Name, reservierte und kaputte markiert.
pub fn themeliste(dateien: Vec<(String, Result<Theme, Fehler>)>) -> Vec<Themeeintrag> {
    let mut liste: Vec<_> = dateien
        .into_iter()
        .map(|(schluessel, ergebnis)| {
            let reserviert = eingebaute()
                .iter()
                .any(|(s, _)| s.eq_ignore_ascii_case(&schluessel));
            let (name, zustand) = match ergebnis {
                Ok(theme) if reserviert => (theme.name, Themezustand::Reserviert),
                Ok(theme) => (theme.name, Themezustand::Gueltig),
                Err(Fehler { zeile }) => (schluessel.clone(), Themezustand::Syntaxfehler { zeile }),
            };
            Themeeintrag {
                schluessel,
                name,
                zustand,
                bild_fehlt: false,
            }
        })
        .collect();
    liste.sort_by_key(|e| e.name.to_lowercase());
    liste
}

/// Liest eine Theme-Datei. Fehlendes und Ungültiges kommt von *Dunkel*.
pub fn lesen(text: &str, datei: &Path) -> Result<(Theme, Vec<Warnung>), Fehler> {
    let dokument: DocumentMut = text.parse().map_err(|e| Fehler {
        zeile: fehlerzeile(text, &e),
    })?;
    let mut theme = dunkel();
    theme.name = datei
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut warnungen = Vec::new();
    let ordner = datei.parent().unwrap_or(Path::new(""));
    for (schluessel, wert) in dokument.iter() {
        let text = wert.as_str();
        let farbe = text.and_then(farbe_lesen);
        let f = &mut theme.farben;
        let gueltig = match schluessel {
            "name" => text.map(|t| theme.name = t.to_owned()),
            "schrift" => text.and_then(schrift_lesen).map(|s| theme.schrift = s),
            "bild" => text.map(|t| theme.bild = Some(ordner.join(t))),
            "tray_zeichen" => text.map(|t| theme.tray_zeichen = Some(t.to_owned())),
            "hintergrund" => farbe.map(|c| f.hintergrund = c),
            "fokus" => deckend(farbe).map(|c| f.fokus = c),
            "kurze_pause" => deckend(farbe).map(|c| f.kurze_pause = c),
            "lange_pause" => deckend(farbe).map(|c| f.lange_pause = c),
            "vordergrund" => deckend(farbe).map(|c| f.vordergrund = c),
            "gedaempft" => deckend(farbe).map(|c| f.gedaempft = c),
            _ => {
                warnungen.push(Warnung::Unbekannt(schluessel.to_owned()));
                continue;
            }
        };
        if gueltig.is_none() {
            warnungen.push(Warnung::Ungueltig(schluessel.to_owned()));
        }
    }
    Ok((theme, warnungen))
}

/// 1-basierte Zeile, in der `toml_edit` den Fehler gefunden hat.
pub fn fehlerzeile(text: &str, fehler: &toml_edit::TomlError) -> usize {
    let stelle = fehler.span().map_or(0, |s| s.start).min(text.len());
    text.as_bytes()[..stelle]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

fn deckend(farbe: Option<Farbe>) -> Option<Farbe> {
    farbe.map(|f| Farbe { a: 255, ..f })
}

fn schrift_lesen(text: &str) -> Option<Schrift> {
    match text {
        "ubuntu" => Some(Schrift::Ubuntu),
        "inter" => Some(Schrift::Inter),
        "jetbrains_mono" => Some(Schrift::JetbrainsMono),
        _ => None,
    }
}

/// `#RRGGBB` oder `#RRGGBBAA`.
fn farbe_lesen(text: &str) -> Option<Farbe> {
    let hex = text.strip_prefix('#')?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let teil = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    let a = if hex.len() == 8 { teil(6)? } else { 255 };
    Some(Farbe::rgba(teil(0)?, teil(2)?, teil(4)?, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn farbe(hex: u32) -> Farbe {
        Farbe::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
    }

    fn datei() -> PathBuf {
        PathBuf::from(r"C:\Nutzer\.rusty_pomodoro_overlay\themes\pipboy.toml")
    }

    #[test]
    fn fehlende_felder_erben_von_dunkel() {
        let (theme, warnungen) = lesen("fokus = \"#33FF66\"\n", &datei()).unwrap();

        assert_eq!(warnungen, vec![]);
        assert_eq!(theme.name, "pipboy");
        assert_eq!(theme.farben.fokus, farbe(0x33FF66));
        assert_eq!(theme.farben.kurze_pause, farbe(0x4CAF50));
        assert_eq!(theme.farben.lange_pause, farbe(0x42A5F5));
        assert_eq!(
            theme.farben.hintergrund,
            Farbe {
                r: 0x1E,
                g: 0x1E,
                b: 0x1E,
                a: 0xD9
            }
        );
        assert_eq!(theme.farben.vordergrund, farbe(0xF0F0F0));
        assert_eq!(theme.farben.gedaempft, farbe(0x5A5A5A));
        assert_eq!(theme.schrift, Schrift::Ubuntu);
        assert_eq!(theme.bild, None);
        assert_eq!(theme.tray_zeichen, None);
    }

    #[test]
    fn das_beispiel_aus_der_spec_liest_alle_felder() {
        let text = r##"
name = "Pip-Boy"
schrift = "jetbrains_mono"
bild = "pipboy.jpg"
tray_zeichen = "☢"
fokus = "#33FF66"
kurze_pause = "#99FFAA"
lange_pause = "#66CCFF"
hintergrund = "#000000CC"
"##;
        let (theme, warnungen) = lesen(text, &datei()).unwrap();

        assert_eq!(warnungen, vec![]);
        assert_eq!(theme.name, "Pip-Boy");
        assert_eq!(theme.schrift, Schrift::JetbrainsMono);
        assert_eq!(
            theme.bild,
            Some(PathBuf::from(
                r"C:\Nutzer\.rusty_pomodoro_overlay\themes\pipboy.jpg"
            ))
        );
        assert_eq!(theme.tray_zeichen.as_deref(), Some("☢"));
        assert_eq!(theme.farben.kurze_pause, farbe(0x99FFAA));
        assert_eq!(
            theme.farben.hintergrund,
            Farbe {
                r: 0,
                g: 0,
                b: 0,
                a: 0xCC
            }
        );
    }

    #[test]
    fn nur_der_hintergrund_hat_eine_deckkraft() {
        let (theme, _) = lesen(
            "vordergrund = \"#11223344\"\ngedaempft = \"#AABBCC00\"\n",
            &datei(),
        )
        .unwrap();

        assert_eq!(theme.farben.vordergrund, farbe(0x112233));
        assert_eq!(theme.farben.gedaempft, farbe(0xAABBCC));
    }

    #[test]
    fn inter_ist_als_schrift_waehlbar() {
        let (theme, _) = lesen("schrift = \"inter\"\n", &datei()).unwrap();
        assert_eq!(theme.schrift, Schrift::Inter);
    }

    #[test]
    fn ungueltige_und_unbekannte_felder_werden_gemeldet_und_nehmen_dunkel() {
        let text = r##"
schrift = "comic_sans"
fokus = "rot"
kurze_pause = 5
lange_pause = "#12345"
hintergrund = "#GG0000"
bild = 3
tray_zeichen = true
farbe = "#FFFFFF"
"##;
        let (theme, warnungen) = lesen(text, &datei()).unwrap();

        let dunkel = dunkel();
        assert_eq!(theme.schrift, Schrift::Ubuntu);
        assert_eq!(theme.farben, dunkel.farben);
        assert_eq!(theme.bild, None);
        assert_eq!(theme.tray_zeichen, None);
        assert_eq!(
            warnungen,
            vec![
                Warnung::Ungueltig("schrift".into()),
                Warnung::Ungueltig("fokus".into()),
                Warnung::Ungueltig("kurze_pause".into()),
                Warnung::Ungueltig("lange_pause".into()),
                Warnung::Ungueltig("hintergrund".into()),
                Warnung::Ungueltig("bild".into()),
                Warnung::Ungueltig("tray_zeichen".into()),
                Warnung::Unbekannt("farbe".into()),
            ]
        );
    }

    #[test]
    fn syntaxfehler_meldet_die_zeile() {
        let text = "name = \"Pip-Boy\"\n\nfokus = \"#33FF66\"\nkurze_pause = \n";

        assert_eq!(lesen(text, &datei()), Err(Fehler { zeile: 4 }));
    }

    #[test]
    fn eingebaut_sind_dunkel_und_hell_mit_den_farben_aus_der_spec() {
        let [(s1, dunkel), (s2, hell)] = eingebaute();

        assert_eq!((s1, dunkel.name.as_str()), ("dunkel", "Dunkel"));
        assert_eq!((s2, hell.name.as_str()), ("hell", "Hell"));
        assert_eq!(
            hell.farben,
            Farben {
                hintergrund: Farbe {
                    r: 0xF5,
                    g: 0xF5,
                    b: 0xF5,
                    a: 0xE6
                },
                vordergrund: farbe(0x202020),
                gedaempft: farbe(0xB0B0B0),
                fokus: farbe(0xC62828),
                kurze_pause: farbe(0x2E7D32),
                lange_pause: farbe(0x1565C0),
            }
        );
        assert_eq!(
            (hell.schrift, hell.bild, hell.tray_zeichen),
            (Schrift::Ubuntu, None, None)
        );
    }

    #[test]
    fn themeliste_sortiert_nach_name_und_markiert_reservierte_und_kaputte() {
        let theme = |name: &str| Theme {
            name: name.into(),
            ..dunkel()
        };
        let liste = themeliste(vec![
            ("pipboy".into(), Ok(theme("Pip-Boy"))),
            ("zebra".into(), Ok(theme("aqua"))),
            ("Hell".into(), Ok(theme("Mein Hell"))),
            ("kaputt".into(), Err(Fehler { zeile: 4 })),
        ]);

        let kurz: Vec<_> = liste
            .iter()
            .map(|e| (e.schluessel.as_str(), e.name.as_str(), e.zustand))
            .collect();
        assert_eq!(
            kurz,
            vec![
                ("zebra", "aqua", Themezustand::Gueltig),
                ("kaputt", "kaputt", Themezustand::Syntaxfehler { zeile: 4 }),
                ("Hell", "Mein Hell", Themezustand::Reserviert),
                ("pipboy", "Pip-Boy", Themezustand::Gueltig),
            ]
        );
    }
}
