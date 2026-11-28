//! Alles, was direkt mit Windows spricht (SPEC 11.7): freie Funktionen über `windows-sys`,
//! ohne Traits. Rechtecke in physischen Pixeln des virtuellen Bildschirms.

use std::ffi::c_void;
use std::path::Path;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_OBJECT_0,
    WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::{
    DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, MONITOR_DEFAULTTOPRIMARY,
    MONITORINFO, MonitorFromPoint, MonitorFromRect,
};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
use windows_sys::Win32::System::Threading::{
    CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, OpenEventW, SetEvent, WaitForSingleObject,
};
use windows_sys::Win32::UI::HiDpi::{GetDpiForSystem, GetDpiForWindow, GetSystemMetricsForDpi};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
};
use windows_sys::Win32::UI::Shell::{
    DefSubclassProc, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
    SHQueryUserNotificationState, SetWindowSubclass, ShellExecuteW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, GetWindowRect, IsWindowVisible, MB_ICONERROR, MB_OK,
    MessageBoxW, SM_CXSMICON, STYLESTRUCT, SW_HIDE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
    SWP_NOACTIVATE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_ENDSESSION,
    WM_STYLECHANGING, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use crate::einstellungsdatei::Sicherung;

const MUTEX_NAME: &str = r"Local\rusty_pomodoro_overlay";
const EVENT_NAME: &str = r"Local\rusty_pomodoro_overlay_zeigen";

/// Ein Fensterrechteck in physischen Pixeln.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rechteck {
    pub x: i32,
    pub y: i32,
    pub breite: i32,
    pub hoehe: i32,
}

impl Rechteck {
    fn von(r: RECT) -> Self {
        Self {
            x: r.left,
            y: r.top,
            breite: r.right - r.left,
            hoehe: r.bottom - r.top,
        }
    }

    fn als_rect(self) -> RECT {
        RECT {
            left: self.x,
            top: self.y,
            right: self.x + self.breite,
            bottom: self.y + self.hoehe,
        }
    }

    pub fn mitte(self) -> (i32, i32) {
        (self.x + self.breite / 2, self.y + self.hoehe / 2)
    }

    /// Schiebt das Rechteck so wenig wie nötig in `bereich`.
    fn einpassen(self, bereich: Rechteck) -> Self {
        let passe = |pos: i32, groesse: i32, anfang: i32, laenge: i32| {
            pos.min(anfang + laenge - groesse).max(anfang)
        };
        Self {
            x: passe(self.x, self.breite, bereich.x, bereich.breite),
            y: passe(self.y, self.hoehe, bereich.y, bereich.hoehe),
            ..self
        }
    }
}

fn breit(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// HWND des Haupt-Viewports.
pub fn hwnd_von(fenster: &impl HasWindowHandle) -> Option<HWND> {
    match fenster.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as HWND),
        _ => None,
    }
}

// --- Einzelinstanz (SPEC 10.1, 10.2) ---------------------------------------------------

pub enum Instanz {
    /// Diese Instanz läuft; das Event meldet einen zweiten Start.
    Erste { zeigen: HANDLE },
    /// Es läuft schon eine; sie wurde benachrichtigt.
    Zweite,
}

/// Mutex anlegen. Gibt es ihn schon, das Event der laufenden Instanz setzen.
/// Mutex und Event leben bis zum Prozessende (Windows gibt sie dann frei).
pub fn einzelinstanz() -> Instanz {
    unsafe {
        let mutex = CreateMutexW(std::ptr::null(), 0, breit(MUTEX_NAME).as_ptr());
        if !mutex.is_null() && GetLastError() == ERROR_ALREADY_EXISTS {
            let event = OpenEventW(EVENT_MODIFY_STATE, 0, breit(EVENT_NAME).as_ptr());
            if !event.is_null() {
                SetEvent(event);
            }
            return Instanz::Zweite;
        }
        // Auto-Reset, nicht gesetzt
        let zeigen = CreateEventW(std::ptr::null(), 0, 0, breit(EVENT_NAME).as_ptr());
        Instanz::Erste { zeigen }
    }
}

/// Ob seit der letzten Abfrage ein zweiter Start angeklopft hat.
pub fn zweiter_start_angefragt(zeigen: HANDLE) -> bool {
    !zeigen.is_null() && unsafe { WaitForSingleObject(zeigen, 0) } == WAIT_OBJECT_0
}

// --- Fenster ---------------------------------------------------------------------------

/// Keine Taskleisten-Schaltfläche, nicht in Alt+Tab; ohne den hellen Rahmen von Windows 11.
/// Die Ecken rundet Windows (`DWMWCP_ROUND`): Es schneidet sie wirklich aus. Selbst
/// gezeichnete Rundungen funktionieren nicht, weil Pixel außerhalb davon schwarz statt
/// durchsichtig erscheinen. winit setzt den erweiterten Stil bei
/// jeder eigenen Änderung neu; deshalb hält [`unterklasse_anhaengen`] ihn fest.
pub fn als_werkzeugfenster(hwnd: HWND) {
    unsafe {
        let stil = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, werkzeugstil(stil) as isize);
        let ecken = DWMWCP_ROUND;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&raw const ecken).cast(),
            4,
        );
        let rahmen = DWMWA_COLOR_NONE;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            (&raw const rahmen).cast(),
            4,
        );
    }
}

fn werkzeugstil(stil: u32) -> u32 {
    (stil | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW
}

/// Zeigt das Fenster, ohne ihm den Fokus zu geben.
pub fn ist_sichtbar(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) != 0 }
}

pub fn zeigen_ohne_fokus(hwnd: HWND) {
    unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
}

pub fn ausblenden(hwnd: HWND) {
    unsafe { ShowWindow(hwnd, SW_HIDE) };
}

pub fn fenster_rechteck(hwnd: HWND) -> Option<Rechteck> {
    let mut r = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (unsafe { GetWindowRect(hwnd, &mut r) } != 0).then(|| Rechteck::von(r))
}

pub fn fenster_setzen(hwnd: HWND, r: Rechteck) {
    unsafe {
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            r.x,
            r.y,
            r.breite,
            r.hoehe,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// Anzeigeskalierung des Monitors, auf dem das Fenster liegt (1.0 = 96 dpi).
pub fn dpi_faktor(hwnd: HWND) -> f32 {
    match unsafe { GetDpiForWindow(hwnd) } {
        0 => 1.0,
        dpi => dpi as f32 / 96.0,
    }
}

/// Kantenlänge des Tray-Icons in Pixeln (16/20/24/32 je nach Skalierung, SPEC 7.6).
pub fn tray_icon_groesse() -> u32 {
    let n = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem()) };
    u32::try_from(n).ok().filter(|&n| n > 0).unwrap_or(16)
}

fn arbeitsbereich(monitor: *mut c_void) -> Option<Rechteck> {
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    (unsafe { GetMonitorInfoW(monitor, &mut info) } != 0).then(|| Rechteck::von(info.rcWork))
}

/// Startposition: unten rechts im Arbeitsbereich des Hauptmonitors mit `abstand` Pixeln.
pub fn startposition(breite: i32, hoehe: i32, abstand: i32) -> (i32, i32) {
    let haupt = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
    let bereich = arbeitsbereich(haupt).unwrap_or(Rechteck {
        x: 0,
        y: 0,
        breite: 1920,
        hoehe: 1080,
    });
    (
        bereich.x + bereich.breite - breite - abstand,
        bereich.y + bereich.hoehe - hoehe - abstand,
    )
}

/// Wohin das Fenster muss, damit es ganz im Arbeitsbereich eines Monitors liegt (SPEC 5.6).
/// `None`, wenn es schon passt. Liegt es auf keinem Monitor, kommt es an `start`.
pub fn in_arbeitsbereich(r: Rechteck, start: (i32, i32)) -> Option<Rechteck> {
    let rect = r.als_rect();
    let ziel = if unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) }.is_null() {
        Rechteck {
            x: start.0,
            y: start.1,
            ..r
        }
    } else {
        let naechster = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST) };
        r.einpassen(arbeitsbereich(naechster)?)
    };
    (ziel != r).then_some(ziel)
}

/// Ob gerade eine Maustaste gedrückt ist (dann nicht verschieben).
pub fn maustaste_gedrueckt() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
        .iter()
        .any(|&t| unsafe { GetAsyncKeyState(i32::from(t)) } < 0)
}

// --- Ton (SPEC 6.2) --------------------------------------------------------------------

/// Kein Ton bei Vollbild, D3D-Vollbild und Präsentation. Jedes Mal neu fragen.
pub fn stoeren_erlaubt() -> bool {
    let mut zustand = 0;
    if unsafe { SHQueryUserNotificationState(&mut zustand) } < 0 {
        return true;
    }
    !matches!(
        zustand,
        QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE
    )
}

/// Spielt eine eingebettete WAV asynchron; ohne Audiogerät passiert still nichts.
pub fn klang_abspielen(wav: &'static [u8]) {
    unsafe {
        PlaySoundW(
            wav.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        )
    };
}

// --- Autostart (SPEC 10.3) ---------------------------------------------------------------

const RUN_SCHLUESSEL: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_WERT: &str = "RustyPomodoroOverlay";

/// Ob der Autostart-Eintrag existiert.
pub fn autostart_an() -> bool {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
    let ergebnis = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            breit(RUN_SCHLUESSEL).as_ptr(),
            breit(RUN_WERT).as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    ergebnis == 0
}

/// Legt den Eintrag mit dem Pfad der laufenden Exe an oder löscht ihn. Liefert, ob es geklappt hat.
pub fn autostart_setzen(an: bool) -> bool {
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RegDeleteKeyValueW, RegSetKeyValueW,
    };
    let schluessel = breit(RUN_SCHLUESSEL);
    let wert = breit(RUN_WERT);
    if !an {
        let e =
            unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, schluessel.as_ptr(), wert.as_ptr()) };
        // ERROR_FILE_NOT_FOUND: war schon weg
        return e == 0 || e == 2;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let daten = breit(&format!("\"{}\"", exe.display()));
    let e = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            schluessel.as_ptr(),
            wert.as_ptr(),
            REG_SZ,
            daten.as_ptr().cast(),
            (daten.len() * 2) as u32,
        )
    };
    e == 0
}

// --- Shell -----------------------------------------------------------------------------

/// Öffnet eine Datei mit dem Standardprogramm (`open`) bzw. einen Ordner im Explorer
/// (`explore`). Liefert, ob es geklappt hat.
pub fn oeffnen(pfad: &Path, verb: &str) -> bool {
    let pfad = breit(&pfad.to_string_lossy());
    let ergebnis = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            breit(verb).as_ptr(),
            pfad.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    ergebnis as isize > 32
}

/// Fehlermeldung als MessageBox (nur bei Absturz und Startfehler).
pub fn meldung(text: &str) {
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            breit(text).as_ptr(),
            breit("Rusty Pomodoro Overlay").as_ptr(),
            MB_OK | MB_ICONERROR,
        )
    };
}

// --- Unterklasse der Fensterprozedur --------------------------------------------------

/// Hängt sich an die Fensterprozedur: hält den Werkzeugfenster-Stil fest und sichert die
/// Einstellungen bei `WM_ENDSESSION` (SPEC 10.4).
pub fn unterklasse_anhaengen(hwnd: HWND, sicherung: Sicherung) {
    // Lebt bis zum Prozessende (absichtlich nie freigegeben).
    let zeiger = Box::into_raw(Box::new(sicherung));
    unsafe { SetWindowSubclass(hwnd, Some(unterklasse), 1, zeiger as usize) };
}

unsafe extern "system" fn unterklasse(
    hwnd: HWND,
    nachricht: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    daten: usize,
) -> LRESULT {
    match nachricht {
        WM_STYLECHANGING if wparam as i32 == GWL_EXSTYLE => {
            let stil = unsafe { &mut *(lparam as *mut STYLESTRUCT) };
            stil.styleNew = werkzeugstil(stil.styleNew);
        }
        WM_ENDSESSION if wparam != 0 => {
            let sicherung = unsafe { &*(daten as *const Sicherung) };
            sicherung.sichern();
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, nachricht, wparam, lparam) }
}
