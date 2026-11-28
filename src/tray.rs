//! Adapter für `tray-icon`/`muda` (SPEC 4.4, 4.5, 7.6): überträgt das reine `Menuemodell`
//! auf das eine native Menü, setzt Tooltip und Icon und ordnet Menü-Klicks den Aktionen zu.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui;
use tray_icon::menu::dpi::{LogicalPosition, Position};
use tray_icon::menu::{
    CheckMenuItem, ContextMenu as _, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem,
    PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::Foundation::HWND;

use crate::menue::{Aktion, Eintrag, Menuemodell};

/// Ein Menüeintrag, wie er in `muda` steckt.
enum Punkt {
    Normal(MenuItem),
    Haekchen(CheckMenuItem),
}

impl Punkt {
    fn neu(e: &Eintrag) -> Self {
        match e.haekchen {
            None => Self::Normal(MenuItem::new(&e.text, e.aktiv, None)),
            Some(an) => Self::Haekchen(CheckMenuItem::new(&e.text, e.aktiv, an, None)),
        }
    }

    fn id(&self) -> &MenuId {
        match self {
            Self::Normal(m) => m.id(),
            Self::Haekchen(m) => m.id(),
        }
    }

    fn als_eintrag(&self) -> &dyn IsMenuItem {
        match self {
            Self::Normal(m) => m,
            Self::Haekchen(m) => m,
        }
    }

    /// Bringt Text, Ausgrauen und Häkchen auf den Stand von `e`. Häkchen werden immer gesetzt,
    /// weil `muda` sie beim Klick selbst umschaltet.
    fn angleichen(&self, alt: &Eintrag, e: &Eintrag) {
        match self {
            Self::Normal(m) => {
                if alt.text != e.text {
                    m.set_text(&e.text);
                }
                if alt.aktiv != e.aktiv {
                    m.set_enabled(e.aktiv);
                }
            }
            Self::Haekchen(m) => {
                if alt.text != e.text {
                    m.set_text(&e.text);
                }
                if alt.aktiv != e.aktiv {
                    m.set_enabled(e.aktiv);
                }
                m.set_checked(e.haekchen == Some(true));
            }
        }
    }
}

pub struct Tray {
    icon: TrayIcon,
    menue: Menu,
    /// in Menüreihenfolge, zusammen mit dem Eintrag, aus dem sie zuletzt gesetzt wurden
    punkte: Vec<(Punkt, Eintrag)>,
    aktionen: HashMap<MenuId, Aktion>,
    klicks: Receiver<MenuId>,
    tooltip: String,
}

impl Tray {
    /// Legt Tray-Icon und Menü an. Menü-Klicks wecken die Oberfläche über `ctx`.
    pub fn neu(ctx: &egui::Context, modell: &Menuemodell, icon: Icon) -> Option<Self> {
        let (sender, klicks) = channel();
        let wecken = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            let _ = sender.send(e.id);
            wecken.request_repaint();
        }));
        // Klicks aufs Icon öffnen das Menü selbst; die Ereignisse würden sonst nur auflaufen
        TrayIconEvent::set_event_handler(Some(|_| {}));

        let menue = Menu::new();
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menue.clone()))
            .with_menu_on_left_click(true)
            .with_icon(icon)
            .build()
            .ok()?;
        let mut tray = Self {
            icon,
            menue,
            punkte: Vec::new(),
            aktionen: HashMap::new(),
            klicks,
            tooltip: String::new(),
        };
        tray.aufbauen(modell);
        Some(tray)
    }

    /// Die seit dem letzten Aufruf angeklickten Einträge.
    pub fn aktionen(&self) -> Vec<Aktion> {
        self.klicks
            .try_iter()
            .filter_map(|id| self.aktionen.get(&id).cloned())
            .collect()
    }

    /// Überträgt das Modell. Ändert sich der Aufbau (Fehlerzeilen, Themes), wird neu
    /// aufgebaut, sonst werden nur Text, Ausgrauen und Häkchen angeglichen.
    pub fn menue_anwenden(&mut self, modell: &Menuemodell) {
        let neu = eintraege(modell);
        // Gleiche Aktionen in gleicher Reihenfolge und gleicher Art legen den Aufbau fest:
        // die Primäraktion steht hinter den Fehlerzeilen, die eingebauten Themes sind fest.
        let gleicher_aufbau = neu.len() == self.punkte.len()
            && neu.iter().zip(&self.punkte).all(|(n, (_, alt))| {
                n.aktion == alt.aktion && n.haekchen.is_some() == alt.haekchen.is_some()
            });
        if !gleicher_aufbau {
            self.aufbauen(modell);
            return;
        }
        for ((punkt, alt), e) in self.punkte.iter_mut().zip(neu) {
            punkt.angleichen(alt, e);
            *alt = e.clone();
        }
    }

    pub fn tooltip_setzen(&mut self, text: &str) {
        if self.tooltip != text {
            let _ = self.icon.set_tooltip(Some(text));
            self.tooltip = text.to_owned();
        }
    }

    pub fn icon_setzen(&self, icon: Icon) {
        let _ = self.icon.set_icon(Some(icon));
    }

    /// Öffnet dasselbe Menü am Overlay: an der Mausposition oder an `stelle` (logische
    /// Pixel relativ zur oberen linken Ecke). Kehrt erst zurück, wenn es zu ist.
    pub fn menue_zeigen(&self, hwnd: HWND, stelle: Option<(f64, f64)>) {
        let position = stelle.map(|(x, y)| Position::Logical(LogicalPosition::new(x, y)));
        // Sicherheit: `hwnd` ist das lebende Overlay-Fenster dieses Threads
        unsafe {
            self.menue
                .show_context_menu_for_hwnd(hwnd as isize, position);
        }
    }

    fn aufbauen(&mut self, modell: &Menuemodell) {
        while self.menue.remove_at(0).is_some() {}
        self.punkte = eintraege(modell)
            .into_iter()
            .map(|e| (Punkt::neu(e), e.clone()))
            .collect();
        self.aktionen = self
            .punkte
            .iter()
            .map(|(p, e)| (p.id().clone(), e.aktion.clone()))
            .collect();

        let mut punkte = self.punkte.iter().map(|(p, _)| p.als_eintrag());
        let mut naechste = |n: usize| -> Vec<&dyn IsMenuItem> { punkte.by_ref().take(n).collect() };
        let trenner = || PredefinedMenuItem::separator();

        let fehler = naechste(modell.fehler.len());
        if !fehler.is_empty() {
            let _ = self.menue.append_items(&fehler);
            let _ = self.menue.append(&trenner());
        }
        let _ = self.menue.append_items(&naechste(4));
        let _ = self.menue.append(&trenner());
        let _ = self.menue.append_items(&naechste(4));

        let theme = Submenu::new("Theme", true);
        let _ = theme.append_items(&naechste(modell.eingebaute_themes.len()));
        let _ = theme.append(&trenner());
        let eigene = naechste(modell.eigene_themes.len());
        if !eigene.is_empty() {
            let _ = theme.append_items(&eigene);
            let _ = theme.append(&trenner());
        }
        let _ = theme.append_items(&naechste(1));
        let _ = self.menue.append(&theme);

        let _ = self.menue.append_items(&naechste(1));
        let _ = self.menue.append(&trenner());
        let _ = self.menue.append_items(&naechste(1));
    }
}

/// Alle Einträge in Menüreihenfolge (ohne Trenner und Untermenü).
fn eintraege(m: &Menuemodell) -> Vec<&Eintrag> {
    let mut v: Vec<&Eintrag> = m.fehler.iter().collect();
    v.extend([
        &m.primaeraktion,
        &m.ueberspringen,
        &m.phase_zuruecksetzen,
        &m.neu_beginnen,
        &m.immer_im_vordergrund,
        &m.click_through,
        &m.overlay_anzeigen,
        &m.ton,
    ]);
    v.extend(&m.eingebaute_themes);
    v.extend(&m.eigene_themes);
    v.extend([&m.theme_ordner_oeffnen, &m.einstellungen, &m.beenden]);
    v
}
