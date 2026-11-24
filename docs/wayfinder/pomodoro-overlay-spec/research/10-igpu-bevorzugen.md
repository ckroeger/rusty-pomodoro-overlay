# Recherche: iGPU für das Overlay bevorzugen

Ticket: [10 – iGPU für das Overlay bevorzugen](../tickets/10-igpu-bevorzugen.md) · Stand: 2026-11-24

## Frage

Wie erreicht eine Rust-App mit `eframe` 0.37 + `glow` (OpenGL über WGL) unter Windows 11 auf Hybrid-GPU-Laptops (NVIDIA Optimus, auch AMD), dass sie auf der stromsparenden iGPU läuft statt auf der dGPU? Bewertet nach Zuverlässigkeit bei OpenGL, Eingriff ins System und Aufwand in Rust.

Gegeben (im Spike empirisch bestätigt, hier nicht erneut gemessen): Ohne Eingriff lädt die App auf dem Testgerät (RTX A2000 Laptop + Intel UHD) `nvoglv64.dll` (~168 MB Working Set). Mit `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, Wertname = voller exe-Pfad, Wert `GpuPreference=1;` lädt sie `igxelpicd64.dll` (~94 MB), und die Transparenz funktioniert weiter.

## Kurzantwort

**Empfehlung: Die App greift nicht selbst ein. Sie exportiert keine Optimus/PowerXpress-Symbole, schreibt nicht ungefragt in die Registry, und die Doku erklärt, wie man in den Windows-Grafikeinstellungen „Energiesparmodus“ für die exe wählt.** Damit man die Wahl prüfen kann, schreibt die App den `GL_RENDERER` ins Log bzw. zeigt ihn in einer Info-Ansicht an.
Als **optionale Bequemlichkeit** (nicht für v1 nötig) kann sie einen ausdrücklichen Befehl wie `--prefer-igpu` anbieten. Der setzt den Wert in `UserGpuPreferences` für `std::env::current_exe()`, und zwar nur, wenn dort noch nichts steht. Er wirkt erst beim nächsten Start.
- `NvOptimusEnablement` / `AmdPowerXpressRequestHighPerformance` helfen nicht: Sie können nur die dGPU **anfordern**. Bei NVIDIA bedeutet 0 laut Dokumentation ausdrücklich „ignorieren“, nicht „iGPU“.
- glutin/winit/eframe bieten für WGL **keine** Adapter-Wahl.
- Ein Software-Renderer ist kein sinnvoller Weg: Microsofts GDI-Renderer kann nur OpenGL 1.1, und egui_glow verlangt mindestens 2.0. Mesa/llvmpipe ginge nur über eine fremde DLL.

## 1. `NvOptimusEnablement` / `AmdPowerXpressRequestHighPerformance`

### NVIDIA

Primärquelle: NVIDIA, *Enabling High Performance Graphics Rendering on Optimus Systems*, TB-05942-003v02, Juli 2012 ([PDF](https://developer.download.nvidia.cn/devzone/devcenter/gamegraphics/files/OptimusRenderingPolicies.pdf)).

- Wertsemantik, wörtlich: „A value of 0x00000001 indicates that rendering should be performed using High Performance Graphics. **A value of 0x00000000 indicates that this method should be ignored.**“ **0 steuert also nicht aktiv zur iGPU.** Es ist nur so, als gäbe es das Symbol nicht.
- Vorrang (höchster zuerst): 1. Forced Rendering Mode (OEM), 2. Rechtsklick-Kontextmenü, 3. Anwendungsprofil, 4. statische Bindung an bestimmte Bibliotheken (`nvapi.dll`, `nvapi64.dll`, `opencl.dll`, `nvcuda.dll`, `cudart*` …), 5. `NvOptimusEnablement`, 6. globales Profil.
- Default ohne Profil: Steht das globale Profil auf „Auto-select“, „any other application will render using Integrated Graphics“. Steht es auf „High Performance“, landen Apps ohne Profil auf der dGPU. **Folgerung (nicht verifiziert):** Dass die App auf dem Testgerät standardmäßig auf NVIDIA läuft, liegt wahrscheinlich an der globalen Einstellung dieses Geräts, die bei Workstation-Laptops (RTX A-Serie) oft „High Performance“ ist, oder an einer OEM-Vorgabe. Auf anderen Geräten kann die App ohne jeden Eingriff schon auf der iGPU laufen.
- „The rendering device for a given application is determined when the application starts.“ Eine Änderung wirkt also erst beim nächsten Start.
- Gilt laut Dokument für DirectX und OpenGL. Die Abfrage `glGetString(GL_VENDOR/GL_RENDERER)` ist ausdrücklich als Weg genannt, die verwendete GPU zu erkennen.

Neuere Lage: Die NVIDIA-Systemsteuerungshilfe ([Setting the Preferred Graphics Processor](https://www.nvidia.com/content/Control-Panel-Help/vLatest/en-us/mergedProjects/nv3d/Setting_the_Preferred_Graphics_Processor.htm)) sagt: „Beginning with Windows 10 20H1, the Windows OS assigns the graphics processor to use for various applications, and overrides any settings made in the NVIDIA Control Panel … for that application.“ Und: „If the OS does not have a GPU preference for an application, then you can specify the preferred graphics processor … from the NVIDIA Control Panel.“

### AMD

Primärquelle: GPUOpen, Ken Mitchell, *Selecting the Best Graphics Device to Run a 3D Intensive Application*, 2016-11-16 ([Link](https://gpuopen.com/learn/amdpowerxpressrequesthighperformance/)).

- Beispiel: `extern "C" { _declspec(dllexport) DWORD AmdPowerXpressRequestHighPerformance = 0x00000001; }`. Der Treiber sucht das Symbol in der Export-Tabelle der **exe**: „Adding the above piece of code in a dynamically linked library will not work.“
- Gilt für DirectX und OpenGL, „these instructions do not apply to Vulkan™ or OpenCL™ applications“.
- Was der Wert 0 bedeutet, steht dort **nicht**. Das Symbol fordert jedenfalls nur hohe Leistung an. Einen dokumentierten Wert für „iGPU erzwingen“ gibt es nicht.

### Export aus einer Rust-exe

- rustc exportiert bei einer `bin` standardmäßig keine Symbole. `#[no_mangle] pub static` allein reicht also nicht. Der stabile Weg ist ein Linker-Argument (MSVC `/EXPORT:NvOptimusEnablement`, z. B. per `build.rs` mit `cargo:rustc-link-arg-bins=/EXPORT:NvOptimusEnablement`) oder eine `.drectve`-Section, wie sie das Crate `surfman` per Makro anlegt ([surfman 0.3.0 macros.rs](https://docs.rs/crate/surfman/0.3.0/source/src/macros.rs)). Das Nightly-Flag `-Z export-executable-symbols` ([Unstable Book](https://doc.rust-lang.org/unstable-book/compiler-flags/export-executable-symbols.html), [RFC 2841](https://rust-lang.github.io/rfcs/2841-export-executable-symbols.html)) ist instabil. Kontrolle mit `dumpbin /exports`.
- **Für dieses Ticket ist das egal**, weil die Symbole nur in die falsche Richtung wirken. Relevant ist aber die Umkehrung: **Kein Crate im Abhängigkeitsbaum darf diese Symbole exportieren oder `nvapi`/`nvcuda`/`opencl` statisch einbinden**, sonst landet die App auf der dGPU. Im Registry-Quellbaum des Spikes (eframe 0.37, egui_glow 0.37, glutin 0.32.3, glutin-winit 0.5.0, winit 0.30.13 usw.) kommt keiner dieser Namen vor (grep nach `NvOptimus`/`PowerXpress`, ohne Treffer). Neue Abhängigkeiten (Tray, Hotkeys, Ton) sollte man später kurz daraufhin prüfen.

## 2. Eigener Eintrag in `UserGpuPreferences`

### Was dokumentiert ist

- Die Windows-Grafikeinstellungen pro App gibt es seit Windows 10 1803 (Insider-Build 17093). Im Microsoft-Blog ([Build 17093](https://blogs.windows.com/windows-insider/2018/02/07/announcing-windows-10-insider-preview-build-17093-pc/)) steht: „When you set an application preference in the Windows Graphics settings, that will take precedence over the other control panel settings.“ Und: „**Applications are always allowed to have the ultimate choice of which GPU to use**, so you may see additional applications that do not follow the preferences you set.“
- Das DirectX-Blog zur neuen Einstellungsseite ([Navigating the redesigned Graphics settings page](https://devblogs.microsoft.com/directx/navigating-the-redesigned-graphics-settings-page), Jan. 2023): Die Wahl pro App „will override the system-wide setting preference“. Für welche APIs das gilt, sagt die Seite nicht.
- Für Treiber gibt es seit Windows 10 1809 die DDI-Aufzählung [`D3DKMT_GPU_PREFERENCE_QUERY_STATE`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/d3dkmthk/ne-d3dkmthk-_d3dkmt_gpu_preference_query_state) (`HIGH_PERFORMANCE`, `MINIMUM_POWER`, `UNSPECIFIED` …). Damit fragt ein User-Mode-Treiber, also auch eine OpenGL-ICD, die Vorgabe des Systems ab. Die Zahlenwerte von `GpuPreference=` passen zu [`DXGI_GPU_PREFERENCE`](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_6/ne-dxgi1_6-dxgi_gpu_preference) (0 = unspecified, 1 = minimum power, 2 = high performance).
- **Nicht dokumentiert:** Der Registry-Pfad und das Format des Werts (`GpuPreference=1;`). Microsoft beschreibt sie nirgends als Schnittstelle für Apps. In den Primärquellen kommt die Registry nicht vor. Sie ist der Speicherort der Einstellungs-App, den Anleitungen von Dritten und Microsoft-Q&A-Beiträge übereinstimmend nennen. Microsoft könnte ihn ändern, ohne einen Vertrag zu brechen.

### Wirkt das bei OpenGL garantiert?

- **Nicht offiziell garantiert.** Microsoft beschreibt die GPU-Präferenz nirgends ausdrücklich für OpenGL. Bei OpenGL wählt nicht Windows den Adapter, sondern die Treiber-ICD (bei Optimus der NVIDIA-Treiber, der zur Intel-ICD weiterreicht). Dass sie sich an die Präferenz hält, hängt also vom Treiber ab.
- Die NVIDIA-Aussage „beginning with Windows 10 20H1 the OS assigns the graphics processor … and overrides … NVIDIA Control Panel“ ist die nächstbeste Primärquelle dafür, dass aktuelle NVIDIA-Treiber die Windows-Präferenz auch für OpenGL beachten. Der Spike bestätigt das für das Testgerät. Zu Treibern vor 20H1 und zu AMD-Hybrid-Systemen gibt es keine Primärquelle (nicht verifiziert). Im Vogons-Forum berichtet ein Nutzer, dass „Energiesparen“ bei alten OpenGL-Spielen nicht wirkte (anekdotisch, ältere Treiber).
- Ein OEM-„Forced Rendering Mode“ (z. B. dGPU-only/MUX im BIOS) hat Vorrang, dann gibt es überhaupt keine Wahl.

### Darf/sollte die App sich selbst eintragen?

- **Technisch ja:** Es ist HKCU, braucht keine Admin-Rechte, und es ist derselbe Wert, den die Einstellungs-App schreibt. Er erscheint dort auch und lässt sich dort wieder ändern oder entfernen.
- **Konvention:** Es gibt keine dokumentierte App-API. Der Wert ist eine *Benutzer*-Präferenz („User“GpuPreferences), und Microsoft betont, dass die *App* ihre eigene Wahl auf anderem Weg trifft (DXGI `EnumAdapterByGpuPreference`, das es für OpenGL nicht gibt). Wenn eine App heimlich die Benutzereinstellung überschreibt, ist das schlechter Stil. Deshalb, wenn überhaupt: nur auf ausdrücklichen Befehl, und nie einen vorhandenen Wert überschreiben (der Wert kann auch andere Schlüssel wie `AutoHDREnable=…;` enthalten).
- **Nebenwirkungen:**
  - Es wirkt erst beim nächsten Start (NVIDIA: GPU wird „when the application starts“ bestimmt). Ein Eintrag beim ersten Start hilft also erst beim zweiten.
  - **Pfadbindung:** Der Wertname ist der volle exe-Pfad. `cargo install` legt die exe unter `%USERPROFILE%\.cargo\bin\<name>.exe` ab (bzw. `$CARGO_HOME\bin`), `cargo run` unter `target\release\…`. Das sind zwei verschiedene Einträge. Bei einem anderen `CARGO_HOME` oder `--root` gilt der Eintrag nicht mehr. `std::env::current_exe()` liefert den richtigen Pfad zur Laufzeit.
  - **Reste:** `cargo uninstall` kennt keinen Hook. Der Eintrag bleibt zurück, ist aber harmlos.
  - Die dGPU kann dann nicht mehr ungewollt anspringen, was für ein Overlay gewollt ist. Die Transparenz hat der Spike auf beiden GPUs bestätigt.

## 3. Adapter-Wahl in glutin / egui_glow / winit

Geprüft im Quellcode (Cargo-Registry, Versionen aus `prototypes/02-overlay-spike/Cargo.lock` auf Branch `prototype/02-overlay-spike`):

- **glutin 0.32.3, WGL** (`src/api/wgl/display.rs`, `config.rs`): `Display::new` lädt per `LoadLibraryW` die `opengl32.dll` bzw. die DLL aus der Umgebungsvariable **`GLUTIN_WGL_OPENGL_DLL`**. Die Config-Vorlage kennt nur `hardware_accelerated` (→ `PFD_GENERIC_ACCELERATED`/`PFD_GENERIC_FORMAT` bzw. `WGL_ACCELERATION_ARB` = `FULL_`/`NO_ACCELERATION_ARB`). Es gibt **keinen Code für Adapter oder GPU-Affinität** (weder `WGL_NV_gpu_affinity` noch `WGL_AMD_gpu_association`, Suche nach `gpu`/`adapter`/`affinity` ohne Treffer). Beide Erweiterungen gäbe es ohnehin nur auf Quadro/FirePro, und keine kann bei Optimus auf die Intel-GPU wechseln.
- **glutin-winit 0.5.0**: keine Adapter-Optionen (Suche ohne Treffer).
- **eframe 0.37 / egui_glow 0.37** (`eframe/src/native/glow_integration.rs`): `NativeOptions::glow_options.hardware_acceleration` (`Required`/`Preferred`/`Off`) wird nur auf `prefer_hardware_accelerated` abgebildet. Die API-Wahl ist fest auf `ApiPreference::FallbackEgl` gesetzt (Kommentar: „we might want to expose this option to users in the future“).
- **winit 0.30**: erzeugt nur das Fenster und hat mit dem GL-Adapter nichts zu tun.

**Ergebnis:** Für WGL gibt es keine Adapter-Wahl. Welche ICD geladen wird, entscheiden Windows und der Treiber. Nur wgpu (DXGI/Vulkan) bietet `PowerPreference::LowPower`, aber wgpu wurde in Ticket 01 wegen der Transparenz unter DX12 und des größeren RAM-Bedarfs verworfen.

## 4. Weitere Optionen

- **Software-Renderer über `HardwareAcceleration::Off`:** Unter Windows landet man so bei Microsofts „GDI Generic“-Renderer mit OpenGL 1.1. egui_glow 0.37 lehnt das ab (`painter.rs`: „egui_glow requires opengl 2.0+“, mit Kommentar zum „opengl 1.1 microsoft sw rendering context“). **Geht also nicht.**
- **Mesa llvmpipe** als eigene `opengl32.dll` über `GLUTIN_WGL_OPENGL_DLL`: technisch möglich (glutin liest die Variable vor dem Laden), aber man müsste eine fremde DLL von ~20–30 MB mitliefern bzw. installieren lassen. Das passt nicht zu „nur `cargo install`“, braucht CPU-Zeit fürs Rendern, und ob Alpha pro Pixel funktioniert, ist ungeprüft. **Nicht empfohlen**, höchstens als Notnagel bei Treiberproblemen.
- **EGL/ANGLE** (`ApiPreference` ist in eframe fest auf `FallbackEgl`): Man bräuchte eine mitgelieferte `libEGL.dll`/`libGLESv2.dll`, und ANGLE wählt den D3D11-Adapter selbst. Das ist nicht über eframe steuerbar und wurde nicht weiter geprüft.
- **NVIDIA-Anwendungsprofil** (NVAPI/DRS): Damit ließe sich per Programm ein Profil „Integrated“ anlegen. Dafür müsste man aber `nvapi64` einbinden, was laut NVIDIA-Dokument selbst eine Bindung ist, die zur *dGPU* führt. Außerdem gilt es nur für NVIDIA und ist ab 20H1 der Windows-Präferenz untergeordnet. **Nicht empfohlen.**
- **Doku-Hinweis an den Nutzer:** Einstellungen → System → Anzeige → Grafik → App hinzufügen (die exe unter `%USERPROFILE%\.cargo\bin`) → Optionen → „Energiesparmodus“. Das ist der von Microsoft vorgesehene, unterstützte Weg, ohne Eingriff durch die App. Er funktioniert auch, wenn sich die Registry-Struktur ändert.
- **Diagnose:** `GL_RENDERER` per `cc.gl` (`eframe::CreationContext::gl`, `glow::Context::get_parameter_string(glow::RENDERER)`) einmal beim Start loggen. egui_glow macht das schon auf `log::debug`-Ebene. So sieht der Nutzer, ob die Einstellung wirkt.

## Bewertung

| Option | Zuverlässigkeit bei OpenGL | Eingriff ins System | Aufwand in Rust |
|---|---|---|---|
| `NvOptimusEnablement`/`AmdPowerXpress…` = 0 | **wirkungslos** (0 = ignorieren; nur dGPU anforderbar) | keiner | gering (Linker-Flag) |
| Doku-Hinweis (Windows-Grafikeinstellungen) | hoch auf aktuellen Treibern (Spike bestätigt; ab 20H1 laut NVIDIA vorrangig) | durch den Nutzer, unterstützt | keiner |
| App schreibt `UserGpuPreferences` (opt-in) | wie oben, aber Format undokumentiert; wirkt erst beim nächsten Start | HKCU, Benutzereinstellung | gering (`windows-sys` Registry, ~30 Zeilen) |
| Adapter-Wahl glutin/eframe | gibt es nicht | – | – |
| Software (GDI) | geht nicht (GL 1.1) | – | – |
| Mesa llvmpipe | funktioniert vermutlich, Alpha ungeprüft | fremde DLL | mittel, Verteilung problematisch |

## Nicht verifiziert

- Ob ältere Treiber (vor Windows 10 20H1) und AMD-Hybrid-Systeme die Windows-Präferenz bei OpenGL beachten. Belegt ist es nur für das Testgerät (Spike) und indirekt durch die NVIDIA-Hilfe.
- Warum das Testgerät ohne Eingriff NVIDIA nimmt (vermutlich die globale NVIDIA-Einstellung „Hochleistung“ oder eine OEM-Vorgabe; nicht nachgesehen, keine Systemeinstellungen verändert).
- Die Bedeutung von `AmdPowerXpressRequestHighPerformance = 0` (AMD dokumentiert nur 1).
- Das genaue Format von `UserGpuPreferences` ist nur durch die Einstellungs-App, Dritt-Anleitungen und den Spike belegt, nicht durch Microsoft-Dokumentation.
- Mesa llvmpipe mit Alpha pro Pixel unter Windows und dessen RAM-Bedarf.
