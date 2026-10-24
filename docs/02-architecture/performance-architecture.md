# Performance-Architektur

**Maßgebliche Quelle für die Performance-Architekturregeln.** Status: `PROPOSED`, gestützt auf Messungen (`../04-research/benchmarks.md`, Spike SP-08) und die Zielwerte `../07-quality/performance-targets.md`. Vorgabe: OD-025. Die Spezies-/Package-Architektur (`species-system.md`, `package-system.md`) bleibt unverändert verbindlich; jede Regel hier ist mit ihr verträglich (siehe §11).

Belegstufen: **[M]** gemessen im Spike · **[A]** Annahme/Schätzung, noch zu messen · **[D]** Designregel.

## 1. Leitprinzipien

1. **Arbeit proportional zur Änderung, nicht zur Modellgröße.** Jeder Zustand hat Abhängigkeiten und Dirty-Flags; nur Betroffenes wird neu berechnet [D].
2. **Der Render-/UI-Thread tut nur Wiedergabe.** Alles, was > 1 ms dauern kann, läuft auf Worker-Threads mit Abbruch und Fortschritt [D].
3. **Daten im Speicherlayout der Verarbeitung, nicht im Dateilayout.** Importformate werden einmalig in einen binären, speicherabbildbaren Cache übersetzt [D, M: Textladen 1,3 s].
4. **Messen, bevor man optimiert – und von Beginn an.** Profiling ist Teil des Kerns (REQ-PERF-018) [D].
5. **Sparse statt dicht.** Morphs, Gewichte, Fits werden als Deltas/Listen gespeichert [M: 13 MB statt 93 MB].

## 2. Schichten und Threads (Punkt 3 der Vorgabe)

```
UI-Thread (Eventloop)        Core-Thread(s)                   Worker-Pool                 GPU
─────────────────────        ───────────────                  ───────────                 ───
egui-Panels, Eingaben   ──►  Command-Bus → Projektzustand ──► Jobs: Laden/Dekodieren,    Compute: Morph (optional)
Gizmos, Overlays             Versionierter Snapshot            Fitting, Normalen,          Render: Schatten, Haupt,
Profiling-Overlay       ◄──  (immutabel, Arc)             ◄──  Export, Validierung,        Skinning (Vertex Pulling)
                             Evaluator (Dirty-Graph)           Thumbnails, Hashing
```
- **Projektzustand** (Parameter, Asset-Instanzen, Delta-Layer; `project-format.md`) ist klein und **versioniert unveränderlich** (Snapshot via `Arc`). UI liest den letzten fertigen Snapshot; Evaluator arbeitet an der nächsten Version.
- **Commands** (Undo/Redo, REQ-UX-009) sind die einzige Schreibstelle; Slider-Drags werden zu einem Command mit koaleszierten Werten zusammengefasst.
- **Evaluator** baut einen *Dirty-Graph*: Parameter → Morph-Zustand → Rig-Fit → Asset-Fits → Materialien → GPU-Upload. Jede Stufe cached ihr Ergebnis unter einem Schlüssel aus Eingabe-Digests; nicht betroffene Knoten bleiben unangetastet (REQ-PERF-008; Zähler im Profiler).
- Ergebnisse kommen als **Fertig-Nachrichten** zurück; stale Ergebnisse (veraltete Version) werden verworfen. Während Arbeit läuft, zeigt die UI den letzten gültigen Stand (kein Einfrieren).

## 3. Rendering (Punkt 1)
- **GPU-Render mit expliziten Passes:** Schatten (2048²), Hauptpass (MSAA 4×), optional Outline/Post. Pipeline im Spike lauffähig [M, nur Funktion].
- **Vertex Pulling + Skinning im Vertex-Shader** aus Storage-Buffern; Geometrie wird **nicht** pro Frame neu hochgeladen, nur Änderungen (Bones je Frame, Morph-Ergebnisse bei Änderung) [D].
- **Materialverwaltung:** Materialien nach Shader/Pipeline sortiert, Texturen in geteilten Atlanten/Arrays; MToon-Parameter in einem Storage-Buffer, nicht je Draw-Uniform [D/A].
- **Normalen:** Im Vorschaupfad aus Screen-Space-Derivaten (kein Normalen-Update je Morph); exakte Normalen nur bei Export/Stillstand (CPU 0,13 ms [M] oder Compute) [D].
- **Beleuchtung/Schatten:** Eine Hauptlichtquelle + Ambient; Presets (REQ-UX-010) ändern Uniforms, keine Pipelines [D].
- **Vorschau-LOD:** Bei Interaktion optional reduzierte Qualität (MSAA 2×, Schattenauflösung), bei Stillstand volle Qualität [A].

## 4. Morphing (Punkt 2)
- **Datenhaltung:** je Body-Package Morph-Daten als *Sparse-Deltas* (Index+Vektor, optional f16/quantisiert), geordnet je Target (Scatter) und optional je Vertex (GPU-Gather) [M].
- **Aktivierungsmodell:** nur Targets mit Gewicht ≠ 0 sind „aktiv“; Auswertung = Summe über *aktive* Deltas. Bibliotheksgröße (610 oder 5.000 Targets) beeinflusst die Frame-Time nicht [M: 0,14 ms bei 64 aktiven].
- **Inkrementell:** bei Änderung eines Parameters wird `pos += Δw · delta_t` angewendet (0,001 ms [M]); voller Neuaufbau nur bei Strukturwechsel (Body/Spezies/Undo über mehrere Schritte).
- **Wo gerechnet wird:** Standard = CPU-Scatter auf Core-/Worker-Thread, Ergebnis per `queue.write_buffer` (Teil-Update) zur GPU. **GPU-Compute-Gather** (im Spike implementiert) ist die Skalierungsreserve für hohe Auflösung, viele Charaktere, Kleidung mit eigenen Morphs [D; Entscheidung nach Windows-Messung].
- **Zonen/Symmetrie:** Parameter tragen Zonen-IDs → nur betroffene Vertex-Bereiche werden aktualisiert [A].

## 5. Rigging und Animation (Punkt 4)
- **Skinning auf der GPU** (Vertex Pulling, 4 Einflüsse) mit Bone-Matrizen als Storage-Buffer; je Frame nur Matrizen hochladen (53 Bones × 64 B) [M: Pipeline läuft; CPU-Referenz 0,18 ms].
- **Speziesabhängige Zusatzknochen** (Ohren/Schwanz, `asset.extraBones`) werden in dasselbe Bone-Array angehängt (Index-Offset je Asset); Spring-Bone-Simulation läuft auf dem Core-Thread (kleine Ketten) und liefert Matrizen [D].
- **Rig-Fit bei Morph:** Joint-Positionen werden aus den Vertex-Würfeln des gemorphten Körpers abgeleitet (im Testcandidate umgesetzt); bei Parameteränderung nur betroffene Joints (Zonen) neu [D/A]. Weights bleiben (gleiche Topologie, T0).
- **Animation:** Sampling auf Core-Thread (VRMA-Kurven vorab in Arrays), Retargeting-Tabellen gecacht je (Body, Animation) [D].

## 6. Asset-System und Caches (Punkt 6, 7)
- **Index statt Scan:** Beim Start wird ein **Package-Index** (Manifest-Digests, Slots, Capabilities, Thumbnails-Refs) aus einer Cache-Datei geladen; Neu-Scan inkrementell per Datei-Zeitstempel/Digest (REQ-PERF-010) [D/A].
- **Binäre Runtime-Caches** (`.hasebin`, memory-mapped): Mesh, Sparse-Morphs, Skin, Fit-Daten; erzeugt beim Import/Installieren, versioniert mit Format- und Package-Digest [D; Motivation M: 464 ms + 851 ms Textladen].
- **Asynchrones Laden:** Priorisierte Job-Queue (sichtbar > Vorschau > Hintergrund), Abbruch bei Auswahlwechsel, Platzhalter-Proxy bis fertig; kein Frame > 50 ms durch Laden (REQ-PERF-015) [D].
- **Ressourcenmanagement:** Referenzzähler + LRU für CPU- und GPU-Ressourcen; Texturen mit Mipmaps, GPU-komprimiert (BC7/ASTC via Basis Universal/KTX2) für Vorschau; Quell-Texturen nur bei Export/Bake [D].
- **Speicherbudgets** (pro Kategorie, im Profiler sichtbar): Texturen, Meshes, Morph-Daten, Caches; bei Überschreitung Verdrängung per LRU, nie Absturz [D].
- **Bibliotheken:** UI-Listen sind **virtualisiert** (nur sichtbare Einträge) [M: ohne Virtualisierung 2,3 ms], Thumbnails lazy und gecacht (Atlas) [D].

## 7. Kleidung und Fitting (Punkt 5)
- **Fit als Funktion** `fit(asset, body_state_version, zone)`; Ergebnis gecacht unter Digest der relevanten Parameter (nur Parameter der Zonen, die das Asset überdeckt) [D].
- **Basismesh-gebundene Fits (barycentric)** sind rein vektorielle Rechnung: pro Vertex 3 Basis-Vertices + Offset – vollständig inkrementell und GPU-geeignet [D; Prototyp CPU in `build_makehuman_vrm.py`].
- **Nur betroffene Assets:** Dirty-Graph hängt Assets an Zonen; Gesichtsparameter fassen Kleidung nicht an (REQ-PERF-008) [D].
- **Teure Schritte** (Kollisionskorrektur, Weight-Transfer für Fremdtopologie) laufen asynchron; UI zeigt vorläufigen schnellen Fit und tauscht bei Fertigstellung (REQ-PERF-007) [D].
- **Abdeckungs-Culling** (verdeckte Körperflächen entfernen) wird einmal je Asset-Kombination berechnet und gecacht [D; Prototyp M: 2.338 Flächen].

## 8. Export und Validierung (Punkt 9)
- Eigener **Export-Job** auf Worker-Pool; arbeitet auf einem **Snapshot** des Zustands (unveränderlich), nicht auf dem Live-Zustand [D].
- Deterministische Reihenfolge, kein Zugriff auf Renderer/UI; UI zeigt Fortschritt/Abbruch. Im Spike: UI-Frame bei 2 Volllast-Threads unverändert (0,19→0,21 ms) [M].
- Validierung (Rig, Mesh, Material, VRM-Schema) ist Teil desselben Jobs; Ergebnisbericht kommt als Nachricht.

## 9. Skalierung (Punkt 8)
- Frame-Time hängt an **sichtbaren/aktiven** Objekten, nicht an installierten: Registry/Index lebt außerhalb des Frame-Pfads [D, REQ-PERF-017].
- Viele Packages: Manifeste lazy; Auflösung inkrementell; Digest-Cache (kein Re-Hash unveränderter Dateien) [D].
- Große Charaktere: GPU-Morph/Skinning für Skalierung; Instanzen teilen Morph- und Mesh-Daten, Zustand pro Instanz klein (Gewichte, Bones) [M: Pipeline mit `copies` getestet].

## 10. Profiling und Messbarkeit (Punkt 10)
- Eingebaut ab dem Walking Skeleton: Frame-Time (CPU/GPU, Timestamp-Queries), Phasenzeiten (Evaluator-Stufen, Upload, Passes), Zähler (Neuberechnungen je Stufe, Cache-Treffer), Speicher je Kategorie, Job-Queue-Tiefe; Overlay + JSON-Ausgabe [D; Timestamps im Spike lauffähig].
- Externe Werkzeuge: Tracy/`tracing` Spans für CPU, RenderDoc/PIX für GPU (Windows) [A].
- **Benchmark-Harness** `bench/morph-spike/` wird zum CI-Regressionstest (Warnschwelle +20 %, REQ-PERF-019).

## 11. Vereinbarkeit mit der Spezies-/Package-Architektur
- Performance-Regeln arbeiten auf der **Schnittstelle** des Body-Packages (Parameter, Slots, Zonen, Skeleton-Profil), nicht auf einem festen Charakter. Nicht-humanoide Archetypes bringen eigene Caches/Layouts mit; der Kern kennt nur Sparse-Deltas, Bone-Arrays, Slots.
- Packages liefern zusätzlich **Performance-Metadaten** (Vertex-/Tri-Zahlen, Textur-Größen, nnz der Morphs), damit der Editor Budgets *vor* dem Laden prüft (Budget-Warnung statt Absturz) [D; Schema-Erweiterung `x-perf` in K3].
- Das Package-Digest-Konzept (`package-system.md` §3) liefert die Cache-Schlüssel gratis.

## 12. Risiken
| Risiko | Gegenmaßnahme |
|---|---|
| Immediate-Mode-UI skaliert schlecht bei sehr großen Docking-/Tabellen-UIs | Virtualisierung als Regel, Messung je neuem Panel, Fallback Retained-UI nur für Spezialpanels [M/A] |
| GPU-Treiber-/Backend-Unterschiede (DX12/Vulkan) | Benchmark auf T-Min/T-Ref, Backend wählbar, Fallback-Pfad CPU-Morph [A] |
| Hohe Vertexzahlen (HD-Köpfe, Haarkarten) | Zonen-Updates, GPU-Compute-Reserve, LOD im Editor [A] |
| Speicher bei vielen Texturen | GPU-Kompression, LRU, Budgets [D] |
