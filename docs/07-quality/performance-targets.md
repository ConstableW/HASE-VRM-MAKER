# Performance-Anforderungen, Zielwerte und Testbedingungen

**Maßgebliche Quelle für alle Performance-Ziele und Messregeln.** Status: Zielwerte `PROPOSED` (Owner nennt Referenzgeräte, OQ-09). Verbindlich ist die Vorgabe selbst: **Performance ist eine zentrale Architekturvorgabe (OD-025)**. Festgelegt am 2026-10-24 **vor** den Messungen des Performance-Spikes (SP-08), damit Ergebnisse gegen vorab definierte Werte bewertet werden.

Anforderungs-IDs: REQ-PERF-001…020 (`../00-project/requirements.md`). Architektur: `../02-architecture/performance-architecture.md`. Ergebnisse: `../04-research/benchmarks.md`.

## 1. Referenzhardware (Annahmen, bis der Owner Geräte nennt)

| Klasse | CPU | GPU | RAM | Windows-Ziel |
|---|---|---|---|---|
| **T-Min** | 4 Kerne, ~3 GHz | integrierte GPU (Intel Iris Xe / AMD Vega-8-Klasse), DX12/Vulkan | 8 GB | Windows 10/11 |
| **T-Ref** | 6–8 Kerne | dedizierte Mittelklasse (GTX 1660 / RTX 3050-Klasse), 4–6 GB VRAM | 16 GB | Windows 11 |

Messwerte aus anderen Umgebungen (z. B. Entwicklungs-Container ohne GPU) werden als **„CPU-Referenz“ bzw. „Software-Rasterizer“** gekennzeichnet und nie gegen die GPU-Ziele bewertet.

## 2. Zielwerte

Frame-Budget bei 60 fps = **16,6 ms**. Statistik immer als Median / p95 / p99 / Max.

| ID | Bereich | Zielwert (T-Ref) | Zielwert (T-Min) | Typ |
|---|---|---|---|---|
| REQ-PERF-001 | Interaktive Frame-Time (statisch, Charakter + Kleidung + Haar, 1080p, MSAA 4×, 1 Schatten) | p95 ≤ 8 ms GPU, ≤ 16,6 ms Frame | p95 ≤ 16 ms GPU, ≤ 33 ms Frame | Muss |
| REQ-PERF-002 | Kontinuierliche Parameteränderung (Slider-Drag), 64 aktive Morph-Targets | p95 Frame ≤ 16,6 ms; Morph-Aufwand ≤ 2 ms | p95 Frame ≤ 33 ms; Morph ≤ 4 ms | Muss |
| REQ-PERF-003 | Einzelne Parameteränderung wirkt sichtbar | ≤ 2 Frames (≤ 33 ms) Eingabe→Pixel | ≤ 3 Frames | Muss |
| REQ-PERF-004 | Alle 609 Targets gleichzeitig aktiv (Stress) | Morph ≤ 4 ms | Morph ≤ 8 ms | Soll |
| REQ-PERF-005 | UI-Reaktion: CPU-Zeit eines UI-Frames mit Property-Panel (≥ 600 Parameter sichtbar, virtualisiert ≥ 100 Widgets) | ≤ 2 ms | ≤ 4 ms | Muss |
| REQ-PERF-006 | Keine Operation > 100 ms im UI-Thread; Operationen > 1 s abbrechbar mit Fortschritt | 100 % | 100 % | Muss |
| REQ-PERF-007 | Inkrementelles Fitting: Neuberechnung **eines** Kleidungsstücks nach Parameteränderung (async) | ≤ 50 ms | ≤ 150 ms | Soll |
| REQ-PERF-008 | Nicht betroffene Komponenten werden bei einer Änderung nicht neu berechnet | 0 unnötige Neuberechnungen (Messung per Zähler) | gleich | Muss |
| REQ-PERF-009 | Projekt öffnen bis Charakter sichtbar (kalt) | ≤ 3 s | ≤ 6 s | Soll |
| REQ-PERF-010 | Package-Index mit 1.000 Packages / 10.000 Assets: Start aus Cache-Index | ≤ 1 s; Bibliotheksliste ≤ 2 ms/Frame (virtualisiert) | ≤ 3 s; ≤ 4 ms | Soll |
| REQ-PERF-011 | CPU-RAM Editor mit 1 Charakter + 50 geladenen Assets | ≤ 1,5 GB | ≤ 1 GB | Muss |
| REQ-PERF-012 | GPU-Speicher gleiche Szene | ≤ 1 GB | ≤ 512 MB | Muss |
| REQ-PERF-013 | Morph-Daten eines Bodys (609 Targets) im RAM, komprimiert/sparse | ≤ 100 MB | ≤ 100 MB | Muss |
| REQ-PERF-014 | Export (VRM) läuft in Hintergrund-Thread; UI-Frame-p95 ändert sich währenddessen um ≤ 10 % | ja | ja | Muss |
| REQ-PERF-015 | Asset-Laden asynchron; Platzhalter statt Blockade; kein Frame > 50 ms durch Laden | ja | ja | Muss |
| REQ-PERF-016 | Skinning + Posing (Animation abspielen, 60 Bones-Äquivalent, 150k Tris) | ≤ 1,5 ms GPU | ≤ 3 ms | Muss |
| REQ-PERF-017 | Skalierung: Frame-Time wächst höchstens linear mit Anzahl sichtbarer Assets; Bibliotheksgröße beeinflusst Frame-Time nicht | ja | ja | Muss |
| REQ-PERF-018 | Profiling eingebaut: Frame-Time, CPU-/GPU-Zeit je Phase, Speicher, Zähler (Neuberechnungen) jederzeit abrufbar (Overlay + Datei) | ab erstem Walking Skeleton | gleich | Muss |
| REQ-PERF-019 | Performance-Regressionstest in CI (Benchmark mit Schwellwerten, Warnung ab +20 %) | ab I0 | gleich | Soll |
| REQ-PERF-020 | Spezies-/Package-Erweiterbarkeit darf Zielwerte nicht verletzen: Mehrfach-Body-Familien, 1.000 Packages, T1-Erweiterungen | ja | ja | Muss |

## 3. Testbedingungen (für alle Benchmarks)

- **Build:** `--release`, LTO/opt-level 3; keine Debug-Overlays.
- **Szene:** reales humanoides Basismesh (Testcandidate: 13.380 Körper-Vertices bzw. 19.158 inkl. Helper) + Proxys; repräsentative Morph-Targets aus `content/testcandidate/mpfb2-cc0/targets/` (609 Dateien).
- **Rendering:** 1920×1080, MSAA 4×, vsync **aus**, eine Richtungslicht-Quelle mit Schatten, MToon-ähnlicher Shader (Stufe 2: Platzhalter-Shader mit Normal-/Lichtberechnung).
- **Protokoll:** 120 Warm-up-Frames, 600 Messframes, 3 Wiederholungen; Ergebnis = Median der Mediane; p95/p99/Max je Lauf.
- **Umgebung erfassen:** OS, CPU, GPU + Treiber, wgpu-Backend (DX12/Vulkan), Rust-/Crate-Versionen, Leistungsplan.
- **Trennung:** *Gemessen* (mit Datum, Hardware, Befehl) vs. *Geschätzt* (explizit als ASSUMED markiert).
- **Reproduzierbar:** Benchmark-Code in `bench/morph-spike/`, ein Befehl, JSON-Ausgabe.

## 4. Bewertungslogik
Ein Stack besteht, wenn er alle **Muss**-Ziele auf T-Ref **und** T-Min erreicht (oder die Abweichung ohne Architekturänderung behebbar ist) und die **Soll**-Ziele nicht grundsätzlich verhindert. Nicht gemessene Ziele bleiben offen und blockieren die Freigabe von ADR-0002 (Owner-Entscheidung, OD-018).
