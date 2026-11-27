//! Der Theme-Ordner (SPEC 7.3, 7.4): im Sekundentakt gelesen, ohne Datei-Watcher.
//! Neu geparst wird nur, was sich laut Änderungszeitpunkt geändert hat.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::theme::{self, Theme, Themeeintrag};

#[derive(Debug)]
struct Datei {
    geaendert: Option<SystemTime>,
    ergebnis: Result<Theme, theme::Fehler>,
    bild_fehlt: bool,
}

pub struct Themeordner {
    pfad: PathBuf,
    /// nach Dateiname ohne Endung
    dateien: BTreeMap<String, Datei>,
}

impl Themeordner {
    /// Liest noch nichts; das macht der erste [`Themeordner::takt`].
    pub fn neu(pfad: &Path) -> Self {
        Self {
            pfad: pfad.to_owned(),
            dateien: BTreeMap::new(),
        }
    }

    pub fn pfad(&self) -> &Path {
        &self.pfad
    }

    /// Liest Dateinamen und Änderungszeitpunkte und parst nur Neues und Geändertes.
    /// Liefert, ob sich etwas geändert hat.
    pub fn takt(&mut self) -> bool {
        let mut gefunden = BTreeMap::new();
        if let Ok(eintraege) = fs::read_dir(&self.pfad) {
            for eintrag in eintraege.flatten() {
                let pfad = eintrag.path();
                if pfad
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
                    && let Some(stamm) = pfad.file_stem()
                {
                    let geaendert = eintrag.metadata().and_then(|m| m.modified()).ok();
                    gefunden.insert(stamm.to_string_lossy().into_owned(), (pfad, geaendert));
                }
            }
        }
        let mut veraendert = false;
        let vorher = self.dateien.len();
        self.dateien
            .retain(|schluessel, _| gefunden.contains_key(schluessel));
        veraendert |= self.dateien.len() != vorher;
        for (schluessel, (pfad, geaendert)) in gefunden {
            let bekannt = self
                .dateien
                .get(&schluessel)
                .is_some_and(|d| d.geaendert == geaendert);
            if !bekannt {
                let ergebnis = match fs::read_to_string(&pfad) {
                    Ok(text) => theme::lesen(&text, &pfad).map(|(t, _warnungen)| t),
                    // nicht lesbar (z. B. kein UTF-8) wie ein Syntaxfehler in Zeile 1
                    Err(_) => Err(theme::Fehler { zeile: 1 }),
                };
                self.dateien.insert(
                    schluessel.clone(),
                    Datei {
                        geaendert,
                        ergebnis,
                        bild_fehlt: false,
                    },
                );
                veraendert = true;
            }
            let datei = self
                .dateien
                .get_mut(&schluessel)
                .expect("eben eingefügt oder bekannt");
            let bild_fehlt = datei
                .ergebnis
                .as_ref()
                .is_ok_and(|t| t.bild.as_ref().is_some_and(|b| !b.is_file()));
            veraendert |= datei.bild_fehlt != bild_fehlt;
            datei.bild_fehlt = bild_fehlt;
        }
        veraendert
    }

    /// Die eigenen Themes fürs Menü.
    pub fn liste(&self) -> Vec<Themeeintrag> {
        let mut liste = theme::themeliste(
            self.dateien
                .iter()
                .map(|(schluessel, d)| (schluessel.clone(), d.ergebnis.clone()))
                .collect(),
        );
        for eintrag in &mut liste {
            eintrag.bild_fehlt = self.dateien[&eintrag.schluessel].bild_fehlt;
        }
        liste
    }

    /// Ein wählbares eigenes Theme (nicht kaputt, nicht reserviert).
    pub fn theme(&self, schluessel: &str) -> Option<&Theme> {
        let reserviert = theme::eingebaute()
            .iter()
            .any(|(s, _)| s.eq_ignore_ascii_case(schluessel));
        if reserviert {
            return None;
        }
        self.dateien.get(schluessel)?.ergebnis.as_ref().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn testordner(name: &str) -> PathBuf {
        let ordner = std::env::temp_dir().join(format!(
            "rusty-pomodoro-test-{}-themes-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&ordner);
        fs::create_dir_all(&ordner).unwrap();
        ordner
    }

    fn namen(ordner: &Themeordner) -> Vec<(String, String, theme::Themezustand, bool)> {
        ordner
            .liste()
            .into_iter()
            .map(|e| (e.schluessel, e.name, e.zustand, e.bild_fehlt))
            .collect()
    }

    #[test]
    fn liest_nur_toml_dateien_sortiert_und_markiert_kaputte() {
        let pfad = testordner("lesen");
        fs::write(
            pfad.join("pipboy.toml"),
            "name = \"Pip-Boy\"\nfokus = \"#33FF66\"\n",
        )
        .unwrap();
        fs::write(pfad.join("aqua.toml"), "fokus = \n").unwrap();
        fs::write(pfad.join("notiz.txt"), "kein Theme").unwrap();
        let mut ordner = Themeordner::neu(&pfad);

        assert!(ordner.takt());

        use theme::Themezustand::*;
        assert_eq!(
            namen(&ordner),
            vec![
                (
                    "aqua".into(),
                    "aqua".into(),
                    Syntaxfehler { zeile: 1 },
                    false
                ),
                ("pipboy".into(), "Pip-Boy".into(), Gueltig, false),
            ]
        );
        assert_eq!(ordner.theme("pipboy").map(|t| t.farben.fokus.g), Some(0xFF));
        assert_eq!(ordner.theme("aqua"), None);
    }

    #[test]
    fn fehlender_ordner_ergibt_eine_leere_liste_und_wird_nicht_angelegt() {
        let pfad = testordner("fehlt").join("gibt-es-nicht");
        let mut ordner = Themeordner::neu(&pfad);

        assert!(!ordner.takt());

        assert_eq!(ordner.liste(), vec![]);
        assert!(!pfad.exists());
    }

    /// Schreibt mit sicher anderem Änderungszeitpunkt.
    fn schreiben(pfad: &Path, text: &str, sekunden_spaeter: u64) {
        fs::write(pfad, text).unwrap();
        let zeit = SystemTime::now() + std::time::Duration::from_secs(sekunden_spaeter);
        fs::File::options()
            .write(true)
            .open(pfad)
            .unwrap()
            .set_modified(zeit)
            .unwrap();
    }

    #[test]
    fn erkennt_geaenderte_neue_und_geloeschte_dateien() {
        let pfad = testordner("aenderungen");
        schreiben(&pfad.join("a.toml"), "name = \"Alt\"\n", 0);
        schreiben(&pfad.join("b.toml"), "", 0);
        let mut ordner = Themeordner::neu(&pfad);
        ordner.takt();
        assert!(!ordner.takt(), "nichts geändert");

        schreiben(&pfad.join("a.toml"), "name = \"Neu\"\n", 2);
        fs::remove_file(pfad.join("b.toml")).unwrap();
        schreiben(&pfad.join("c.toml"), "", 0);

        assert!(ordner.takt());
        let liste: Vec<_> = ordner
            .liste()
            .into_iter()
            .map(|e| (e.schluessel, e.name))
            .collect();
        assert_eq!(
            liste,
            vec![("c".into(), "c".into()), ("a".into(), "Neu".into())]
        );
    }

    #[test]
    fn fehlendes_bild_wird_markiert_und_wiederkommen_erkannt() {
        let pfad = testordner("bild");
        fs::write(pfad.join("foto.toml"), "bild = \"foto.jpg\"\n").unwrap();
        let mut ordner = Themeordner::neu(&pfad);
        ordner.takt();
        assert!(ordner.liste()[0].bild_fehlt);

        fs::write(pfad.join("foto.jpg"), b"irgendwas").unwrap();

        assert!(ordner.takt());
        assert!(!ordner.liste()[0].bild_fehlt);
        assert!(ordner.theme("foto").is_some(), "bleibt wählbar");
    }

    #[test]
    fn reservierte_namen_sind_nicht_waehlbar() {
        let pfad = testordner("reserviert");
        fs::write(pfad.join("Hell.toml"), "").unwrap();
        let mut ordner = Themeordner::neu(&pfad);
        ordner.takt();

        assert_eq!(ordner.liste()[0].zustand, theme::Themezustand::Reserviert);
        assert_eq!(ordner.theme("Hell"), None);
    }
}
