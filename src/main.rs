#![windows_subsystem = "windows"]

mod ablauf;
mod anzeige;
mod app;
mod einstellungsdatei;
mod einstellungsfenster;
mod grafik;
mod hinweis;
mod menue;
mod overlay;
mod plattform;
mod schriften;
mod theme;
mod themeordner;
mod tray;

use eframe::egui;

fn main() {
    // 1. Panic-Hook als Erstes (SPEC 9)
    std::panic::set_hook(Box::new(|info| {
        let text = format!("Rusty Pomodoro Overlay ist abgestürzt.\n\n{info}");
        let pfad = app::app_ordner().join("absturz.txt");
        let _ = std::fs::create_dir_all(app::app_ordner());
        let _ = std::fs::write(&pfad, &text);
        plattform::meldung(&format!("{text}\n\nDetails: {}", pfad.display()));
    }));

    // 2./3. Einzelinstanz (SPEC 10.1)
    let zeigen = match plattform::einzelinstanz() {
        plattform::Instanz::Erste { zeigen } => zeigen,
        plattform::Instanz::Zweite => return,
    };

    // 5. Overlay unsichtbar und ohne Fokus erzeugen; Lage setzt der erste logic-Aufruf
    let optionen = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rusty Pomodoro Overlay")
            .with_inner_size(overlay::GRUNDGROESSE)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            // eframe zeigt das Fenster nach dem ersten Bild selbst an; mit `active(false)`
            // geschieht das ohne Fokus (SW_SHOWNOACTIVATE)
            .with_visible(false)
            .with_active(false)
            .with_taskbar(false),
        ..Default::default()
    };
    let ergebnis = eframe::run_native(
        "rusty-pomodoro-overlay",
        optionen,
        // 4. Einstellungsdatei öffnen und Theme laden passiert in App::neu
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(schriften::definitionen());
            Ok(Box::new(app::App::neu(&cc.egui_ctx, zeigen)))
        }),
    );
    if let Err(fehler) = ergebnis {
        plattform::meldung(&format!(
            "Rusty Pomodoro Overlay konnte nicht starten.\n\n{fehler}"
        ));
    }
}
