//! PROTOTYP – Overlay-Spike (eframe/egui + glow, nur Windows).
//! Prüft: randlos, transparent (Alpha pro Pixel UND einheitliche Deckkraft),
//! Immer-im-Vordergrund zur Laufzeit, Verschieben per Maus, Click-through.
//! Wegwerf-Code: keine Tests, keine Fehlerbehandlung, keine Abstraktionen.
#![windows_subsystem = "windows"]

use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, ViewportCommand, WindowLevel};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE, LWA_ALPHA,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

const CLICK_THROUGH_SECS: u64 = 5;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Overlay-Spike")
            .with_inner_size([300.0, 230.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native("overlay-spike", options, Box::new(|_cc| Ok(Box::new(Spike::new()))))
}

struct Spike {
    started: Instant,
    on_top: bool,
    bg_alpha: u8,
    layered_on: bool,
    layered_alpha: u8,
    original_ex_style: Option<isize>,
    click_through_until: Option<Instant>,
    frames: u64,
    last_frame: Instant,
    frame_gap: Duration,
}

impl Spike {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            on_top: true,
            bg_alpha: 140,
            layered_on: false,
            layered_alpha: 200,
            original_ex_style: None,
            click_through_until: None,
            frames: 0,
            last_frame: Instant::now(),
            frame_gap: Duration::ZERO,
        }
    }
}

fn hwnd(frame: &eframe::Frame) -> Option<HWND> {
    match frame.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as HWND),
        _ => None,
    }
}

fn ex_style(h: HWND) -> isize {
    unsafe { GetWindowLongPtrW(h, GWL_EXSTYLE) }
}

impl eframe::App for Spike {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // nötig für Alpha pro Pixel
    }

    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = &root.ctx().clone();
        let now = Instant::now();
        self.frames += 1;
        self.frame_gap = now - self.last_frame;
        self.last_frame = now;
        let h = hwnd(frame);

        // Click-through läuft automatisch ab – sonst gäbe es ohne Hotkey/Tray keinen Weg zurück.
        if let Some(until) = self.click_through_until {
            if now >= until {
                self.click_through_until = None;
                ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
            }
        }

        // Tastenkürzel (nur bei Fokus).
        let (key_t, key_c, key_l, key_esc) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::T),
                i.key_pressed(egui::Key::C),
                i.key_pressed(egui::Key::L),
                i.key_pressed(egui::Key::Escape),
            )
        });
        if key_esc {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        let mut toggle_top = key_t;
        let mut start_click_through = key_c;
        let mut toggle_layered = key_l;

        // Fake-Fokusphase: 25:00 rückwärts.
        let left = Duration::from_secs(25 * 60).saturating_sub(self.started.elapsed());
        let (m, s) = (left.as_secs() / 60, left.as_secs() % 60);

        let panel_frame = egui::Frame::NONE
            .fill(Color32::from_black_alpha(self.bg_alpha))
            .corner_radius(12.0)
            .inner_margin(10.0);

        egui::CentralPanel::default().frame(panel_frame).show(root, |ui| {
            // Hintergrundfläche zuerst: Widgets darüber haben Vorrang, der Rest zieht das Fenster.
            let bg = ui.interact(ui.max_rect(), egui::Id::unique("drag-bg"), egui::Sense::click_and_drag());
            if bg.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }

            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Fokusphase").color(Color32::LIGHT_GRAY));
                ui.label(RichText::new(format!("{m:02}:{s:02}")).size(48.0).strong().color(Color32::WHITE));
            });

            ui.horizontal(|ui| {
                if ui.button(if self.on_top { "Vordergrund: an [T]" } else { "Vordergrund: aus [T]" }).clicked() {
                    toggle_top = true;
                }
                if ui.button(format!("Click-through {CLICK_THROUGH_SECS}s [C]")).clicked() {
                    start_click_through = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Hintergrund-α");
                ui.add(egui::Slider::new(&mut self.bg_alpha, 0..=255));
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.layered_on.clone(), "Fenster-Deckkraft [L]").clicked() {
                    toggle_layered = true;
                }
                let changed = ui.add_enabled(self.layered_on, egui::Slider::new(&mut self.layered_alpha, 30..=255)).changed();
                if changed {
                    if let Some(h) = h {
                        unsafe { SetLayeredWindowAttributes(h, 0, self.layered_alpha, LWA_ALPHA) };
                    }
                }
            });

            // Zustand sichtbar machen.
            let ex = h.map(ex_style).unwrap_or(0) as u32;
            let flag = |f: u32| if ex & f != 0 { "ja" } else { "nein" };
            ui.label(
                RichText::new(format!(
                    "WS_EX_TOPMOST: {}  LAYERED: {}  TRANSPARENT: {}\nClick-through: {}  Frames: {}  Abstand: {} ms\nZiehen: auf freie Fläche klicken · Esc beendet",
                    flag(WS_EX_TOPMOST),
                    flag(WS_EX_LAYERED),
                    flag(WS_EX_TRANSPARENT),
                    match self.click_through_until {
                        Some(u) => format!("noch {}s", u.saturating_duration_since(now).as_secs() + 1),
                        None => "aus".into(),
                    },
                    self.frames,
                    self.frame_gap.as_millis(),
                ))
                .small()
                .monospace()
                .color(Color32::from_gray(200)),
            );
        });

        if toggle_top {
            self.on_top = !self.on_top;
            let level = if self.on_top { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
        }
        if start_click_through {
            // Workaround egui #2537: erst zur Laufzeit setzen, nicht im Builder.
            ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));
            self.click_through_until = Some(now + Duration::from_secs(CLICK_THROUGH_SECS));
        }
        if toggle_layered {
            if let Some(h) = h {
                self.layered_on = !self.layered_on;
                unsafe {
                    if self.layered_on {
                        let ex = ex_style(h);
                        self.original_ex_style = Some(ex);
                        SetWindowLongPtrW(h, GWL_EXSTYLE, ex | WS_EX_LAYERED as isize);
                        SetLayeredWindowAttributes(h, 0, self.layered_alpha, LWA_ALPHA);
                    } else if let Some(orig) = self.original_ex_style.take() {
                        SetLayeredWindowAttributes(h, 0, 255, LWA_ALPHA);
                        SetWindowLongPtrW(h, GWL_EXSTYLE, orig);
                    }
                }
            }
        }

        // Leerlauf: nur einmal pro Sekunde neu zeichnen (plus bei Eingaben).
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}
