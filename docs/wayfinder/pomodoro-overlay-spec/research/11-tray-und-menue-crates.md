# Recherche: Crates für Tray-Icon und gemeinsames Menü

Ticket: [11 – Crates für Tray-Icon und gemeinsames Menü](../tickets/11-tray-und-menue-crates.md) · Stand: 2026-11-24

## Frage

Mit welchem Crate bekommt die App (eframe/egui 0.37 + glow, winit 0.30, nur Windows 11) ein Tray-Icon mit sekündlich geändertem Tooltip und ein natives Menü? Das Menü soll an beiden Stellen dasselbe sein: Links-/Rechtsklick aufs Tray-Icon und Rechtsklick aufs Overlay. Es braucht Häkchen, ausgegraute Einträge, Trenner, ein Untermenü (*Theme ▸*) und einen Eintragstext, der sich zur Laufzeit ändert (*Primäraktion*). Dazu kommen die Tasten `Esc`, `Umschalt+F10` und die Menütaste. Bewertet wird nach Zuverlässigkeit auf Windows 11, Aufwand und Binärgröße.

## Kurzantwort

**Empfehlung: `tray-icon` 0.26 mit dem mitgelieferten `muda` 0.21 (beide von tauri-apps), ohne Default-Features.**
- Ein `muda::Menu` hängt am Tray-Icon (`with_menu`, `with_menu_on_left_click(true)`). Für das Overlay öffnet `ContextMenu::show_context_menu_for_hwnd(hwnd, None)` **dasselbe Menü** (ein `HMENU`) an der Mausposition. Ein Nachbau in egui ist nicht nötig.
- Alles, was das Menü braucht, gibt es als API: `MenuItem::set_text`, `set_enabled`, `CheckMenuItem::set_checked`, `Submenu`, `PredefinedMenuItem::separator()`. Der Tooltip ändert sich über `TrayIcon::set_tooltip`, das ist ein einfaches `Shell_NotifyIconW(NIM_MODIFY, NIF_TIP)`.
- Es läuft ohne eigenen Thread auf dem Haupt-Thread mit. Die versteckten Fenster des Crates bekommen ihre Nachrichten aus der Message-Pump von winit.
- Im Probelauf (siehe unten) gingen Rechtsklick, Menütaste, `Umschalt+F10` und `Esc`. Die Exe wird dadurch **+161 KB** größer.

Zwei Punkte muss die App selbst richtig machen:
1. **Solange ein natives Menü offen ist, steht die Event-Loop.** Overlay und Tooltip werden in dieser Zeit nicht neu gezeichnet. Der Timer muss deshalb mit Uhrzeit-Differenzen rechnen und darf keine Ticks zählen.
2. **`Umschalt+F10` kommt in egui nur als Loslassen-Ereignis an**, das Drücken fehlt. Die App muss also auf `key_released(F10)` mit Umschalt reagieren oder `WM_CONTEXTMENU` abfangen.

## Geprüfte Versionen (crates.io-API, abgefragt 2026-11-24)

| Crate | Version | Datum | Bemerkung |
|---|---|---|---|
| `tray-icon` | 0.26.1 | 2026-10-07 | tauri-apps, ~47 Mio. Downloads; hängt von `muda` ab und re-exportiert es als `tray_icon::menu` |
| `muda` | 0.21.2 | 2026-10-08 | tauri-apps, ~55 Mio. Downloads |
| `trayicon` | 0.4.1 | 2026-06-02 | Einzelautor, ~64 Tsd. Downloads, Windows-Teil über `winapi` 0.3 |
| `tray-item` | 0.10.0 | 2024-04-07 | lange kein Release mehr, nicht vertieft |
| `windows` / `windows-sys` | 0.62.2 / 0.61.2 | 2025-10-06 | für den Eigenbau |

Im Spike-Baum (`prototype/02-overlay-spike`) stecken eframe 0.37.0, winit 0.30.13 und `windows-sys` 0.52/0.59/0.61. `tray-icon` und `muda` verlangen `windows-sys >=0.60, <=0.61` und nutzen damit die 0.61.2, die eframe/egui-winit schon mitbringen.

## Wie Windows Tray-Icon und Kontextmenü umsetzt (Hintergrund)

- **Tooltip:** `NOTIFYICONDATAW.szTip` fasst „maximum of 128 characters, including the terminating null character“. Geändert wird er mit `NIM_MODIFY` + `NIF_TIP` ([NOTIFYICONDATAW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw)). Bei `NOTIFYICON_VERSION_4` ist der Standard-Tooltip unterdrückt, außer man setzt `NIF_SHOWTIP`. `tray-icon` ruft `NIM_SETVERSION` gar nicht auf und bleibt damit beim alten Verhalten mit Standard-Tooltip.
- **Tray-Menü:** Laut [TrackPopupMenu](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-trackpopupmenu) muss das eigene Fenster vorher Vordergrundfenster sein, „Otherwise, the menu will not disappear when the user clicks outside of the menu“. Danach soll man eine harmlose Nachricht posten (`PostMessage(hDlg, WM_NULL, 0, 0)`), weil das Menü sonst beim zweiten Mal „appears and then immediately disappears“.
- **Explorer-Neustart:** Nach einem Neustart der Taskleiste sendet Windows die Nachricht `TaskbarCreated`. Icons müssen sich dann neu anmelden.
- **Tastatur:** `WM_CONTEXTMENU` entsteht bei Rechtsklick, `SHIFT+F10` und der Menütaste (`VK_APPS`). Kommt es von der Tastatur, sind die Koordinaten −1, und die App soll das Menü an der aktuellen Auswahl zeigen ([WM_CONTEXTMENU](https://learn.microsoft.com/en-us/windows/win32/menurc/wm-contextmenu)). `Esc`, Pfeiltasten, Enter und `&`-Zugriffstasten erledigt die Menüschleife von Windows selbst.
- `TrackPopupMenu` ist **modal**: Es kehrt erst zurück, wenn das Menü zu ist, und pumpt so lange eigene Nachrichten.

## Kandidaten im Einzelnen

### 1. `tray-icon` 0.26 + `muda` 0.21 (Empfehlung)

Quelle für alles Folgende ist der Quellcode der Crates: [tray-icon `platform_impl/windows/mod.rs`](https://docs.rs/crate/tray-icon/0.26.1/source/src/platform_impl/windows/mod.rs), [muda `platform_impl/windows/mod.rs`](https://docs.rs/crate/muda/0.21.2/source/src/platform_impl/windows/mod.rs), [muda `context_menu.rs`](https://docs.rs/crate/muda/0.21.2/source/src/context_menu.rs).

**Tray-Icon**
- Erzeugt ein verstecktes Fenster (`WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW`) im aufrufenden Thread. Ein Kommentar im Code erklärt, dass `WS_EX_TOOLWINDOW` gegen einen Fehler gesetzt ist, bei dem das Fenster nach Stunden oder nach einem Explorer-Neustart in der Taskleiste auftauchte.
- `TaskbarCreated` wird registriert und per `ChangeWindowMessageFilterEx` durchgelassen. Startet die App vor dem Explorer, wartet das Icon auf diese Nachricht und meldet sich dann an.
- `set_tooltip` macht direkt `Shell_NotifyIconW(NIM_MODIFY)` mit `NIF_TIP` und kopiert höchstens 128 UTF-16-Zeichen. Bei genau 128 Zeichen oder mehr fehlt die abschließende Null, also sollte der Text unter 128 bleiben. Der Tooltip aus *Bedienung* („Fokus 2/4 · läuft · 12:34“) ist weit kürzer.
- Menü am Tray: Bei `WM_LBUTTONUP`/`WM_RBUTTONUP` ruft das Crate `SetForegroundWindow` und `TrackPopupMenu` an der Cursorposition auf, danach `PostMessageW(hwnd, WM_NULL, …)`. Das ist genau das Muster von Microsoft. Links- und Rechtsklick lassen sich getrennt schalten (`with_menu_on_left_click`, `with_menu_on_right_click`).
- Weitere API ([docs.rs](https://docs.rs/tray-icon/0.26.1/tray_icon/)): `set_icon` (z. B. ein anderes Icon pro Phase), `set_visible`, `rect`, `window_handle`, `show_menu`, `with_guid`.
- **Event-Loop:** Die Crate-Doku sagt: „On Windows … an event loop must be running on the thread.“ Für winit-Nutzer empfiehlt sie `TrayIconEvent::set_event_handler` / `MenuEvent::set_event_handler`, um die Event-Loop zu wecken. Bei eframe gibt es keinen `EventLoopProxy`, aber `egui::Context::request_repaint()` ist laut [eframe-Doku](https://docs.rs/eframe/0.37.0/eframe/trait.App.html#method.logic) „at any time (e.g. from another thread)“ erlaubt. Es braucht **keinen eigenen Thread**. `GetMessage`/`DispatchMessage` von winit stellt auch die Nachrichten an die versteckten Fenster von `tray-icon` zu.

**Menü (muda)**
- Typen: `MenuItem` (`set_text`, `set_enabled`), `CheckMenuItem` (`set_checked`), `Submenu`, `PredefinedMenuItem::separator()`, `IconMenuItem`. Ein Radio-Item gibt es nicht, *Theme ▸* wird daher aus `CheckMenuItem`s gebaut.
- Unter Windows sind das echte Win32-Menüs: `SetMenuItemInfoW` für Text, `EnableMenuItem` für Ausgrauen, `CheckMenuItem` für Häkchen. Änderungen wirken sofort am selben `HMENU`, das Menü muss man nicht neu bauen.
- `Menu` ist `Clone` (geteiltes `Rc`), also hängt `Box::new(menu.clone())` am Tray, während die App das Original behält. Die Typen sind nicht `Send` und bleiben auf dem Haupt-Thread, was hier ohnehin der Fall ist.
- **Dasselbe Menü am Overlay:** `ContextMenu::show_context_menu_for_hwnd(hwnd, position)` ruft `SetForegroundWindow(hwnd)` und `TrackPopupMenu(TPM_LEFTALIGN | TPM_RETURNCMD)` auf. Mit `None` geht das Menü an der Cursorposition auf, mit `Some(Position)` relativ zur Fensterecke (DPI-umgerechnet über `ClientToScreen`). Die Auswahl kommt als normales `MenuEvent`, so wie beim Tray. Das HWND liefert `raw-window-handle` wie im Spike.
- **Achtung, Häkchen schalten sich selbst um:** Beim Klick auf ein `CheckMenuItem` dreht muda das Häkchen selbst um (`MenuItemAction::Toggle`) und sendet dann das Event. Die App sollte nach jedem Event die Häkchen aus ihrem eigenen Zustand neu setzen, damit Menü und Zustand nicht auseinanderlaufen (z. B. *Click-through*, das nur übers Tray zurückgeht).
- **Kein Dunkelmodus für Popup-Menüs:** muda zeichnet nur die *Menüleiste* dunkel (`dark_menu_bar.rs`). Popup-Menüs sehen aus wie Standard-Windows-Menüs, und weder muda, tray-icon noch winit rufen dafür `SetPreferredAppMode` auf. Ein egui-Theme wirkt also nicht aufs Menü (wichtig für *Theming*).
- Features: Ohne Default-Features (`default-features = false`) entfallen GTK und libappindicator, auf Windows ändert das nichts. Neue Abhängigkeiten: `muda`, `crossbeam-channel`, `dpi`, `keyboard-types`, `once_cell`, `thiserror`. `windows-sys` 0.61 ist schon im Baum.

### 2. `trayicon` 0.4.1

Quelle: [docs.rs](https://docs.rs/trayicon/0.4.1/trayicon/), [Quellcode `sys/windows`](https://docs.rs/crate/trayicon/0.4.1/source/src/sys/windows/).
- Kann Tooltip setzen, Häkchen (`checkable`, `set_menu_item_checkable`), Ausgrauen (`set_menu_item_disabled`), Trenner und Untermenüs. **Eintragstext lässt sich nicht einzeln ändern**, man muss das ganze Menü per `set_menu` neu bauen.
- `show_menu()` postet eine Nachricht an das eigene versteckte Fenster und zeigt das Menü dort an der Cursorposition. Damit ginge es auch fürs Overlay, und das Menü liefe sogar außerhalb von `ui()`. Nach `TrackPopupMenu` fehlt aber das empfohlene `PostMessage(WM_NULL)`. Positionieren für `Umschalt+F10` geht nicht („Currently shows always in mouse cursor coordinates“).
- Unter Windows hängt es an `winapi` 0.3, also einer zweiten Win32-Bindung neben `windows-sys`. Es hat wenige Nutzer.
- Fazit: geht grundsätzlich, ist aber schwächer als tray-icon/muda und hat keinen Vorteil.

### 3. Eigenbau mit `windows`/`windows-sys` (`Shell_NotifyIconW` + `TrackPopupMenu`)

- Damit hätte man volle Kontrolle (z. B. `TrackPopupMenuEx` mit Ausschlussrechteck, `NIF_INFO`-Balloons, `NOTIFYICON_VERSION_4`). Man müsste aber genau das nachbauen, was `tray-icon` schon richtig macht: verstecktes Fenster, `TaskbarCreated`, `SetForegroundWindow` + `WM_NULL`, ID-Vergabe, `WM_COMMAND`-Weiterleitung, DPI-Position, Icon aus RGBA. Grob geschätzt sind das einige hundert Zeilen `unsafe`.
- Bei der Größe bringt es kaum etwas, das ganze Paket tray-icon + muda kostet nur ~161 KB.
- Taugt als Rückfallebene, falls tray-icon/muda an einer Stelle nicht reicht (z. B. Balloon-Hinweis am selben Icon, siehe unten).

### 4. Menü in egui nachbauen

- Für das Overlay ginge es, für das Tray aber nicht sinnvoll: Ist das Overlay ausgeblendet oder auf Click-through, bräuchte das Tray-Menü einen eigenen egui-Viewport (eigenes Fenster an der Taskleiste) samt Fokus- und „Klick daneben schließt“-Logik. Das ist viel Aufwand und weniger zuverlässig als das Systemmenü.
- Zwei verschiedene Menü-Umsetzungen widersprächen außerdem der Vorgabe „dasselbe Menü an beiden Stellen“. **Verworfen.**

## Zusammenspiel mit eframe 0.37 / winit 0.30

- **Wo die Tray-Logik hingehört:** eframe 0.37 hat neben `App::ui` die Methode `App::logic`. Laut Doku läuft sie auch dann, wenn das Fenster versteckt ist. Im Quellcode ([glow_integration.rs](https://docs.rs/crate/eframe/0.37.0/source/src/native/glow_integration.rs)) ruft eframe bei verstecktem Root-Viewport `update_logic_only` auf: „The app logic keeps ticking, so it can e.g. ask to be shown again“. Tooltip setzen, Menüeinträge nachführen und `MenuEvent`s abarbeiten gehört also in `logic()`, sonst funktioniert das Tray bei *Overlay anzeigen = aus* nicht. Mit `ViewportCommand::RequestPaintWhileHidden` kann man auch `ui()` erzwingen.
- **Menü aus `ui()` öffnen ist sicher:** `TrackPopupMenu` läuft dann mitten im Event-Handler von winit. winit 0.30 puffert Ereignisse, die währenddessen eintreffen (`should_buffer` / `event_buffer` in [runner.rs](https://docs.rs/crate/winit/0.30.13/source/src/platform_impl/windows/event_loop/runner.rs)). Auch `WM_PAINT` führt dann nicht zu einem verschachtelten `RedrawRequested` ([event_loop.rs](https://docs.rs/crate/winit/0.30.13/source/src/platform_impl/windows/event_loop.rs), Zweig `WM_PAINT`). Im Probelauf gab es keinen Absturz.
- **Event-Loop steht, solange ein Menü offen ist.** winit sagt das selbst im Code: „When right-click the title bar, the system window menu is presented to the user, and the modal event loop begins. This dummy event does *not* prevent the freeze in the main event loop caused by that popup menu.“ Das gilt für das Overlay-Menü (blockiert `ui()`) und genauso für das Tray-Menü (gleicher Thread). Gemessen: 4 s Menü offen = 4 s ohne `logic()`-Aufruf. Folge: Die Restzeit muss aus `Instant`-Differenzen kommen und darf nicht pro Tick heruntergezählt werden. Während das Menü offen ist, bleiben Overlay-Zahl und Tooltip stehen. Wer das nicht will, müsste Tray und Menü in einen eigenen Thread mit eigener Message-Pump legen. Das geht, weil die muda-Typen dann nur dort leben, macht aber Zustandsabgleich über Kanäle nötig und lohnt sich für v1 nicht.

## Tastatur

- **`Esc`**, Pfeiltasten, Enter und `&`-Zugriffstasten erledigt die Menüschleife von Windows. Im Probelauf hat `Esc` das Menü geschlossen.
- **Menütaste:** winit übersetzt `VK_APPS` in `NamedKey::ContextMenu`, egui-winit macht daraus `egui::Key::Menu` ([egui-winit lib.rs](https://docs.rs/crate/egui-winit/0.37.0/source/src/lib.rs)). `key_pressed(Key::Menu)` geht, im Probelauf getestet.
- **`Umschalt+F10`:** Im Probelauf (synthetische Tastendrücke über `keybd_event`) kam in egui **nur das Loslassen** von F10 mit gesetztem Umschalt an. Das Drücken fehlte, `F10` allein kam vollständig an. Eine mögliche Ursache (nicht bis ins Letzte geprüft): F10 ist eine System-Taste (`WM_SYSKEYDOWN`), die winit an `DefWindowProc` weiterreicht, und `DefWindowProc` erzeugt aus `Umschalt+F10` ein `WM_CONTEXTMENU`. Abhilfe, beide getestet bzw. dokumentiert:
  - `i.modifiers.shift && i.key_released(egui::Key::F10)`. Im Probelauf ging das Menü damit auf.
  - oder das Overlay-HWND subclassen (`SetWindowSubclass`) und `WM_CONTEXTMENU` abfangen. Das deckt `Umschalt+F10` und Menütaste nach der Microsoft-Vorgabe ab (Koordinaten −1 → Menü an fester Stelle des Overlays zeigen).
- Bei Tastatur-Öffnung das Menü mit `Some(Position)` an eine feste Stelle des Overlays legen (z. B. oben links), nicht an den Mauszeiger.

## Probelauf (Scratch-Verzeichnis, nicht im Repo)

Minimal-App: eframe 0.37 (`default-features = false`, `glow`, `default_fonts`) + `tray-icon` 0.26 (`default-features = false`), Release mit LTO, `codegen-units = 1`, `strip`, `panic = "abort"`, MSVC, Windows 11 Pro 26100. Menü mit Primäraktion (Text wechselt jede Sekunde), ausgegrautem Eintrag (wechselt mit), Häkchen, Untermenü *Theme* und Trennern. Tooltip wird jede Sekunde in `logic()` gesetzt. Das Overlay öffnet dasselbe Menü per `show_context_menu_for_hwnd`. Gesteuert wurde per PowerShell (`mouse_event`/`keybd_event`, Suche nach Menüfenster-Klasse `#32768`):

| Prüfung | Ergebnis |
|---|---|
| Rechtsklick aufs Overlay | Menü offen |
| `Esc` | Menü zu |
| Menütaste | Menü offen |
| `Umschalt+F10` mit `key_pressed` | **kein** Menü (nur Loslassen kommt an) |
| `Umschalt+F10` mit `key_released` | Menü offen |
| `logic()`-Aufrufe bei 4 s offenem Menü | Lücke von 4,0 s (3,46 → 7,48 s) |
| Absturz/Re-Entrancy | keiner |
| Binärgröße | ohne Tray 4 513 792 B, mit Tray 4 675 072 B → **+161 KB** |

Nicht automatisiert geprüft: der Klick aufs Tray-Icon selbst (Lage im Infobereich bzw. Überlauf von Windows 11) und ob sich ein gerade angezeigter Tooltip beim `NIM_MODIFY` sichtbar aktualisiert.

## Rangliste

| Rang | Option | Gleiches Menü am Overlay | Laufzeit-Änderungen | Zuverlässigkeit Win 11 | Aufwand | Größe |
|---|---|---|---|---|---|---|
| **1** | **tray-icon 0.26 + muda 0.21** | ja (`show_context_menu_for_hwnd`) | Text, Ausgrauen, Häkchen einzeln | hoch (TaskbarCreated, WM_NULL, große Nutzerbasis über tauri) | klein | +161 KB |
| 2 | trayicon 0.4 | ja, nur an Cursorposition | Text nur durch Neubau | mittel (WM_NULL fehlt, wenige Nutzer) | klein | ungeprüft, zusätzlich `winapi` |
| 3 | Eigenbau windows-sys | ja | alles | so gut wie selbst gebaut | hoch (`unsafe`, mehrere hundert Zeilen) | minimal kleiner |
| 4 | egui-Nachbau | – | alles | niedrig fürs Tray | hoch | – |

## Folgen für spätere Tickets

- **Architektur:** Der Ablauf-Zustand ist die einzige Wahrheit. Pro `logic()`-Tick (und nach jedem `MenuEvent`) werden Primäraktions-Text, Ausgrauen, Häkchen und Tooltip daraus gesetzt. Die Zeit wird aus `Instant` berechnet. Tray-Code steht in `logic()`, nicht in `ui()`.
- **Ausgeblendet / Click-through:** Funktioniert nur, wenn `logic()` bei verstecktem Fenster weiter regelmäßig läuft. `request_repaint_after(1 s)` bei `ViewportCommand::Visible(false)` muss ein Prototyp zeigen (ungeprüft, siehe unten).
- **Phasenwechsel-Hinweis / Benachrichtigungen:** `tray-icon` hat **keine** Balloon-/Toast-API (kein `NIF_INFO`). Es gibt `set_icon` für ein anderes Icon pro Phase und `window_handle()`. Die interne `uID` ist nicht öffentlich, ein eigenes `Shell_NotifyIconW(NIF_INFO)` am selben Icon ginge daher nur mit `with_guid`. Eine GUID ist laut Microsoft an den Pfad der Exe gebunden („The binary file that contains the icon was moved“ → Fehler), was bei `cargo run` gegenüber `cargo install` stört. Toasts bräuchten ein eigenes Crate bzw. WinRT. Das gehört in *Phasenwechsel-Hinweis*.
- **Theming:** Das native Menü folgt nicht dem egui-Theme und ist hell. Ein dunkles Popup-Menü ginge nur über die undokumentierte uxtheme-Funktion `SetPreferredAppMode`. Für v1 so hinnehmen.
- **Single-Instance:** `tray-icon` hilft dabei nicht. Bei zwei Instanzen gäbe es zwei Tray-Icons. Das muss *Lebenszyklus* lösen (z. B. benannter Mutex).
- **Beenden:** Das `TrayIcon` muss vor dem Prozessende gedroppt werden (`Drop` ruft `NIM_DELETE`), sonst bleibt ein Geister-Icon im Infobereich, bis man mit der Maus darüberfährt (bekanntes Verhalten der Shell, hier nicht geprüft). Bei einem Panic mit `panic = "abort"` läuft `Drop` nicht.

## Nicht verifiziert

- Ob sich ein gerade sichtbarer Tray-Tooltip bei `NIM_MODIFY` sofort aktualisiert oder erst beim nächsten Hover (Verhalten der Shell, keine Doku gefunden).
- Tray-Klick (links/rechts) auf echter Hardware, Verhalten im Überlaufbereich von Windows 11.
- Ob `logic()` bei `ViewportCommand::Visible(false)` mit `request_repaint_after` zuverlässig jede Sekunde läuft. eframe-Doku und Code sagen ja, aber unter Windows hängt das am `RedrawWindow` für ein verstecktes Fenster.
- Die genaue Ursache, warum das Drücken bei `Umschalt+F10` nicht in egui ankommt. Getestet wurde nur mit synthetischen Tastendrücken.
