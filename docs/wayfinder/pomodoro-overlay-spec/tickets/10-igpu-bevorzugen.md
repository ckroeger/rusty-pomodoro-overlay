---
title: iGPU für das Overlay bevorzugen
labels: [wayfinder:research]
status: closed
assignee: ckr
blocked_by: [02]
---

# iGPU für das Overlay bevorzugen

## Question

Wie kann eine Rust-App mit `eframe` 0.37 + `glow` (OpenGL) unter Windows 11 auf Hybrid-GPU-Laptops (NVIDIA Optimus, auch AMD) erreichen, dass sie auf der stromsparenden iGPU läuft statt auf der dGPU? Im Spike (siehe *Overlay-Spike mit der gewählten Technik*) sank der RAM so von ~168 MB auf ~94 MB Working Set. Kandidaten: exportierte Symbole `NvOptimusEnablement` / `AmdPowerXpressRequestHighPerformance` (= 0) und ob sie bei OpenGL überhaupt wirken, ein eigener Registry-Eintrag unter `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` (`GpuPreference=1;`), das Angebot von glutin/winit zur Adapter-Wahl, ein Software-Renderer als Fallback. Bewertet nach: Zuverlässigkeit bei OpenGL, Eingriff ins System, Aufwand in Rust. Ergebnis: Empfehlung, ob und wie die App das tun soll, oder ob ein Hinweis in der Doku genügt.

## Resolution

**Empfehlung: Die App greift nicht selbst ein, ein Doku-Hinweis genügt.** Die Doku erklärt den unterstützten Weg: Einstellungen → System → Anzeige → Grafik → exe (`%USERPROFILE%\.cargo\bin\<name>.exe`) → „Energiesparmodus“. Die App schreibt den `GL_RENDERER` beim Start ins Log bzw. zeigt ihn in einer Info-Ansicht an, damit man die Wahl prüfen kann.
- **Optional, nicht für v1 nötig:** ein ausdrücklicher Befehl `--prefer-igpu`. Er schreibt `GpuPreference=1;` nach `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` für `current_exe()`, nur wenn dort noch kein Wert steht, und wirkt erst beim nächsten Start.
- **Kein Weg über die Optimus-Symbole:** `NvOptimusEnablement`/`AmdPowerXpressRequestHighPerformance` können nur die dGPU *anfordern*. Bei NVIDIA bedeutet 0 laut Dokument „this method should be ignored“, nicht iGPU. Umgekehrt darf keine Abhängigkeit diese Symbole exportieren oder `nvapi`/`nvcuda`/`opencl` statisch einbinden. Im aktuellen Baum tut das keine.
- **Keine Adapter-Wahl im Stack:** glutin 0.32 (WGL), glutin-winit, egui_glow und eframe 0.37 bieten keine; es gibt nur `hardware_acceleration` und die Umgebungsvariable `GLUTIN_WGL_OPENGL_DLL`. Einen Software-Fallback gibt es praktisch nicht: GDI kann nur GL 1.1, egui_glow verlangt 2.0+. Mesa wäre eine fremde DLL.

**Risiken:** `UserGpuPreferences` ist der Speicherort der Einstellungs-App, aber keine dokumentierte App-Schnittstelle. Dass die Präferenz bei OpenGL wirkt, hängt vom Treiber ab. Belegt ist es durch den Spike und durch die NVIDIA-Hilfe („ab Windows 10 20H1 weist das OS die GPU zu“), für ältere Treiber und AMD-Hybrid nicht. Ein OEM-Zwangsmodus (z. B. dGPU-only im BIOS) hat Vorrang. Der Eintrag hängt am exe-Pfad, `cargo run` und `cargo install` brauchen also getrennte Einträge.

Details: [Recherche](../research/10-igpu-bevorzugen.md)
