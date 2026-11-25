# PROTOTYP – Messergebnisse Sekundentakt (Ticket 14)

Windows 11, Release-Build, eframe 0.37 + glow, tray-icon 0.26.

| Fall | Taktgeber | Abstände `logic()` | Lücken > 1,2 s |
|---|---|---|---|
| sichtbar | A `request_repaint_after` | Median 16–72 ms (Zusatz-Frames), max ~1000 ms | nur erster Aufruf beim Start |
| versteckt | A | Median 999 ms, max 1001–1008 ms (3 Läufe, zusammen ~120 s) | 0 (einmal 1950 ms im Handtest) |
| sichtbar | B Thread + `request_repaint` | unregelmäßig, max 2392 ms | 3 |
| versteckt | B | Median 300–490 ms (doppelt so oft wie nötig) | 0 |
| Menü offen | A | Loop steht: 12,5 s / 7,1 s Lücke | erwartet |
| Ziehen | A | Uhr läuft weiter, keine Lücke | 0 |
| Standby ~130 s | A | ein Aufruf mit 129 755 ms, mono = wand | erwartet |

- Wiedereinblenden per `Visible(true)` aus `logic()` ohne Eingabe: nach exakt 60 s (Handtest 2).
- `Instant` zählt den Standby mit (Monotonuhr und Wanduhr laufen gleich).
- CPU im sichtbaren Leerlauf: 0,63 % eines Kerns, 117 MB Working Set.

CSV-Dateien: `PROTOTYP-takt-handtest1.csv` (Menü, Ziehen, Ausblenden), `PROTOTYP-takt-handtest2-standby.csv`.
