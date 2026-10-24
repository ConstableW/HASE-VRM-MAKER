# morph-spike – Performance-Spike SP-08

Misst Morphing, Skinning, Rendering (wgpu) und UI-Kosten (egui) auf den **realen** Testcandidate-Daten (Basismesh, 610 Morph-Targets, Skin-Gewichte). Zielwerte/Testbedingungen: `docs/07-quality/performance-targets.md`. Ergebnisse/Auswertung: `docs/04-research/benchmarks.md`.

```
cargo run --release -- cpu            # CPU-Referenz Morph/Skinning/Normalen (kein GPU-Bezug)
cargo run --release -- ui             # egui-Layout+Tessellation (CPU)
cargo run --release -- gpu --copies 6 --frames 600   # wgpu offscreen 1080p, MSAA4, Schatten, Timestamps
cargo run --release -- all            # alles, JSON auf stdout
```
Windows (echte GPU): `powershell -File bench/morph-spike/run-benchmark.ps1` → `bench/results/<Datum>_<Rechner>.json`.
Im Linux-Container ohne GPU: `VK_ICD_FILENAMES=<chromium>/vk_swiftshader_icd.json` nutzen (Software-Vulkan; nur Funktionsnachweis, **keine** GPU-Zeiten).

Szenarien (gpu): statisch · 1 / 64 / alle Targets kontinuierlich verändert · nur Compute. Ausgabe je Szenario: Frame-Wall-Time, CPU-Encode/Submit, GPU-Zeit je Pass (Timestamp-Queries, falls unterstützt). Wissens-Grenze: Messung = Offscreen, ohne Fenster/Present und ohne UI-Rendering.
