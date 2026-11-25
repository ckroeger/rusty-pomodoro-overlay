//! PROTOTYP – Sekundentakt bei ausgeblendetem Overlay (eframe 0.37 + glow, nur Windows).
//! Misst, wie oft `App::logic` aufgerufen wird: sichtbar, ausgeblendet, bei offenem Menü,
//! beim Ziehen und nach Standby. Zwei Taktgeber zum Vergleich:
//!   A) `ctx.request_repaint_after(..)` aus `logic` heraus (auf volle Sekunden ausgerichtet)
//!   B) eigener Thread, der jede volle Sekunde `ctx.request_repaint()` ruft
//! Jeder `logic`-Aufruf landet als Zeile in `PROTOTYP-takt.csv` (im aktuellen Ordner).
//! Wegwerf-Code: keine Tests, keine Fehlerbehandlung, keine Abstraktionen.
#![windows_subsystem = "windows"]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, RichText, ViewportCommand};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tray_icon::menu::{CheckMenuItem, ContextMenu, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

/// Abstände über dieser Grenze gelten als "Takt verpasst".
const LUECKE_MS: u128 = 1200;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sekundentakt-Prototyp")
            .with_inner_size([440.0, 330.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "sekundentakt-prototyp",
        options,
        Box::new(|cc| Ok(Box::new(Prototyp::new(&cc.egui_ctx)))),
    )
}

#[derive(Default, Clone, Copy)]
struct Statistik {
    aufrufe: u64,
    min_ms: u128,
    max_ms: u128,
    summe_ms: u128,
    luecken: u64,
}

impl Statistik {
    fn neu() -> Self {
        Self { min_ms: u128::MAX, ..Default::default() }
    }
    fn zaehle(&mut self, ms: u128) {
        self.aufrufe += 1;
        self.min_ms = self.min_ms.min(ms);
        self.max_ms = self.max_ms.max(ms);
        self.summe_ms += ms;
        if ms > LUECKE_MS {
            self.luecken += 1;
        }
    }
    fn text(&self) -> String {
        if self.aufrufe == 0 {
            return "noch keine Aufrufe".into();
        }
        format!(
            "{} Aufrufe · min {} · Ø {} · max {} ms · Lücken >{} ms: {}",
            self.aufrufe,
            self.min_ms,
            self.summe_ms / self.aufrufe as u128,
            self.max_ms,
            LUECKE_MS,
            self.luecken
        )
    }
}

struct Prototyp {
    tray: TrayIcon,
    menu: Menu,
    eintrag_sichtbar: MenuItem,
    eintrag_60s: MenuItem,
    eintrag_thread: CheckMenuItem,
    eintrag_abschnitt: MenuItem,
    eintrag_beenden: MenuItem,

    sichtbar: bool,
    wieder_einblenden_um: Option<Instant>,
    thread_modus: Arc<AtomicBool>,

    start: Instant,
    letzter_mono: Instant,
    letzte_wand: SystemTime,
    abschnitt: u32,
    stat_sichtbar: Statistik,
    stat_versteckt: Statistik,
    letzte_luecken: Vec<String>,
    csv: BufWriter<File>,

    cpu_basis: (Instant, u64),
    cpu_prozent: f64,
    /// `--auto`: Skript ohne Klicks (Schritt, Startzeit des Schritts).
    auto: Option<(usize, Instant)>,
}

/// Skript für `--auto`: (Dauer in s, sichtbar, Thread-Taktgeber).
const AUTO: [(u64, bool, bool); 4] = [(20, true, false), (20, false, false), (20, true, true), (20, false, true)];

impl Prototyp {
    fn new(ctx: &egui::Context) -> Self {
        // Menü-Ereignisse wecken die Event-Loop, damit `logic` sie sofort sieht.
        let c = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e| {
            MENU_KANAL.with_sender(e);
            c.request_repaint();
        }));

        let eintrag_sichtbar = MenuItem::new("Overlay ausblenden", true, None);
        let eintrag_60s = MenuItem::new("Ausblenden, nach 60 s von selbst einblenden", true, None);
        let eintrag_thread = CheckMenuItem::new("Taktgeber B: eigener Thread", true, false, None);
        let eintrag_abschnitt = MenuItem::new("Neuer Messabschnitt (Statistik leeren)", true, None);
        let eintrag_beenden = MenuItem::new("Beenden", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &eintrag_sichtbar,
            &eintrag_60s,
            &PredefinedMenuItem::separator(),
            &eintrag_thread,
            &eintrag_abschnitt,
            &PredefinedMenuItem::separator(),
            &eintrag_beenden,
        ])
        .unwrap();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_menu_on_left_click(true)
            .with_tooltip("Sekundentakt-Prototyp")
            .with_icon(kreis_icon())
            .build()
            .unwrap();

        // Taktgeber B: läuft immer, ruft aber nur `request_repaint`, wenn eingeschaltet.
        let thread_modus = Arc::new(AtomicBool::new(false));
        {
            let modus = thread_modus.clone();
            let c = ctx.clone();
            std::thread::spawn(move || {
                loop {
                    let jetzt = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let bis_volle_sekunde = 1000 - (jetzt.as_millis() % 1000) as u64;
                    std::thread::sleep(Duration::from_millis(bis_volle_sekunde));
                    if modus.load(Ordering::Relaxed) {
                        c.request_repaint();
                    }
                }
            });
        }

        let mut csv = BufWriter::new(File::create("PROTOTYP-takt.csv").unwrap());
        writeln!(
            csv,
            "wand_unix_ms;seit_start_ms;abstand_mono_ms;abstand_wand_ms;sichtbar;taktgeber;abschnitt"
        )
        .unwrap();

        let jetzt = Instant::now();
        Self {
            tray,
            menu,
            eintrag_sichtbar,
            eintrag_60s,
            eintrag_thread,
            eintrag_abschnitt,
            eintrag_beenden,
            sichtbar: true,
            wieder_einblenden_um: None,
            thread_modus,
            start: jetzt,
            letzter_mono: jetzt,
            letzte_wand: SystemTime::now(),
            abschnitt: 1,
            stat_sichtbar: Statistik::neu(),
            stat_versteckt: Statistik::neu(),
            letzte_luecken: Vec::new(),
            csv,
            cpu_basis: (jetzt, cpu_zeit_100ns()),
            cpu_prozent: 0.0,
            auto: std::env::args().any(|a| a == "--auto").then(|| (usize::MAX, jetzt)),
        }
    }

    fn setze_sichtbar(&mut self, ctx: &egui::Context, sichtbar: bool) {
        self.sichtbar = sichtbar;
        ctx.send_viewport_cmd(ViewportCommand::Visible(sichtbar));
        self.eintrag_sichtbar
            .set_text(if sichtbar { "Overlay ausblenden" } else { "Overlay einblenden" });
    }

    fn thread_an(&self) -> bool {
        self.thread_modus.load(Ordering::Relaxed)
    }
}

impl eframe::App for Prototyp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    /// Hier liegt alles, was am Sekundentakt hängt – eframe 0.37 ruft das auch bei
    /// verstecktem Fenster auf (dann ohne `ui`).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mono = Instant::now();
        let wand = SystemTime::now();
        let abstand_mono = (mono - self.letzter_mono).as_millis();
        let abstand_wand = wand.duration_since(self.letzte_wand).map(|d| d.as_millis()).unwrap_or(0);
        self.letzter_mono = mono;
        self.letzte_wand = wand;

        let taktgeber = if self.thread_an() { "B-thread" } else { "A-repaint_after" };
        if self.sichtbar {
            self.stat_sichtbar.zaehle(abstand_mono);
        } else {
            self.stat_versteckt.zaehle(abstand_mono);
        }
        if abstand_mono > LUECKE_MS || abstand_wand > LUECKE_MS {
            let seit = self.start.elapsed().as_secs();
            self.letzte_luecken.push(format!(
                "t+{:>5}s  mono {:>6} ms  wand {:>6} ms  {}",
                seit,
                abstand_mono,
                abstand_wand,
                if self.sichtbar { "sichtbar" } else { "versteckt" }
            ));
            if self.letzte_luecken.len() > 6 {
                self.letzte_luecken.remove(0);
            }
        }
        writeln!(
            self.csv,
            "{};{};{};{};{};{};{}",
            wand.duration_since(UNIX_EPOCH).unwrap().as_millis(),
            self.start.elapsed().as_millis(),
            abstand_mono,
            abstand_wand,
            self.sichtbar,
            taktgeber,
            self.abschnitt
        )
        .unwrap();
        self.csv.flush().unwrap();

        // CPU-Anteil über ~10 s.
        if self.cpu_basis.0.elapsed() >= Duration::from_secs(10) {
            let cpu = cpu_zeit_100ns();
            let wand_100ns = self.cpu_basis.0.elapsed().as_nanos() as f64 / 100.0;
            self.cpu_prozent = (cpu - self.cpu_basis.1) as f64 / wand_100ns * 100.0;
            self.cpu_basis = (Instant::now(), cpu);
        }

        // Menü-Ereignisse.
        while let Some(e) = MENU_KANAL.try_recv() {
            if e.id == *self.eintrag_sichtbar.id() {
                self.wieder_einblenden_um = None;
                self.setze_sichtbar(ctx, !self.sichtbar);
            } else if e.id == *self.eintrag_60s.id() {
                self.wieder_einblenden_um = Some(Instant::now() + Duration::from_secs(60));
                self.setze_sichtbar(ctx, false);
            } else if e.id == *self.eintrag_thread.id() {
                let an = !self.thread_an();
                self.thread_modus.store(an, Ordering::Relaxed);
                self.eintrag_thread.set_checked(an); // muda schaltet selbst um – aus Zustand neu setzen
            } else if e.id == *self.eintrag_abschnitt.id() {
                self.abschnitt += 1;
                self.stat_sichtbar = Statistik::neu();
                self.stat_versteckt = Statistik::neu();
                self.letzte_luecken.clear();
            } else if e.id == *self.eintrag_beenden.id() {
                std::process::exit(0); // Prototyp: CSV ist schon geflusht
            }
        }

        // `--auto`: nächster Skriptschritt (neuer Abschnitt je Schritt).
        if let Some((schritt, seit)) = self.auto {
            let faellig = schritt == usize::MAX || seit.elapsed() >= Duration::from_secs(AUTO[schritt].0);
            if faellig {
                let naechster = schritt.wrapping_add(1);
                if naechster >= AUTO.len() {
                    std::process::exit(0);
                }
                let (_, sichtbar, thread) = AUTO[naechster];
                self.thread_modus.store(thread, Ordering::Relaxed);
                self.eintrag_thread.set_checked(thread);
                if sichtbar != self.sichtbar {
                    self.setze_sichtbar(ctx, sichtbar);
                }
                self.abschnitt = naechster as u32 + 1;
                self.auto = Some((naechster, Instant::now()));
            }
        }

        // Selbstständiges Wiedereinblenden – prüft Visible(true) aus `logic` ohne Eingabe.
        if let Some(um) = self.wieder_einblenden_um
            && mono >= um
        {
            self.wieder_einblenden_um = None;
            self.setze_sichtbar(ctx, true);
        }

        // Tray-Tooltip jede Sekunde neu.
        let s = if self.sichtbar { &self.stat_sichtbar } else { &self.stat_versteckt };
        let _ = self.tray.set_tooltip(Some(format!(
            "{} · {}\nletzter Abstand {} ms · max {} ms",
            if self.sichtbar { "sichtbar" } else { "versteckt" },
            taktgeber,
            abstand_mono,
            s.max_ms
        )));

        // Taktgeber A: nächster Aufruf zur nächsten vollen Sekunde.
        if !self.thread_an() {
            let ms = wand.duration_since(UNIX_EPOCH).unwrap().as_millis() % 1000;
            ctx.request_repaint_after(Duration::from_millis(1000 - ms as u64));
        }
    }

    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let panel = egui::Frame::NONE
            .fill(Color32::from_black_alpha(190))
            .corner_radius(12.0)
            .inner_margin(12.0);

        egui::CentralPanel::default().frame(panel).show(root, |ui| {
            let bg = ui.interact(ui.max_rect(), egui::Id::unique("hg"), egui::Sense::click_and_drag());
            if bg.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if bg.secondary_clicked()
                && let Some(h) = hwnd(frame)
            {
                // Dasselbe Menü wie am Tray. Solange es offen ist, steht die Event-Loop.
                unsafe { self.menu.show_context_menu_for_hwnd(h, None) };
            }

            let seit = self.start.elapsed().as_secs();
            ui.label(
                RichText::new(format!("{:02}:{:02}", seit / 60, seit % 60))
                    .size(40.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            let mono = |t: String| RichText::new(t).monospace().small().color(Color32::from_gray(220));
            ui.label(mono(format!(
                "Taktgeber: {}   Abschnitt {}   CPU ~{:.2} %",
                if self.thread_an() { "B (Thread)" } else { "A (request_repaint_after)" },
                self.abschnitt,
                self.cpu_prozent
            )));
            ui.label(mono(format!("sichtbar:  {}", self.stat_sichtbar.text())));
            ui.label(mono(format!("versteckt: {}", self.stat_versteckt.text())));
            ui.separator();
            ui.label(mono(format!("Letzte Lücken (> {LUECKE_MS} ms):")));
            if self.letzte_luecken.is_empty() {
                ui.label(mono("  keine".into()));
            }
            for l in &self.letzte_luecken {
                ui.label(mono(format!("  {l}")));
            }
            ui.separator();
            ui.label(mono(
                "Rechtsklick = Menü (auch am Tray) · Ziehen verschiebt\nCSV: PROTOTYP-takt.csv im Startordner".into(),
            ));
        });
    }
}

// --- Kleinkram -------------------------------------------------------------------------

/// Kanal für Menü-Ereignisse (muda ruft den Handler auf seinem Thread bzw. im WndProc).
struct Kanal {
    inner: std::sync::OnceLock<(std::sync::mpsc::Sender<MenuEvent>, std::sync::Mutex<std::sync::mpsc::Receiver<MenuEvent>>)>,
}
impl Kanal {
    fn get(&self) -> &(std::sync::mpsc::Sender<MenuEvent>, std::sync::Mutex<std::sync::mpsc::Receiver<MenuEvent>>) {
        self.inner.get_or_init(|| {
            let (s, r) = std::sync::mpsc::channel();
            (s, std::sync::Mutex::new(r))
        })
    }
    fn with_sender(&self, e: MenuEvent) {
        let _ = self.get().0.send(e);
    }
    fn try_recv(&self) -> Option<MenuEvent> {
        self.get().1.lock().unwrap().try_recv().ok()
    }
}
static MENU_KANAL: Kanal = Kanal { inner: std::sync::OnceLock::new() };

fn hwnd(frame: &eframe::Frame) -> Option<isize> {
    match frame.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
        _ => None,
    }
}

fn kreis_icon() -> Icon {
    let n = 32u32;
    let mut rgba = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (x as f32 - 15.5, y as f32 - 15.5);
            let innen = dx * dx + dy * dy <= 14.0 * 14.0;
            rgba.extend_from_slice(if innen { &[230, 80, 60, 255] } else { &[0, 0, 0, 0] });
        }
    }
    Icon::from_rgba(rgba, n, n).unwrap()
}

/// Kernel- plus User-Zeit des Prozesses in 100-ns-Einheiten.
fn cpu_zeit_100ns() -> u64 {
    let z = || FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut a, mut b, mut k, mut u) = (z(), z(), z(), z());
    unsafe { GetProcessTimes(GetCurrentProcess(), &mut a, &mut b, &mut k, &mut u) };
    let f = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
    f(k) + f(u)
}
