# Benchmarks (Performance-Spike SP-08)

**Maßgebliche Quelle für gemessene Performance-Werte.** Zielwerte und Testbedingungen wurden **vorab** festgelegt: `../07-quality/performance-targets.md` (Commit `84c8df5`, vor der ersten Messung). Code: `../../bench/morph-spike/`. Rohdaten: `../../bench/results/2026-10-24_container-xeon4_swiftshader.json`. Bewertung/Entscheidung: `../05-decisions/ADR-0002-technologie-stack.md`.

## 0. Wichtigste Aussagen

1. **Gemessen:** CPU-Morphing mit Sparse-Deltas ist auf den realen Daten extrem billig (64 aktive Targets: **0,14 ms**; alle 610: **1,8 ms**; Einzeländerung eines Targets: **~0,001 ms**). Speicher der Morph-Daten: **13 MB** statt 93 MB dicht.
2. **Gemessen:** egui-UI-Kosten sind bei virtualisierten Listen klein (Parameterpanel mit 610 Parametern: **~0,33 ms** CPU/Frame; Bibliothek mit 10.000 Einträgen: **0,06 ms**) – aber **2,3 ms ohne Virtualisierung** (Worst Case, verfehlt das 2-ms-Ziel). Hintergrundlast (2 Threads Volllast) verändert den UI-Frame nicht (0,19 → 0,21 ms).
3. **Gemessen (nur Funktionsnachweis):** Die wgpu-Pipeline (Compute-Morphing, Skinning, Schattenpass, MSAA, Timestamp-Queries) läuft vollständig durch. **Die GPU-Zeiten aus dem Container sind Software-Rasterizer-Werte (SwiftShader, CPU) und für die GPU-Ziele wertlos.**
4. **Offen:** Echte GPU-Frame-Times (REQ-PERF-001/-016) brauchen einen Lauf auf Windows-Hardware (`run-benchmark.ps1`). Ohne diesen Lauf bleibt ADR-0002 nicht freigabefähig.

## 1. Testumgebung dieses Laufs (Datum 2026-10-24)
Intel Xeon @ 2,8 GHz, 4 vCPU, 15 GB RAM, Linux, **keine GPU**. Rust 1.97.0, wgpu 30.0.1, egui 0.37.0, `--release` (LTO thin, 1 Codegen-Unit). GPU-Adapter: *SwiftShader Device (Subzero), Vulkan, DeviceType::Cpu*. Daten: Testcandidate (`base.obj` body-Gruppe: **13.380 Vertices, 26.756 Dreiecke**; **610 Morph-Targets** – die in früheren Dokumenten genannte Zahl „609“ war um 1 zu niedrig; Skin: 53 Bones, 4 Gewichte/Vertex). Protokoll CPU: 300 Wiederholungen nach Warm-up; UI: 600 Frames nach 30 Warm-up; GPU: 60 Messframes nach 60 Warm-up (Software-Rasterizer, daher kurz).

## 2. Gemessen – Daten und Speicher (CPU, repräsentativ bis auf absolute CPU-Geschwindigkeit)

| Größe | Wert |
|---|---|
| Nicht-Null-Deltas gesamt (nnz) | 867.423 (Ø 1.422 je Target; ≈ 10,6 % Dichte) |
| Sparse-CSR je Target (Index + f32×3) | **13,2 MB** |
| Zusätzlich Vertex-CSR (Gather-/GPU-Layout) | 13,3 MB |
| Dicht f32×3 (610 × 13.380 × 12 B) | 93,4 MB |
| Dicht f16×4 | 62,3 MB |
| Max. Einträge je Vertex (Gather-Schleife) | 111 |
| RSS nach Laden (Mesh + Targets in 2 Layouts + Skin) | 37,6 MB (Δ 35 MB) |
| Ladezeit aus Rohdateien | Mesh 9 ms · **Targets 464 ms** (gz-Text parsen) · **Skin 851 ms** (JSON) |

→ Ziel REQ-PERF-013 (≤ 100 MB) mit großem Abstand erfüllt. Die Ladezeiten aus Textformaten sind für den Produktbetrieb zu hoch: **binärer, speicherabbildbarer Cache** (siehe `performance-architecture.md` §6) ist Pflicht.

## 3. Gemessen – Morphing (CPU, 1 Thread, Median / p95, ms)

| Aktive Targets | Scatter je Target (Sparse) | Gather je Vertex | Gather (4 Threads, rayon) |
|---|---|---|---|
| 1 | **0,009** / 0,012 | 0,97 / 1,08 | 0,38 / 0,64 |
| 8 | 0,015 / 0,016 | 1,05 / 1,79 | 0,40 / 0,65 |
| 64 | **0,139** / 0,183 | 1,38 / 1,50 | 0,46 / 0,76 |
| 128 | 0,467 / 0,591 | 1,58 / 1,79 | 0,51 / 0,65 |
| 610 (alle) | **1,769** / 3,106 | 1,58 / 2,77 | 0,50 / 0,62 |

- **Inkrementell** (nur *ein* Target ändert sich, `pos += Δw·delta`): Median **0,0009 ms**, p95 0,022, Max 0,064 über alle 610 Targets.
- CPU-Normalen-Neuberechnung (26.756 Dreiecke): 0,13 ms. CPU-Skinning (4 Gewichte, 13.380 Vertices): 0,18 ms.

**Folgerung (belegt für diese Daten):** Mit *Scatter über nur aktive Targets* ist der Morph-Aufwand proportional zu den aktiven Deltas, nicht zur Bibliotheksgröße (REQ-PERF-017). Der Gather-Pfad (Standard auf der GPU) lohnt auf der CPU nicht. Ein GPU-Compute-Pfad ist für **einen** Charakter dieser Größe nicht zwingend, sondern Reserve für höhere Auflösung, viele Charaktere und Kleidung.

## 4. Gemessen – UI (egui 0.37, CPU: Layout + Tessellation, 1920×1080, ms)

| Szenario | Layout Median / p95 | Tessellation Median |
|---|---|---|
| Parameterpanel, 610 Slider, **virtualisiert** (`show_rows`) | 0,186 / 0,232 | 0,144 |
| … mit Wertänderung in jedem Frame | 0,304 / 0,520 | 0,156 |
| … während 2 Worker-Threads Volllast | 0,206 / 0,256 | – |
| Bibliothek, 10.000 Einträge, virtualisiert | 0,059 / 0,082 | 0,012 |
| Parameterpanel, 610 Slider, **nicht virtualisiert** (Worst Case) | **2,273 / 2,517** | 0,173 |

Nicht enthalten: GPU-Rendering der UI, Texteingabe/Hover-Animationen, Fenster-/Eventloop-Kosten. → REQ-PERF-005 (≤ 2 ms) mit Virtualisierung erfüllt; ohne Virtualisierung **verfehlt**. Konsequenz: Virtualisierung ist Architekturregel.

## 5. Gemessen – wgpu-Pipeline im Software-Rasterizer (NICHT repräsentativ)

Offscreen 1080p, MSAA 4×, Schattenpass 2048², GPU-Morph-Compute (Gather), Vertex-Pulling-Skinning mit 53 Bones. Adapter: SwiftShader (CPU, 4 Threads).

| Szenario (1 Charakter, 26.756 Tris) | Frame (Median) | GPU-Pässe laut Timestamp | CPU Encode+Submit |
|---|---|---|---|
| Statisch | 53,5 ms | Haupt 42,5 · Schatten 10,1 | 0,15 ms |
| 1 Target verändert | 79,5 ms | + Morph-Compute 7,3 | 0,14 ms |
| 64 Targets verändert | 83,0 ms | + Morph-Compute 8,6 | 0,15 ms |
| alle 610 verändert | 82,2 ms | + Morph-Compute 8,7 | 0,16 ms |
| nur Compute (alle Targets) | 7,2 ms | Compute 7,1 | 0,09 ms |

Zwei Kopien: ca. doppelte Zeiten (Compute 13–16 ms). **Lesehinweis:** Absolute Zeiten sind CPU-Emulation und beweisen nichts über echte GPUs. Verwertbar sind nur (a) die Funktionsfähigkeit der gesamten Pipeline, (b) **CPU-Encode+Submit ≈ 0,15 ms je Frame** (dieser Teil läuft real auf der CPU), (c) lineare Skalierung mit Vertexzahl, (d) berechneter GPU-Speicher.

**GPU-Speicher (berechnet, nicht gemessen):** ≈ 105 MB für einen Charakter bei 1080p: Render-Targets (MSAA-Farbe 33 MB + Tiefe 33 MB + Resolve 8 MB + Schatten 16 MB) ≈ 90 MB, Morph-Daten 17 MB (Vertex-CSR 16 B + 4 B je Eintrag), Mesh/Skinning < 2 MB. REQ-PERF-012 (≤ 1 GB) Reserve groß; Texturen (4–16 MB je Atlas) sind nicht eingerechnet.

## 6. Geschätzt (ASSUMED, nicht gemessen)

- GPU-Morph-Dispatch: 867k Einträge × ~20 B ≈ 17 MB Lesevolumen; bei 30–100 GB/s (iGPU bis Mittelklasse) **≈ 0,2–0,6 ms** [Rechnung aus Bandbreite; Messung offen].
- Statisches Rendering eines Charakters mit ~150k Dreiecken, MSAA 4×, Schatten auf Mittelklasse-GPU: typischerweise im niedrigen einstelligen ms-Bereich [Erfahrungswert, keine Quelle, Messung offen].
- Hochauflösende Köpfe/Haare (Faktor 5 an Vertices/nnz) lassen den CPU-Scatter-Aufwand etwa linear wachsen (64 Targets ≈ 0,7 ms) [Extrapolation].

## 7. Bewertung gegen die Ziele (T-Ref/T-Min)

| Ziel | Ergebnis | Status |
|---|---|---|
| PERF-002 Morph ≤ 2 ms bei 64 aktiven | 0,14 ms (CPU-Scatter) | **erfüllt (gemessen, CPU)** |
| PERF-004 alle Targets ≤ 4 ms | 1,77 ms (CPU, 1 Thread) | **erfüllt (gemessen, CPU)** |
| PERF-005 UI ≤ 2 ms | 0,33 ms virtualisiert / 2,4 ms nicht | **erfüllt nur mit Virtualisierung** |
| PERF-008 nur Betroffenes neu berechnen | Einzel-Target-Update 0,001 ms vs. 0,14–1,8 ms voll | **Design belegt**; Zähler im Produkt offen |
| PERF-010 große Bibliotheken | 10.000 Einträge 0,06 ms | **erfüllt (UI-Anteil)** |
| PERF-012 GPU-Speicher | ≈ 105 MB (berechnet) | erfüllt (berechnet) |
| PERF-013 Morph-Daten ≤ 100 MB | 13,2 MB | **erfüllt (gemessen)** |
| PERF-014 UI unter Hintergrundlast | 0,19 → 0,21 ms | **erfüllt (Proxy, gemessen)** |
| PERF-009 Laden ≤ 3 s | Rohdateien: 1,3 s allein für Targets+Skin | **verfehlt ohne Binär-Cache** (Cache nötig) |
| PERF-001 statische Frame-Time (GPU) | nicht messbar im Container | **offen – Windows-Lauf nötig** |
| PERF-016 Skinning/Posing (GPU) | CPU-Skinning 0,18 ms; GPU offen | **offen** |
| PERF-003 Eingabe→Pixel ≤ 2 Frames | Architekturfrage (kein Fenster-/Present-Test) | **offen** |
| PERF-006/-007/-015/-017/-018/-019/-020 | Architekturregeln, noch nicht im Prototyp | **offen** |

## 8. Wiederholung auf Zielhardware
**Einfachster Weg:** `bench/win/Benchmark-starten.bat` doppelklicken (fertige Windows-exe, DX12- und Vulkan-Lauf, kein Rust nötig; exe im Container gebaut, **nicht unter Windows getestet**). Alternativ `powershell -File bench/morph-spike/run-benchmark.ps1` (Rust nötig). Ergebnis: JSON in `bench/results/`. Bitte mit GPU-Modell, Treiber, Leistungsplan an Claude zurückgeben bzw. committen; die Auswertung ergänzt Abschnitt 5 um echte Werte.
