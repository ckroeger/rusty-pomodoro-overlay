//! Die drei Schriften des Overlays (SPEC 2.3). Inter und JetBrains Mono sind eingebettet,
//! Ubuntu Light kommt mit egui.

pub static INTER: &[u8] = include_bytes!("../assets/Inter-Regular-tnum.ttf");
pub static JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

use std::sync::Arc;

use eframe::egui::{FontData, FontDefinitions, FontFamily};

use crate::theme::Schrift;

/// Die egui-Standardschriften plus je eine eigene Familie für Inter und JetBrains Mono,
/// mit den Standardschriften als Rückfall.
pub fn definitionen() -> FontDefinitions {
    let mut d = FontDefinitions::default();
    let rueckfall = d
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, daten) in [("inter", INTER), ("jetbrains_mono", JETBRAINS_MONO)] {
        d.font_data
            .insert(name.into(), Arc::new(FontData::from_static(daten)));
        let mut familie = vec![name.to_owned()];
        familie.extend(rueckfall.iter().cloned());
        d.families.insert(FontFamily::Name(name.into()), familie);
    }
    d
}

/// Die Familie für Zeit und Phasenwort im Overlay.
pub fn familie(schrift: Schrift) -> FontFamily {
    match schrift {
        Schrift::Ubuntu => FontFamily::Proportional,
        Schrift::Inter => FontFamily::Name("inter".into()),
        Schrift::JetbrainsMono => FontFamily::Name("jetbrains_mono".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::MetadataProvider as _;

    /// Vorschubbreiten von 0–9 und `+` in Schrifteinheiten.
    fn breiten(schrift: &[u8]) -> Vec<(char, f32)> {
        let font = skrifa::FontRef::new(schrift).expect("gültige Schrift");
        let metriken = font.glyph_metrics(
            skrifa::instance::Size::unscaled(),
            skrifa::instance::LocationRef::default(),
        );
        ('0'..='9')
            .chain(['+'])
            .map(|c| {
                let id = font.charmap().map(c).expect("Zeichen vorhanden");
                (c, metriken.advance_width(id).expect("Breite vorhanden"))
            })
            .collect()
    }

    fn gleich_breit(name: &str, schrift: &[u8]) {
        let b = breiten(schrift);
        let erste = b[0].1;
        assert!(
            b.iter().all(|&(_, w)| w == erste),
            "{name}: Ziffern und + nicht gleich breit: {b:?}"
        );
    }

    #[test]
    fn ziffern_und_plus_sind_in_inter_gleich_breit() {
        gleich_breit("Inter", INTER);
    }

    #[test]
    fn ziffern_und_plus_sind_in_jetbrains_mono_gleich_breit() {
        gleich_breit("JetBrains Mono", JETBRAINS_MONO);
    }
}
