# Recherche: Ton abspielen und „nicht stören“ erkennen

Ticket: [12 – Ton abspielen und „nicht stören“ erkennen](../tickets/12-ton-und-nicht-stoeren.md) · Stand: 2026-11-24

## Frage

1. **Ton:** Wie spielt die App (eframe/egui 0.37 + glow, winit, `tray-icon` 0.26, nur Windows 11) eine kleine, per `include_bytes!` eingebettete `.wav` asynchron ab, ohne die Event-Loop zu blockieren? Bewertet nach Aufwand, Binärgröße und Zuverlässigkeit. Dazu: Reicht ein `'static`-Puffer für `SND_MEMORY | SND_ASYNC`, und folgt der Ton der Lautstärke der App im Lautstärkemixer?
2. **„Nicht stören“:** Was lässt sich mit **dokumentierter** API erkennen: *Bitte nicht stören* von Windows 11, Präsentationsmodus, Vollbild-App? Und wie teuer ist eine Abfrage beim Phasenwechsel?

## Kurzantwort

**Ton: `PlaySoundW(ptr, null, SND_MEMORY | SND_ASYNC | SND_NODEFAULT)` aus `windows-sys` 0.61 (Feature `Win32_Media_Audio`), ohne `SND_SYSTEM`.**
- Kein neues Crate, `windows-sys` 0.61 ist schon im Baum. Im Messprojekt +17 KB, `rodio` 0.23 (nur `playback` + `wav`) dagegen +652 KB und 49 Crates im Baum.
- Der Puffer aus `include_bytes!` liegt in der Exe und ist `'static`, damit ist die Bedingung von Microsoft erfüllt („memory buffer … must remain valid until the sound has completed playing“).
- Ohne `SND_SYSTEM` landet der Ton in der Standard-Audiositzung des Prozesses, also im Lautstärkemixer unter der App. Mit `SND_SYSTEM` würde er dem Regler *Systemsounds* folgen.
- Der erste Aufruf kehrte nach 22,6 ms zurück (Winmm/Audiogerät wird geladen), jeder weitere nach ~16 µs. Das ist höchstens ein verzögerter Frame.

**„Nicht stören“: `SHQueryUserNotificationState` (Feature `Win32_UI_Shell`) erkennt Vollbild-Apps, D3D-Exklusivvollbild und die Präsentationseinstellungen. *Bitte nicht stören* von Windows 11 erkennt es nicht.**
- `QUNS_QUIET_TIME` ist **nicht** *Bitte nicht stören*. Laut Doku ist das die erste Stunde nach der ersten Anmeldung eines Nutzers bzw. nach einem OS-Upgrade.
- Für *Bitte nicht stören* (manuell, nach Zeitplan) gibt es **keine dokumentierte Abfrage**. Die einzige dokumentierte Teilabdeckung ist `FocusSessionManager.IsFocusActive` (WinRT): Sie meldet eine laufende *Fokussitzung*, die *Bitte nicht stören* einschaltet, aber kein von Hand eingeschaltetes *Bitte nicht stören*. WNF-Zustände sind undokumentiert und scheiden aus.
- Eine Abfrage kostet ~0,08 ms (`SHQueryUserNotificationState`) bzw. ~0,7 ms (`IsFocusActive`). Man fragt also einfach beim Phasenwechsel direkt vor dem Ton.

## Geprüfte Versionen

| Crate | Version | Bemerkung |
|---|---|---|
| `windows-sys` | 0.61.2 | `PlaySoundW` in `Win32::Media::Audio` (link auf `winmm.dll`), `SHQueryUserNotificationState` + `QUNS_*` in `Win32::UI::Shell` (link auf `shell32.dll`), geprüft im Quelltext des Crates |
| `rodio` | 0.23.0 (2026-10-11) | crates.io-API; hängt an `cpal` 0.18 und `symphonia` 0.5; Default-Features ziehen MP3, FLAC, MP4/AAC, Vorbis, Aufnahme und Dither mit |
| `windows` | 0.62 | nur für WinRT (`UI_Shell` → `FocusSessionManager`), `windows-sys` kennt kein WinRT |

## Teil 1: Ton

### `PlaySoundW` laut Microsoft

Aus der Referenz [PlaySound function](https://learn.microsoft.com/en-us/previous-versions/dd743680(v=vs.85)) (archiviert, aber die einzige Referenzseite; `PlaySoundW` liegt in `Winmm.dll`):

- `SND_ASYNC`: „The sound is played asynchronously and **PlaySound** returns immediately after beginning the sound.“ Stoppen mit `PlaySound(NULL, 0, 0)`.
- `SND_MEMORY`: „The *pszSound* parameter points to a sound loaded in memory.“ Der Zeiger zeigt also auf eine vollständige WAV-Datei (RIFF-Header + Daten), nicht auf einen String. In Rust: `BEEP.as_ptr() as *const u16`.
- Remarks: „If you combine the **SND_MEMORY** and **SND_ASYNC** flags, the memory buffer that contains the sound must remain valid until the sound has completed playing.“ `static BEEP: &[u8] = include_bytes!("…")` liegt in der Exe und lebt bis zum Prozessende, das ist erfüllt.
- `SND_NODEFAULT`: Wird der Ton nicht gefunden, spielt Windows sonst den Standard-Systemklang. Mit dem Flag bleibt es still.
- Ohne `SND_NOSTOP` gilt: „**PlaySound** attempts to stop any sound that is currently playing in the same process.“ Ein zweiter Hinweis bricht den ersten also ab, für zwei kurze Klänge ist das richtig.
- `SND_SYSTEM`: „the sound is assigned to the audio session for system notification sounds … If this flag is not set, the sound is assigned to the default audio session for the application's process.“ Ohne das Flag regelt also der Eintrag der App im Lautstärkemixer, so wie es *Hinweis bei Phasenwechsel* festlegt.
- `SND_NOWAIT` und `SND_PURGE` sind „Not supported“.
- Rückgabe `FALSE`, wenn z. B. kein Audiogerät da ist. Die App ignoriert das.

Weil die Klänge eingebettet sind, spielen Soundschema und die Einstellung „Keine Sounds“ von Windows keine Rolle. Die betreffen nur Aliase für Systemereignisse.

### Messung

Ein Wegwerf-Projekt im Scratchpad (nicht im Repo), Release, `lto = true`, `strip = true`, `codegen-units = 1`, eine 13 KB große WAV (22,05 kHz, 16 bit, mono, 0,3 s) per `include_bytes!`:

| Variante | Exe | Δ zur Basis | Crates im Baum |
|---|---|---|---|
| Basis (nur `include_bytes!` + `println!`) | 121.856 B | – | 1 |
| `PlaySoundW` über `windows-sys` 0.61 | 138.752 B | **+17 KB** | 3 (`windows-sys`, `windows-link`) |
| `rodio` 0.23, `default-features = false`, `playback` + `wav` | 773.632 B | **+652 KB** | 50 |

Zeiten (ein Lauf, Windows 11 26100):
- `PlaySoundW` erster Aufruf: kehrt nach **22,6 ms** zurück, zweiter Aufruf nach **15,7 µs**. Ton war hörbar.
- `rodio`: Ausgabegerät öffnen 59 ms, Abspielen danach 0,14 ms. Der `DeviceSink` muss die ganze Laufzeit gehalten werden (beim Drop bricht der Ton ab), und `cpal` hält dafür einen eigenen Audio-Thread offen.

In der echten App ist `windows-sys` 0.61 schon vorhanden. Es kommt nur das Feature `Win32_Media_Audio` dazu, der Zuwachs dürfte also noch kleiner sein.

### Bewertung

| | `PlaySoundW` (`windows-sys`) | `rodio` |
|---|---|---|
| Aufwand | 1 `unsafe`-Aufruf, keine Zustände | Sink beim Start öffnen und halten, Decoder pro Ton |
| Größe | +17 KB, kein neues Crate | +652 KB, 49 Crates (einige davon in der App schon vorhanden) |
| Zuverlässigkeit | seit Windows 2000 im System, nimmt beim Aufruf das aktuelle Standardgerät | `cpal`/WASAPI, eigener Thread; Gerätewechsel muss die App selbst behandeln |
| Lautstärkemixer | Sitzung des Prozesses (ohne `SND_SYSTEM`) | Sitzung des Prozesses |
| Mischen mehrerer Töne | nein, neuer Ton stoppt den alten | ja (nicht gebraucht) |

**Empfehlung:** `PlaySoundW` mit `SND_MEMORY | SND_ASYNC | SND_NODEFAULT`. `rodio` lohnt sich erst, wenn die App mischen, Formate dekodieren oder einen eigenen Lautstärkeregler haben soll. Das ist in v1 ausdrücklich nicht gewollt.

Die 22 ms beim ersten Aufruf fallen in den Frame des Phasenwechsels. Wer das vermeiden will, ruft `PlaySoundW` in einem kurzlebigen `std::thread::spawn` auf. Nötig ist das nicht.

## Teil 2: „Nicht stören“

### `SHQueryUserNotificationState`

[SHQueryUserNotificationState](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shqueryusernotificationstate) (Shell32, ab Vista) „Checks the state of the computer for the current user to determine whether sending a notification is appropriate.“ Benachrichtigungen sollen nur bei `QUNS_ACCEPTS_NOTIFICATIONS` kommen. Dazu: „Top-level windows receive a WM_SETTINGCHANGE message when the user turns presentation settings on or off … Note that there are no notifications sent when the user starts or stops a full-screen application.“ Man muss also im Moment des Tons abfragen, ein Zwischenspeichern hilft nicht.

Die Werte laut [QUERY_USER_NOTIFICATION_STATE](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state):

| Wert | Name | Doku (gekürzt) | Ton? (Vorschlag) |
|---|---|---|---|
| 1 | `QUNS_NOT_PRESENT` | Bildschirmschoner läuft, Rechner gesperrt oder inaktive Sitzung (Benutzerwechsel) | spielen: Der Nutzer ist weg, nicht beschäftigt |
| 2 | `QUNS_BUSY` | „A full-screen application is running or Presentation Settings are applied.“ | **unterdrücken** |
| 3 | `QUNS_RUNNING_D3D_FULL_SCREEN` | Direct3D-App im Exklusivvollbild | **unterdrücken** |
| 4 | `QUNS_PRESENTATION_MODE` | „The user has activated Windows presentation settings to block notifications and pop-up messages.“ | **unterdrücken** |
| 5 | `QUNS_ACCEPTS_NOTIFICATIONS` | keiner der anderen Zustände | spielen |
| 6 | `QUNS_QUIET_TIME` | ab Windows 7: „the first hour after a new user logs into his or her account for the first time“, auch nach OS-Upgrade oder Neuinstallation | spielen: kein Wunsch des Nutzers |
| 7 | `QUNS_APP` | ab Windows 8: „A Windows Store app is running.“ | spielen: Bedeutung unter Windows 10/11 unklar |

**Wichtig:** Kein Wert steht für *Bitte nicht stören* bzw. den früheren *Benachrichtigungsassistenten*. Die Enumeration ist seit Windows 8 unverändert. `QUNS_QUIET_TIME` klingt danach, meint aber nur die erste Stunde nach Einrichtung oder Upgrade. Die Doku sagt außerdem, dass `QUNS_QUIET_TIME` hinter jedem der Sperr-Zustände 1 bis 4 zurücktritt.

Probelauf (PowerShell mit P/Invoke, Windows 11 26100): Rückgabe `S_OK`, Zustand `5`. **1000 Aufrufe in 83 ms, also ~0,08 ms pro Abfrage.** Ein zweiter Versuch mit einem randlosen, maximierten Fenster lieferte durchgehend `1` (`QUNS_NOT_PRESENT`), weil der Rechner inzwischen gesperrt war. Ob ein randloses Vollbildfenster (z. B. Vollbild-Video im Browser) `QUNS_BUSY` auslöst, ist damit **nicht gemessen**.

### *Bitte nicht stören* in Windows 11

Laut [Notifications and Do Not Disturb in Windows](https://support.microsoft.com/en-us/windows/notifications-and-do-not-disturb-in-windows-feeca47f-0baf-5680-16f0-8801db1a8466) schickt *Bitte nicht stören* Benachrichtigungen ohne Banner direkt ins Benachrichtigungscenter (Ausnahmen: Wecker, Erinnerungen, gewählte Apps). Automatisch einschalten lässt es sich unter *System > Benachrichtigungen*: zu festen Zeiten, beim Duplizieren des Bildschirms, beim Spielen, bei Apps im Vollbild und in der ersten Stunde nach einem Funktionsupdate. Laut [Focus](https://support.microsoft.com/windows/how-to-use-focus-in-windows-11-cbcc9ddb-8164-43fa-8919-b9a2af072382) schaltet auch eine *Fokussitzung* *Bitte nicht stören* ein.

Abgleich mit dem, was die App erkennt:

| Auslöser von *Bitte nicht stören* | dokumentiert erkennbar? |
|---|---|
| von Hand eingeschaltet | **nein** |
| feste Zeiten | **nein** |
| App im Vollbild | ja, `QUNS_BUSY` (Heuristik von Windows, nicht selbst gemessen) |
| Spiel (D3D-Exklusivvollbild) | ja, `QUNS_RUNNING_D3D_FULL_SCREEN`; randlose Spiele vermutlich als `QUNS_BUSY` |
| Bildschirm duplizieren | **nicht belegt**: Die Präsentationseinstellungen (`QUNS_PRESENTATION_MODE`) sind ein anderes Feature |
| erste Stunde nach Funktionsupdate | ähnlich `QUNS_QUIET_TIME`, für die App aber unerheblich |
| Fokussitzung | ja, `FocusSessionManager.IsFocusActive` (siehe unten) |

Es wird also nicht der Schalter *Bitte nicht stören* selbst erkannt, sondern einige der Situationen, in denen Windows ihn automatisch setzt.

### WinRT

- [`Windows.UI.Shell.FocusSessionManager`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.shell.focussessionmanager): „Provides APIs that allow apps to manage the state of the Focus Session feature of Windows.“ Lesend: `IsSupported`, `GetDefault()`, [`IsFocusActive`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.shell.focussessionmanager.isfocusactive) („Gets a value indicating if a focus session is active on the device“) und das Ereignis `IsFocusActiveChanged`. Nur `TryStartFocusSession` und `DeactivateFocus` sind *Limited Access Features*. Eingeführt in 10.0.23504, UniversalApiContract v15. Im Probelauf ging das **ohne Paketidentität** (unpackaged Rust-Exe mit `windows` 0.62, Feature `UI_Shell`): `IsSupported = true`, `IsFocusActive = false`, erster Aufruf 16 ms, danach ~0,7 ms pro Abfrage, +12 KB. Es meldet aber nur Fokussitzungen, kein von Hand eingeschaltetes *Bitte nicht stören*. Wer einen Pomodoro-Timer nutzt, startet selten zusätzlich eine Fokussitzung in Windows.
- [`UserNotificationListener`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.management.usernotificationlistener): liest und verwaltet die Benachrichtigungen des Nutzers nach ausdrücklicher Erlaubnis (`RequestAccessAsync`). Einen Zustand *Bitte nicht stören* hat die Klasse nicht. Scheidet aus.

### WNF (nur zur Abgrenzung)

Der tatsächliche Zustand von *Bitte nicht stören* lässt sich nur über die Windows Notification Facility lesen. Das npm-Paket [`notification-state-ffi`](https://app.unpkg.com/notification-state-ffi@1.2.0/files/README.md) nutzt dafür neben `SHQueryUserNotificationState` die WNF und warnt: „WNF (Windows Notification Facility) is an undocumented Windows API … This API can change/break at any time in the future.“ Nach der Regel aus *Hinweis bei Phasenwechsel* scheidet das aus.

## Empfehlung

1. **Ton:** `PlaySoundW(BEEP.as_ptr() as *const u16, null_mut(), SND_MEMORY | SND_ASYNC | SND_NODEFAULT)` mit `static BEEP: &[u8] = include_bytes!(…)`, Feature `Win32_Media_Audio` an `windows-sys` 0.61. Kein `SND_SYSTEM`, Rückgabewert ignorieren. Kein `rodio`.
2. **„Nicht stören“:** Direkt vor dem Ton einmal `SHQueryUserNotificationState` (Feature `Win32_UI_Shell`) abfragen. Bei `QUNS_BUSY`, `QUNS_RUNNING_D3D_FULL_SCREEN` und `QUNS_PRESENTATION_MODE` kein Ton, bei allen anderen Werten und bei einem Fehler-HRESULT spielen. Das eigentliche *Bitte nicht stören* von Windows 11 bleibt unerkannt und wird nach der Regel aus *Hinweis bei Phasenwechsel* ignoriert. Das gehört so in die SPEC, damit es nicht als Fehler gilt.
3. **Optional, nicht für v1:** `FocusSessionManager::IsFocusActive` über das `windows`-Crate. Dokumentiert und billig, deckt aber nur Fokussitzungen ab und braucht neben `windows-sys` ein zweites Windows-Crate.

## Nicht verifiziert

- Ob ein randloses Vollbildfenster (Browser-Video, PowerPoint-Bildschirmpräsentation, randloses Spiel) `QUNS_BUSY` liefert. Der Test lief ins Leere, weil die Sitzung gesperrt war.
- Ob *Bildschirm duplizieren* einen der `QUNS_*`-Werte auslöst.
- Ob der Eintrag der App im Lautstärkemixer nach dem ersten Ton erscheint und den Ton regelt. Das folgt aus der Doku zu `SND_SYSTEM`, wurde aber nicht angesehen.
- Verhalten von `PlaySoundW`, wenn das Standardgerät während des Abspielens wechselt.
