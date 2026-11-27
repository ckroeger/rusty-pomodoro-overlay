//! Erzeugt die beiden Hinweisklänge (SPEC 2.3) als WAV nach `assets/`.
//! Einmalig ausführen: `cargo run --example klaenge_erzeugen`. Die fertigen Dateien liegen im Repo.

use std::fs;
use std::io::{self, Write};

const ABTASTRATE: u32 = 22_050;
const AMPLITUDE: f64 = 0.5;
const BLENDE_S: f64 = 0.010;

/// Ein Ton (Frequenz in Hz) oder Stille (`None`), jeweils mit Dauer in Sekunden.
type Abschnitt = (Option<f64>, f64);

fn main() -> io::Result<()> {
    // fallend: „ausruhen“
    let fokus_endet: &[Abschnitt] = &[(Some(880.0), 0.180), (None, 0.060), (Some(660.0), 0.180)];
    // steigend: „los geht’s“
    let pause_endet: &[Abschnitt] = &[
        (Some(660.0), 0.150),
        (None, 0.050),
        (Some(880.0), 0.150),
        (None, 0.050),
        (Some(1100.0), 0.150),
    ];
    schreiben("assets/fokus_endet.wav", fokus_endet)?;
    schreiben("assets/pause_endet.wav", pause_endet)?;
    Ok(())
}

fn schreiben(pfad: &str, abschnitte: &[Abschnitt]) -> io::Result<()> {
    let mut proben: Vec<i16> = Vec::new();
    for &(frequenz, dauer) in abschnitte {
        let anzahl = (dauer * f64::from(ABTASTRATE)).round() as usize;
        for i in 0..anzahl {
            let t = i as f64 / f64::from(ABTASTRATE);
            let wert = match frequenz {
                Some(f) => {
                    // 10 ms Ein- und Ausblenden gegen Knacken
                    let blende = (t / BLENDE_S).min((dauer - t) / BLENDE_S).clamp(0.0, 1.0);
                    AMPLITUDE * blende * (2.0 * std::f64::consts::PI * f * t).sin()
                }
                None => 0.0,
            };
            proben.push((wert * f64::from(i16::MAX)).round() as i16);
        }
    }
    let daten_bytes = (proben.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + daten_bytes as usize);
    wav.write_all(b"RIFF")?;
    wav.write_all(&(36 + daten_bytes).to_le_bytes())?;
    wav.write_all(b"WAVEfmt ")?;
    wav.write_all(&16u32.to_le_bytes())?; // Größe des fmt-Blocks
    wav.write_all(&1u16.to_le_bytes())?; // PCM
    wav.write_all(&1u16.to_le_bytes())?; // mono
    wav.write_all(&ABTASTRATE.to_le_bytes())?;
    wav.write_all(&(ABTASTRATE * 2).to_le_bytes())?; // Bytes pro Sekunde
    wav.write_all(&2u16.to_le_bytes())?; // Bytes pro Probe
    wav.write_all(&16u16.to_le_bytes())?; // Bits pro Probe
    wav.write_all(b"data")?;
    wav.write_all(&daten_bytes.to_le_bytes())?;
    for probe in proben {
        wav.write_all(&probe.to_le_bytes())?;
    }
    fs::write(pfad, wav)
}
