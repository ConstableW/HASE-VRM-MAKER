# ADR-0002: Technologie-Stack (Rust + wgpu + egui vs. Alternativen) – unter Performance-Vorgabe

- **Status:** PROPOSED – **nicht freigabefähig**, solange der GPU-Lauf auf Windows-Zielhardware (Spike SP-08, Teil 2) fehlt. Entscheidung liegt beim Product Owner (OD-018).
- **Datum:** 2026-10-24
- **Betrifft:** OD-025, REQ-PERF-001…020, REQ-PROD-002, REQ-LEG-001, REQ-UX-014, R-10, R-12, R-15, OQ-01, OQ-09
- **Evidenz:** `../04-research/benchmarks.md` (gemessen), `../04-research/technology-evaluation.md` (Recherche), `../02-architecture/performance-architecture.md`

## Kontext
Performance ist zentrale Architekturvorgabe (OD-025). Der Stack-Vorschlag Rust + wgpu + egui soll an konkreten Anforderungen geprüft und mit Alternativen verglichen werden – nicht nach Sprache oder Lizenz allein. Die Spezies-/Package-Architektur bleibt unverändert.

## Was der Spike gezeigt hat (gemessen vs. offen)
| Anforderung | Aussage | Beleg |
|---|---|---|
| Morph-Aufwand | 64 aktive Targets 0,14 ms, alle 610 1,8 ms, Einzelupdate 0,001 ms (CPU, 1 Thread) | gemessen |
| Morph-Speicher | 13 MB sparse vs. 93 MB dicht | gemessen |
| UI-Kosten | virtualisiert 0,3 ms/Frame, nicht virtualisiert 2,4 ms; unabhängig von Hintergrundlast | gemessen |
| wgpu-Pipeline (Compute-Morph, Skinning, Schatten, MSAA, Timestamps) | lauffähig; CPU-Encode+Submit 0,15 ms/Frame | gemessen (Funktion) |
| GPU-Frame-Times, GPU-Auslastung | im Container nicht messbar | **offen** |
| Eingabe→Pixel-Latenz, Present, Fenster, echte egui-Render-Kosten | nicht gemessen | **offen** |

**Schlüsselbefund:** Die leistungskritischen Operationen (Sparse-Morph, Skinning, inkrementelles Fitting, Cache-Design) sind **sprach- und engineunabhängig** und billig, wenn Datenlayout und Dirty-Tracking stimmen. Die Performance entscheidet sich daher weniger an „Rust vs. X“ als an Architekturregeln (`performance-architecture.md`), und an drei stackabhängigen Punkten: (1) Kontrolle über Threading/Speicher/GPU-Layout, (2) UI-Latenz und -Skalierung, (3) Aufwand für editorspezifische GPU-Features (Picking, Brush, Sculpt).

## Alternativen (Bewertung 1–5, Performance-Gewicht ×2; Begründung; Belegstufe)
| Kriterium (Gewicht) | A Rust+wgpu+egui | B Rust+wgpu+Slint | C Rust-Core + Tauri-WebView-UI + wgpu-Viewport | D Godot 4 (+godot-vrm, GDExtension) | E Unity + UniVRM | F Web (three.js/WebGPU in Tauri) |
|---|---|---|---|---|---|---|
| Morph-/Skinning-Kontrolle (×2) | 5 – volle Kontrolle, gemessen | 5 | 5 | 3 – Engine-Mesh-API, Blendshape-Limits zu prüfen [A] | 4 – GPU-Blendshapes, dicht/speicherhungrig [A] | 3 – Morph-Texturen, JS/WASM-Grenzen [A] |
| UI-Latenz/-Skalierung (×2) | 4 – gemessen schnell, Virtualisierungsdisziplin nötig | 4 [A] | 3 – IPC + Viewport-Einbettung heikel [A] | 4 – Control-UI integriert [A] | 4 [A] | 3 – DOM bei großen Listen/Overlays [A] |
| Threading/Main-Thread-Freiheit (×2) | 5 – Rust, Send/Sync, gemessen | 5 | 4 | 3 – Szenenbaum-Threadregeln [A] | 3 – Main-Thread-gebunden [A] | 3 – Worker, kein geteilter Speicher ohne Aufwand [A] |
| Editor-GPU-Features (Picking, Brush) | 3 – Eigenbau | 3 | 3 | 4 – vorhanden/erweiterbar [A] | 4 [A] | 3 [A] |
| Speicherkontrolle (×2) | 5 | 5 | 4 | 3 | 3 | 3 |
| Windows/Distribution/Lizenz | 5 – MIT/Apache | 4 – Slint-Lizenzbedingungen [R] | 5 | 5 – MIT | 2 – Vertragsrisiko [R] | 5 |
| Wartbarkeit/Reife | 3 – schnelllebige Crates (wgpu quartalsweise Breaking) [R] | 3 | 3 | 4 | 4 | 4 |
| Barrierefreiheit | 4 – AccessKit [R] | 3 [A] | 5 – DOM [A] | 3 [A] | 3 [A] | 5 [A] |
| Erweiterbarkeit/Package-Sandbox | 5 – wasmtime [R] | 5 | 4 | 3 | 3 | 4 |
| **Summe (Gewichte: Morph, UI-Latenz, Threading, Speicher je ×2; übrige ×1; Max. 65)** | **58** | 56 | 52 | 45 | 44 | 45 |

`[R]` = Recherche mit Quellen (`technology-evaluation.md`), `[A]` = Annahme ohne eigene Messung. Die Punktzahlen sind eine **subjektive Bewertung** des Bearbeiters, keine Messung; nur Spalte A (und B teilweise, gleicher Render-Kern) ist im Spike gemessen; die anderen Spalten sind **nicht** gegengemessen und bleiben Schätzung, bis Spikes sie belegen (SP-08b: Godot-GDExtension-Morph-Test, SP-08c: three.js-Morph-Texturen).

## Entscheidung (Empfehlung, PROPOSED)
**Rust-Core + wgpu + egui bleibt der empfohlene Stack – unter drei Bedingungen:**
1. **Abnahmebedingung:** Der Benchmark auf Windows-Zielhardware (`bench/morph-spike/run-benchmark.ps1`) erfüllt REQ-PERF-001, -016 auf T-Ref und T-Min (Zielwerte vorab fixiert). Verfehlt er sie, wird die Empfehlung neu bewertet (Szenario: Engine D/E für Viewport, Kern bleibt Rust).
2. **Architekturregeln verbindlich:** `performance-architecture.md` §1–§10 (Snapshot-Zustand, Dirty-Graph, Sparse-Morphs, Binär-Cache, Virtualisierung, Worker-Export, Profiling ab Tag 1).
3. **UI-Austauschbarkeit wahren:** Der Core hat keine egui-Abhängigkeit; UI ist eine Schicht über Command-Bus und Snapshots. Ein späterer Wechsel (Slint/Web) bleibt möglich; Viewport-Rendering ist von der UI-Bibliothek getrennt.

## Begründung
- **Warum nicht „einfach Engine“:** Der Performance-Vorteil einer Engine liegt im Rendering, das hier ohnehin standardisiert ist. Der Aufwand (Morph, Fitting, Cache, Export-Determinismus, Package-System) liegt im Kern, den wir in jeder Variante selbst bauen – in Rust mit voller Kontrolle über Threads und Datenlayout.
- **Gemessen:** CPU-/UI-Pfade haben große Reserven (Faktor 10–100 gegen die Ziele), die GPU-Pipeline funktioniert. Das größte Risiko liegt nicht in der Rechenleistung, sondern in Entwicklungsaufwand und Crate-Stabilität.
- **Schwäche ehrlich:** egui ist Immediate-Mode: Ohne Virtualisierung verfehlt es das UI-Ziel (2,4 ms). Komplexe Docking-/Tabellen-UIs und Barrierefreiheit müssen früh im Walking Skeleton geprüft werden (R-15).

## Konsequenzen
- OQ-01 bleibt offen bis zur Freigabe; OQ-09 (Referenzgeräte) wird dringend: Owner nennt Hardware oder führt den Benchmark aus.
- Neue Spikes: SP-08b/c als Gegenmessung der Alternativen, falls der Windows-Lauf ein Ziel verfehlt.
- Kein Produktcode ist an diese Entscheidung gebunden; reversibel.
