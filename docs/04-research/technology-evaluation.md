> **Status:** Rechercheergebnis (delegierter Analyseagent, 2026-10-23), von Claude integriert, **nicht** vom Owner geprüft. Kennzeichnung im Text: [R] = RESEARCHED, [A] = ASSUMED. Abgeleitete Empfehlungen sind PROPOSED. Entscheidungen erfolgen ausschließlich per ADR (`../05-decisions/`).
> **Maßgebliche Quelle für:** Technologie-, Engine-, UI- und Renderingbewertung (Mandat 5, Punkte 1–3, 7, 8, 11, 13, 14)

# HASE VRM MAKER – Technologiebewertung Engine / UI / Rendering (Stand 2026-10-23)

Kennzeichnung: **[R]** = RESEARCHED (Quelle + Datum/Version), **[A]** = ASSUMED (kein Beleg in dieser Recherche). Die GitHub-Release-Seiten zeigen oft kein Jahr; "2026" ist dort aus Asset-Zeitstempeln oder Footer abgeleitet, sofern vermerkt. Einige Detailseiten (Lizenztexte, Qt-Versionsstand, Godot-/Bevy-Lizenz) konnten nicht direkt verifiziert werden; das ist jeweils markiert.

## 1. Versions- und Lizenzstand (Belege)

| Komponente | Version / Datum | Lizenz | Quelle |
|---|---|---|---|
| wgpu | v30.0.1, 22. Aug (Jahr nicht angezeigt) [R] | Apache-2.0 + MIT [R]; MSRV 1.87 (Nutzung), Breaking Releases ca. alle 3 Monate [R] | https://github.com/gfx-rs/wgpu , /releases |
| Bevy | v0.20.0, 08. Okt (Jahr nicht angezeigt) [R]; noch 0.x, Breaking Changes je Release [A: aus 0.x-Versionierung] | Lizenz nicht verifiziert (MIT/Apache-2.0 [A]) | https://github.com/bevyengine/bevy/releases |
| Godot | 4.7.2-stable, 18.08.2026 [R] | Lizenz nicht auf der abgerufenen Seite; MIT [A] | https://github.com/godotengine/godot/releases |
| egui | 0.37.0, 15. Okt (2026 laut Seite) [R] | MIT OR Apache-2.0 [R] | https://github.com/emilk/egui/releases , https://github.com/emilk/egui |
| egui_dock | 0.21.0 für egui 0.36; kein 0.37-Beleg gefunden [R, eingeschränkt] | nicht geprüft | https://docs.rs/crate/egui_dock/0.21.0 |
| Slint | 1.18.1, 21.09.2026 [R] | Wahlweise Royalty-free (proprietäre Desktop/Mobile/Web-Apps), GPLv3 oder kommerziell [R] | https://github.com/slint-ui/slint |
| iced | 0.14.0, 7. Dez (vermutlich 2025; Jahr nicht angezeigt) [R] | nicht geprüft (MIT [A]) | https://github.com/iced-rs/iced/releases |
| Dear ImGui | v1.93.0, 14. Okt [R]; Docking-Branch nicht verifiziert | MIT [A] | https://github.com/ocornut/imgui/releases |
| Tauri | v2.12.5, 21. Okt (2026 laut Footer) [R] | MIT/Apache [A] | https://github.com/tauri-apps/tauri/releases |
| three-vrm | v3.5.5, 9. Juli; 3.6.0-beta.0 vom 29. Sept [R] | MIT [R] | https://github.com/pixiv/three-vrm |
| UniVRM | Releases vorhanden, Versionsnummer nicht abgerufen; ab v0.128.0 Unity 2022.3 LTS+ [R] | MIT [R] | https://github.com/vrm-c/UniVRM |
| bgfx | Release nicht ermittelt; Copyright "2010-2026", 9.589 Commits [R] | BSD-2-Clause [R] | https://github.com/bkaradzic/bgfx |
| Diligent Engine | Release nicht ermittelt (Releases-Bereich leer) [R] | Apache-2.0 [R] | https://github.com/DiligentGraphics/DiligentEngine |
| Filament | v1.78.1, 21.10.2026 [R] | Lizenz nicht geprüft (Apache-2.0 [A]) | https://github.com/google/filament/releases |
| wasmtime | v50.0.0, 20.10.2026 [R] | Apache-2.0 [R] | https://github.com/bytecodealliance/wasmtime |
| bevy_vrm | 0.4.0 (2026-07-28), Bevy ^0.19 [R] – hinkt Bevy 0.20 hinterher | Apache-2.0 + MIT [R] | https://docs.rs/crate/bevy_vrm/latest , https://github.com/unavi-xyz/bevy_vrm |
| bevy_vrm1 | 0.7.1, Bevy ^0.18, VRM 1.0 + VRMA, "early stage" [R] | nicht geprüft | https://docs.rs/bevy_vrm1 |
| godot-vrm (V-Sekai) | Asset-Library 2.1.0, 2024-11-10, MIT [R] | MIT [R] | https://godotengine.org/asset-library/asset/2031 , https://github.com/V-Sekai/godot-vrm |

## 2. Architekturoptionen

### 2.1 Native Rust (wgpu + egui/Slint)
- wgpu: Vulkan und DX12 sind unter Windows "first-class"; GL nur best-effort [R, https://github.com/gfx-rs/wgpu]. Lizenz permissiv, gut für kommerziellen Vertrieb [R].
- VRM-Machbarkeitsbeleg: serde_vrm (Serde-Typen VRM 0.0/1.0) und bevy_shader_mtoon existieren als Crates [R, docs.rs/bevy_vrm]. bevy_vrm lädt VRM 0/1 inkl. Humanoid, MToon, First Person, SpringBone; **nicht** unterstützt: Expressions, LookAt, Node Constraints, Spring Colliders [R, GitHub-Seite]. **Kein Export erwähnt** [R, Seite nennt nur Laden]. Ein VRM-1.0-Exporter müsste vermutlich selbst gebaut werden (glTF-Writer + VRMC_*-Extensions) [A].
- Risiko: Alle Rust-Bausteine sind 0.x oder haben häufige Breaking Changes (wgpu ca. quartalsweise [R], egui 0.37 [R], Bevy 0.20 [R]). Ökosystem-Crates (egui_dock, bevy_vrm) laufen den Kernversionen hinterher [R, siehe Tabelle].
- Eigenes Morph-Target-/Skinning-Rendering auf wgpu ist machbar (Standard-GPU-Techniken) [A]; Sculpting/Weight Painting später erfordert Eigenbau von Picking, Brush, Undo [A].

### 2.2 C++ (Qt oder eigene Engine)
- Qt: LGPL-Pflichten laut Qt-FAQ: Anwender muss die App gegen eine geänderte Qt-Version neu linken können; Lizenztext + deutlicher Hinweis; bei Nichteinhaltung "darf nicht verteilt werden"; einige Module sind GPL-only und erzwingen GPL der App; Qt gibt keine Rechtsberatung und nennt die kommerzielle Lizenz bei Zweifel als oft beste Wahl [R, https://www.qt.io/faq/3.7.-what-are-my-obligations-when-using-qt-under-the-lgpl , https://www.qt.io/development/open-source-lgpl-obligations]. Statisches Linken ist damit praktisch nur mit zusätzlichem Aufwand (Objektdateien zum Relinken) konform [A, Ableitung aus Relink-Pflicht]. Aktueller Qt-Stand: Hinweis auf "Qt 6.12 LTS" nur aus Forenbanner, **nicht offiziell verifiziert**; Commercial-LTS 6.5.12 (06.03.2026) und 6.8.7 (27.03.2026) belegt [R, lists.qt-project.org/pipermail/announce/2026-March/].
- Eigene C++-Engine auf bgfx (BSD-2, D3D11/12/Vulkan/Metal/GL, WebGPU nur via Dawn) [R] oder Diligent (Apache-2.0, D3D11/12/Vulkan/GL/Metal/WebGPU) [R]: gute Lizenz, aber höchster Eigenaufwand für Editor-Framework (Docking, Undo, Properties).
- C++-VRM-Bibliotheken: **in dieser Recherche nicht verifiziert** [keine Quelle].

### 2.3 Engine-basiert
- **Godot 4**: Aktiv (4.7.2, 18.08.2026) [R]. godot-vrm bietet VRM-1.0-Import **und -Export** (.vrm), MToon-Shader; gltf-Export unterstützt VRMC_node_constraint und VRMC_materials_mtoon, **nicht** VRMC_springBone; Hinweis auf mögliche Materialkorruption beim gltf-Export [R, https://github.com/V-Sekai/godot-vrm]. Letzter Asset-Library-Eintrag 2.1.0 vom 2024-11-10 [R] – Aktualität der Hauptlinie gegenüber Godot 4.7 unbelegt. Control-UI mit Docking ist eingebaut (Godot-Editor selbst ist Godot-App) [A]. Eigenständiges "Studio" = Godot-Export-Template mit Spiel-UI; Distribution als Windows-Exe ist üblich [A]. Plugin-Lauf in Godot (GDScript) hat keine Sandbox [A].
- **Unity**: Runtime Fee am 12.09.2024 abgeschafft; Personal-Grenze 200.000 USD Umsatz/Funding, Pro ab da (2.200 USD/Sitz/Jahr), Enterprise ab 25 Mio USD, seit 01.01.2025 [R, https://unity.com/blog/unity-is-canceling-the-runtime-fee]. Aktuelle 2026-Konditionen **nicht verifiziert**. UniVRM (MIT) bietet VRM-1.0-Import/Export zur Laufzeit und im Editor [R, https://github.com/vrm-c/UniVRM] – stärkster Export-Beleg. Nachteil: Closed-Source-Engine, Vertragsänderungsrisiko (Historie belegt), Runtime-Distribution eines Tools ist ungewohnt [A].
- **Unreal**: 5 % Royalty ab 1 Mio USD Lifetime-Brutto bei Produkten mit Engine-Code zur Laufzeit; Firmen >1 Mio USD Jahresumsatz in Nicht-Spiele-Bereichen brauchen Seats (1.850 USD/Nutzer/Jahr) [R, https://www.unrealengine.com/en-US/faq ; EULA-Text nicht geprüft]. VRM-Plugin: **nicht recherchiert**. Für ein schlankes Charakter-Tool überdimensioniert [A].
- **Bevy**: ECS-Engine, 0.20.0 [R]. Feathers-Widgets sind für Bevy-Tooling gedacht; Bevy Editor ist laut Quellen nicht als veröffentlicht belegt [R, https://bevy.org/news/bevys-sixth-birthday/]. Für komplexe Docking-Property-Editoren unreif [A, abgeleitet].

### 2.4 Web-Hybrid (Tauri/Electron + three.js + three-vrm)
- three-vrm 3.5.5 (MIT), MToon (auch WebGPU via MToonNodeMaterial, Three.js r167+; NodeMaterial-System laut Doku "under development", Brüche möglich) [R, https://github.com/pixiv/three-vrm]. **Exporter**: auf der Repo-Seite nicht erwähnt [R]; ein Exporter-Paket existiert möglicherweise (nicht verifiziert).
- Tauri 2.12.5 [R] nutzt System-WebView (unter Windows WebView2 [A]); WebGPU-Verfügbarkeit in WebView2 nicht verifiziert. Electron: nicht recherchiert; bündelt Chromium (größere Binärgröße) [A].
- Stärken: schnelle UI-Entwicklung, reife Docking-/Property-Bibliotheken im Web-Ökosystem [A], Barrierefreiheit über DOM/ARIA [A]. Schwächen: Performance bei Sculpting/Weight Painting (JS/WASM), GPU-Speicher, WebView-Unterschiede [A].

### 2.5 Hybrid (Rust/C++-Core + UI anderswo)
- Rust-Core (wgpu, Mesh/Rig/VRM-I/O) + UI in Slint/Tauri-WebView: sauberes Package-/Plugin-Modell möglich (Core als Bibliothek, UI austauschbar) [A]. Integrationsaufwand: Viewport-Einbettung (native GPU-Surface in WebView ist heikel [A]; Slint/egui binden wgpu-Texturen direkt ein [A: für egui belegt "egui-wgpu" Integration, R]).

## 3. Rendering/GPU-Abstraktion

| Option | Windows-Backends | Lizenz | MToon/Toon-Machbarkeit |
|---|---|---|---|
| wgpu | DX12, Vulkan (first-class) [R] | MIT/Apache [R] | WGSL-Eigenbau; Referenz: bevy_shader_mtoon [R, Existenz] |
| Vulkan direkt | Vulkan | n/a | maximal flexibel, höchster Aufwand [A] |
| bgfx | D3D11/12, Vulkan, GL [R] | BSD-2 [R] | eigener Shader (bgfx-Shaderformat) [A] |
| Diligent | D3D11/12, Vulkan, GL, WebGPU [R] | Apache-2.0 [R] | Eigenbau [A] |
| Filament | Release 1.78.1 [R]; Backends nicht geprüft | Apache-2.0 [A] | PBR-orientiert; Toon erfordert Custom-Material [A] |
| Bevy-Renderer | über wgpu [A] | siehe Bevy | bevy_shader_mtoon + bevy_vrm vorhanden [R] |

MToon an sich ist ein ausgedehnter Shader (Rim, Matcap, Outline), in Bevy/three-vrm/Godot/Unity bereits umgesetzt [R: bevy_vrm unterstützt MToon; three-vrm MToonNodeMaterial; godot-vrm "MToon Shader"; UniVRM] – Machbarkeit in allen Stacks belegt, nicht in reinem wgpu ohne Bevy.

## 4. UI-Frameworks mit Viewport

| Framework | Stand | Docking/Property-Editor | Barrierefreiheit |
|---|---|---|---|
| egui | 0.37.0 [R] | egui_dock als Community-Crate, hinkt hinterher (0.21 für egui 0.36) [R]; Immediate Mode, gut für Property-Panels [A] | AccessKit immer aktiv seit 0.34; 0.37 mit AccessKit-Verbesserungen (Rollen, SetValue für TextEdit) [R, https://github.com/emilk/egui/releases] |
| Slint | 1.18.1 [R] | Docking nicht belegt; Barrierefreiheit nicht belegt (Seite schweigt) [R: keine Aussage] | nicht verifiziert |
| iced | 0.14.0 (7. Dez) [R] | nicht verifiziert | nicht verifiziert [A: vermutlich begrenzt] |
| Dear ImGui | v1.93.0 [R] | Docking-Branch [A: existiert, nicht verifiziert] | keine native A11y [A] |
| Qt | siehe Lizenz | reifste Docking-/Model-View-Edit-Infrastruktur [A] | Qt Accessibility vorhanden [A] |
| Tauri-WebView | 2.12.5 [R] | Web-Docking-Bibliotheken [A] | DOM/ARIA [A] |
| Godot Control-UI | 4.7.2 [R] | integriert [A] | nach meiner Kenntnis begrenzt/jüngere Entwicklung [A] |

## 5. Lizenz und Distribution (Fallstricke)
- Qt: LGPL-Relink-Pflichten, GPL-only-Module, Nichteinhaltung = Vertriebsverbot [R]. Kommerzielle Lizenz kostenpflichtig (Preise nicht recherchiert).
- Unity: Runtime-Fee-Historie (2023 angekündigt, 12.09.2024 gestrichen) und Umsatzschwellen [R]. Vertragsrisiko bleibt [A].
- Unreal: 5-%-Royalty/Seat-Modell [R].
- Slint: Royalty-free-Lizenz für proprietäre Desktop-Apps, aber an Bedingungen geknüpft (Details/Attribution im Licensing-FAQ, nicht geprüft) [R teilweise].
- Rust-Stack (wgpu, egui, wasmtime, bevy_vrm): permissive Lizenzen (MIT/Apache) [R] – geringstes Lizenzrisiko.
- GPL-Fallen: GPL-Code in Plugins/Assets; Slint-GPL-Option nicht für proprietäre Nutzung [R].
- Assets/VRM-Lizenzen (Modelle, Avatar-Nutzungsbedingungen) sind eigener Themenkomplex, nicht recherchiert.

## 6. Plugin-/Package-Systeme
- **wasmtime** 50.0.0 (20.10.2026), Apache-2.0, Fokus auf Korrektheit/Sicherheit, OSS-Fuzz, Security Policy, konfigurierbare CPU-/Speicherlimits; WASI/wasm32-wasip2 für Components [R, https://github.com/bytecodealliance/wasmtime]. Fuel/Epoch-Limits auf der Seite nicht erwähnt – API-Dokumentation separat prüfen. Sandbox mit Capability-Modell ist für Community-Erweiterungen am besten geeignet [A].
- Lua (mlua o. ä.): **nicht recherchiert**; Sandboxing muss manuell erfolgen [A].
- Native Plugins (DLL): keine Sandbox, hohes Sicherheitsrisiko, ABI-Instabilität (Rust) [A]; nur für vertrauenswürdige/signierte Plugins.
- Godot-/Unity-Plugins: laufen im Engine-Prozess ohne Sandbox [A].
- Web-Stack: JS-Plugins in iframe/Worker-Sandbox möglich [A].

## 7. Testing (Visual Regression)
- wgpu-CI-Mapping (aus Forks/docs.rs, unvollständig verifiziert): DX12/WARP unter Windows, Vulkan/lavapipe unter Linux, GL/llvmpipe, Metal auf Hardware [R, via https://docs.rs/crate/wgpu/=24 und Forks; Upstream-README nicht direkt bestätigt]. Lavapipe-CTS-Hang erwähnt [R, Forks].
- Vorgehen (Muster): deterministische Szene offscreen rendern, GPU-Sync, Pixel zurücklesen, mit Referenz-PNG vergleichen; Kamera/Zeit/Seed fixieren [R: Muster beschrieben, https://vgpu.labs.vercel.dev]. Software-Rasterizer weichen von Hardware ab – Toleranzen/Per-Backend-Referenzen nötig [A].
- egui_kittest: im Repo erwähnt, Details zu Snapshots nicht verifiziert [R teilweise].
- Eine dedizierte, offizielle wgpu-Anleitung für Visual Regression wurde **nicht gefunden** [R].

## 8. Bestehende Machbarkeitsbelege
- Rust: serde_vrm, bevy_vrm (Laden, MToon, Spring Bones), bevy_vrm1 (VRM 1.0 + VRMA) [R]. **Kein Rust-VRM-Exporter belegt.**
- Godot: godot-vrm mit Import/Export VRM 1.0 [R].
- Unity: UniVRM mit Import/Export zur Laufzeit [R].
- Web: three-vrm (Laden, MToon, WebGPU-Node-Material) [R]; Exporter nicht belegt.
- **Character-Editor-Referenzprojekte** (z. B. Open-Source-Editoren auf three-vrm): in dieser Recherche **nicht verifiziert** – kein Beleg.

## 9. Bewertungsmatrix (1 = schlecht, 5 = sehr gut; Begründungen überwiegend [A] bzw. abgeleitet aus [R]-Fakten)

Kriterien: F = Funktionsabdeckung, W = Wartbarkeit, L = Lizenz, P = Performance, I = Integrationsaufwand (5 = gering), R = Entwicklungsrisiko (5 = gering), E = Erweiterbarkeit.

| Option | F | W | L | P | I | R | E | Kurzbegründung |
|---|---|---|---|---|---|---|---|---|
| Rust + wgpu + egui (Eigenbau) | 3 | 3 | 5 | 5 | 2 | 3 | 5 | Alle Lizenzen permissiv [R]; AccessKit eingebaut [R]; kein VRM-Export vorhanden, Docking-Crate hinkt [R]; viel Eigenbau |
| Rust + Bevy (+ bevy_vrm) | 3 | 2 | 4 | 4 | 3 | 2 | 4 | MToon/Loader vorhanden [R], aber Lücken (Expressions, Constraints) und Editor-Tooling unreif [R]; häufige Breaking Changes |
| Rust-Core + Slint-UI | 3 | 3 | 3 | 5 | 2 | 3 | 4 | Slint aktiv [R], Royalty-free-Lizenz mit Bedingungen [R]; Docking/A11y unbelegt |
| Rust-Core + Tauri/WebView-UI | 3 | 3 | 5 | 4 | 3 | 3 | 5 | Web-UI-Reife [A]; Viewport-Einbettung heikel [A]; WebGPU in WebView ungeprüft |
| Tauri + three.js + three-vrm | 4 | 3 | 5 | 3 | 4 | 3 | 4 | three-vrm aktuell [R]; Exporter ungeklärt [R]; Sculpting-Performance fraglich [A] |
| Godot 4 + godot-vrm | 4 | 4 | 5 | 4 | 4 | 3 | 3 | Import/Export vorhanden [R]; Spring-Bone-Export fehlt [R]; Add-on-Aktualität unklar [R]; Studio-Look nicht Standard |
| Unity + UniVRM | 5 | 4 | 2 | 4 | 4 | 3 | 3 | Bester VRM-Export [R]; Lizenz-/Vertragsrisiko [R]; Plugins ohne Sandbox [A] |
| Unreal | 2 | 3 | 2 | 5 | 2 | 2 | 3 | Royalty/Seats [R]; VRM nicht recherchiert; überdimensioniert [A] |
| C++/Qt + eigener Renderer (bgfx/Diligent) | 3 | 3 | 2 | 5 | 1 | 2 | 4 | LGPL-Pflichten [R]; reifste Editor-UI [A]; höchster Aufwand |

## 10. Empfehlung

**Primärempfehlung: Hybrid mit Rust-Core (wgpu, eigener VRM-1.0-glTF-Reader/-Writer auf Basis von serde_vrm-Typen) und egui als UI-Schicht, mit wasmtime für Community-Packages.**
Begründung: permissive Lizenzen durchgängig [R]; AccessKit seit egui 0.34 eingebaut und in 0.37 verbessert [R]; wgpu-Windows-Backends first-class [R]; wasmtime bietet ein belastbar gepflegtes Sandbox-Modell [R]; Kern bleibt UI-unabhängig, sodass ein späterer UI-Wechsel (z. B. Slint oder Web-UI) möglich ist [A]. Hauptlücke: VRM-Export und Docking-Ökosystem sind Eigenbau bzw. nachlaufend [R]. Die Entscheidung bleibt bis zu den Spikes unter Vorbehalt.

**Alternative A: Godot 4 + godot-vrm.** Schnellster Weg zu einem funktionsfähigen Prototyp mit Import/Export und Docking-UI [R/A]. Risiken: Spring-Bone-Export fehlt, Add-on-Pflege, Sandbox-Plugins fehlen, Look eines "eigenständigen Studios".

**Alternative B: Tauri + three.js + three-vrm.** Beste UI-Produktivität und Barrierefreiheit über DOM [A]; Risiken: Exporter, WebGPU im WebView, Sculpting-Performance.

Nicht empfohlen: Unity (Lizenz-/Vertragsrisiko trotz bestem VRM-Export), Unreal (Royalty, Overkill), Qt (LGPL-Aufwand, außer bei kommerzieller Lizenz).

## 11. Offene Risiken und messbare Spikes

1. **VRM-1.0-Export-Spike (Rust):** serde_vrm + gltf-Writer; Round-Trip eines Beispielmodells; Akzeptanz: Export wird in three-vrm und UniVRM/VRM-Validator ohne Fehler geladen; Kennzahl: 100 % der Pflicht-Extensions (VRMC_vrm, materials_mtoon, springBone) round-trip-erhaltend.
2. **MToon-in-wgpu-Spike:** MToon-Port ohne Bevy, Vergleich mit three-vrm-Referenzbildern; Kennzahl: mittlere Pixelabweichung < definierter Schwelle (z. B. < 2 % ΔE), 60 FPS bei 3 Mio. Dreiecken auf Mittelklasse-GPU (Schwellen sind Vorschläge).
3. **egui-Docking/A11y-Spike:** Dock-Layout + Property-Editor mit 500 Feldern; Screenreader-Test (NVDA) der AccessKit-Baumstruktur; Kennzahl: Alle Felder per AccessKit erreichbar, Frame < 4 ms UI-Zeit. Prüfen, ob egui_dock für egui 0.37 verfügbar ist (aktuell kein Beleg).
4. **Morph/Skinning-Skalierung:** 100 Morph Targets, GPU-Compute vs. Vertex-Shader; Kennzahl: Slider-Latenz < 16 ms.
5. **wasmtime-Sandbox-Spike:** Fuel-/Memory-Limits, Capability-Host-API, bösartiges Test-Plugin; Kennzahl: Endlosschleife und Speicherbombe werden innerhalb definierter Grenzen abgebrochen.
6. **Visual-Regression-Pipeline:** Headless wgpu auf Windows (WARP) und Linux (lavapipe), 20 Referenzszenen; Kennzahl: Flake-Rate < 1 % bei festen Toleranzen; Unterschiede zwischen Software- und Hardware-Rasterizer messen.
7. **Alternativen-Spike Godot:** godot-vrm gegen Godot 4.7.2 testen (Add-on-Stand 2024-11 [R]), Export in UniVRM validieren.
8. **Web-Alternative:** WebGPU in WebView2 prüfen (nicht verifiziert) und Sculpting-Brush-Performance messen.
9. **Offene Recherchelücken:** Qt-Aktualstand, Godot-/Bevy-Lizenzen, Unity-/Unreal-Konditionen 2026 aus Primärquellen, Lua-Optionen, Open-Source-Charaktereditoren als Referenz, Barrierefreiheit von Slint/iced/Godot, Qt-Preise.

Hinweis: Alle Versionsdaten sind Momentaufnahmen vom 2026-10-23 (Abruf via GitHub-Releaseseiten). Lizenzentscheidungen vor Vertrieb juristisch prüfen.
