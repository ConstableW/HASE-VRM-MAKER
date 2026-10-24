# WORK_LOG

Chronologisch, zusammengefasst, kein Chatverlauf.

## 2026-10-23 – Arbeitslauf 1: Konsolidierung und Erstrecherche
- Mandat (docx) in Repo übernommen (`docs/00-project/source/`), Repo-Bestand geprüft (nur `assets/`, Stub-README).
- Dokumentationsgerüst nach Mandat 7.1 angelegt, Ergänzungen in ADR-0001 begründet.
- Erstellt: `requirements.md`, `owner-decisions.md`, `vision.md`, `scope.md`, `constraints.md`, `glossary.md`, `functional-decomposition.md`, `risks.md`, `open-questions.md`, `roadmap.md`, `investigations.md`, `overview.md`, ADR-Register/-Vorlage/ADR-0001, `AGENTS.md`, `README.md`, Handoff-Dateien.
- Drei Recherche-Subagenten (Engine/UI; Formate/Geometrie; Referenzen/Packages/Recht) beauftragt; Berichte unverändert als `technology-evaluation.md`, `feasibility.md`, `references.md` abgelegt; Kernergebnisse in `CURRENT_STATE.md` und `overview.md` eingearbeitet.
- Befund: Animationspaket in `assets/` rechtlich ungeklärt (Mixamo-Herkunft, keine Lizenz) → OQ-03, R-11.
- Bewusst nicht getan: Commit, Push, Spikes, Code, Beschaffung.
- Git-Freigabe vom Owner erhalten (OD-019); Ergebnisse committet und gepusht.
- Push erfolgreich nach Wiederherstellung des GitHub-Zugriffs (Branch `claude/gifted-ptolemy-tdh9vc`).
- OQ-02 auf Owner-Delegation entschieden: ADR-0009 (Apache-2.0), OD-020.
- OQ-05 per Owner-Vorgabe entschieden: ADR-0010, OD-021; neue Fragen OQ-26/-27.
- Asset-Recherche (Subagent) → `docs/04-research/assets-candidates.md`. Ergebnis: MPFB2-Daten (CC0) per raw.githubusercontent.com erreichbar; MakeHuman-Packs (Haare/Augen/Kleidung) und Quaternius-Animationen nur nach Allowlist-Freigabe.
- Testcandidate vorbefüllt: `content/testcandidate/mpfb2-cc0/` (base.obj, 609 Targets, 2 Rigs, Lizenztexte, SHA-256-Manifest), Provenienzregister `content/PROVENANCE.md`, Build-Skript `tools/assets/build_testcandidate_body.py` (bpy headless) erzeugt Body-GLB (13.380 Vertices, 548 Shape Keys, 1,67 m; Artefakt 45 MB, nicht versioniert).
- OD-019/-024: Arbeit ab jetzt direkt auf `main`. Owner-Upload der MakeHuman-/Quaternius-Packs ausgewertet (`content/PROVENANCE_downloads.csv`: 158 CC0, 3 CC-BY, 14 wegen AGPL-Header gesperrt).
- Test-Avatar A: Quaternius Base Character + Haare → VRM 1.0 (`tools/assets/build_quaternius_vrm.py`; 52 Humanoid-Bones, Schema-Validierung 0 Fehler via `tools/validate/validate_vrm.py`, glTF-Validator 0 Fehler).
- 84 CC0-Animationen (Quaternius UAL1/UAL2) → VRMA (`build_quaternius_vrma.py`), alle glTF-Validator-fehlerfrei.
- Owner: „Tu was du für richtig hältst.“ → Mixamo-abgeleitete `assets/animation-*` und doppelte Quaternius-Ordner aus `assets/` entfernt (Git-Historie bleibt; ersetzt durch CC0-VRMA, OQ-03 erledigt).
- Test-Avatar B (MakeHuman-CC0-Pipeline, `tools/assets/build_makehuman_vrm.py`): Macro-Morph → Rig aus Gelenk-Würfeln des gemorphten Körpers → Gewichte → `.mhclo`-Proxys (Augen, Brauen, Wimpern, Haar, Shirt, Hose) per Barycentric-Bindung → VRM 1.0. Validierung: Schema 0 Fehler, glTF-Validator 0 Fehler; Render-Vorschau `content/testcandidate/previews/`.
- Skinning-Test: Avatar B + `UAL1/Sitting_Idle_Loop.vrma` (Frame 10) verformt sich sauber (`previews/avatar_b_sitting_idle_vrma.png`, `tools/assets/pose_test.py`).
- **Owner-Test (2026-10-23):** `avatar_a_quaternius_female.vrm` in HASEC geöffnet; Laufen, Tanzen und Springen mit den CC0-VRMA funktionieren (erster Nachweis in einem realen VRM-Player; RESEARCHED durch Owner-Aussage, ohne Screenshot).
- Avatar B v2: MToon-1.0-Materialien (Tönung für Shirt/Hose/Haare), Smooth Shading (Datei 39→19 MB), 12 VRM-Expressions (blink/blinkLeft/blinkRight, aa/ih/ou/ee/oh, happy/sad/relaxed/surprised) aus MakeHuman-Targets, auf Körper, Brauen und Wimpern angewendet (`previews/avatar_b_expressions.png`). Owner-Test in HASEC: Avatar B lädt, sitzende Animation korrekt. Bekannte Grenzen: Mund öffnet kaum (Basismesh ohne Mundraum), Blender-Vorschau zeigt MToon-Alpha nicht (Augen-Hornhaut ist in der Textur transparent, in Viewern korrekt).
- Avatar B v3: Owner-Screenshot (Tanz in HASEC) zeigte Haut-Durchschimmern an Schultern/Achseln. Fix: von Shirt/Hose verdeckte Körperflächen werden entfernt (2.338 Flächen, MakeHuman-Verfahren); Posen-Rendering ohne Durchschimmern (`previews/avatar_b_poses_dance_sit.png`). Owner-Urteil zur Optik: „PC-Spiel aus den 2000ern, so schlimm isses ned“ → akzeptabel als Platzhalter bis RC.

## 2026-10-24 – Species-/Package-System (Erstfassung)
- Architektur: `docs/02-architecture/species-system.md` (Ebenen, Topologie-Stufen T0/T1/T2, Entscheidungsregel Archetype), `package-system.md` (Typen, Identität/Digest, Auflösung, Sicherheit, SDK/Builder-Konzept).
- Spezifikationen: `docs/03-specifications/` – `package-manifest.md`, `species-schema.md`, `asset-schema.md`, `compatibility-rules.md` (C1…C9), `project-format.md`; normative JSON-Schemas `schemas/package.schema.json`, `schemas/character.schema.json`.
- Beispiele: 14 Packages (Body `humanoid-mh08` aus Testcandidate-Daten, Spezies Mensch/Elf/Katzen-Demi/Hunde-Demi, Assets Haar/Shirt/Hose + Stubs für Ohren/Schwanz/Schnauze) und `character.example.hase.json` mit Lock.
- Prototyp-Validator `tools/validate/validate_packages.py`: Schema + Kompatibilitätsregeln + Lock-Digests, 8 negative Selbsttests bestanden.
- ADR-0004, ADR-0005 (PROPOSED).

## 2026-10-24 – Performance als Architekturvorgabe (OD-025), Spike SP-08 Teil 1
- Zielwerte, Referenzhardware, Testbedingungen **vor** den Messungen festgelegt und committet (`docs/07-quality/performance-targets.md`, REQ-PERF-001…020).
- Rust-Benchmark `bench/morph-spike/` (wgpu 30.0.1, egui 0.37.0): reale Daten (13.380 Vertices, 610 Targets, 867k Deltas). CPU-Morphing, egui-UI-Kosten, wgpu-Pipeline (Compute-Morph, Skinning, Schatten, MSAA, Timestamps) gebaut und gelaufen.
- Ergebnisse `docs/04-research/benchmarks.md` + `bench/results/*.json`: CPU-Morph 64 Targets 0,14 ms, alle 610 1,8 ms, Einzelupdate 0,001 ms; Morph-Daten 13 MB (dicht 93 MB); UI virtualisiert 0,3 ms (nicht virtualisiert 2,4 ms); UI unter Hintergrundlast unverändert; **GPU-Zeiten im Container nicht aussagekräftig (SwiftShader)**.
- `docs/02-architecture/performance-architecture.md` (10 Bereiche), ADR-0002 (PROPOSED, Stack-Vergleich, Abnahmebedingung Windows-Lauf), R-17, OQ-31.
- Korrektur: Anzahl Morph-Targets ist **610**, nicht 609.
- Windows-Benchmark als fertige exe (`bench/win/morph-spike.exe`, 10 MB, Cross-Build mit cargo-zigbuild/Zig) + Doppelklick-Starter (`Benchmark-starten.bat`, DX12 und Vulkan) für den Owner; Backend-Wahl via `WGPU_BACKEND`; **nicht unter Windows getestet**.

## 2026-10-24 – Erster lauffähiger Editor
- Workspace `app/` (hase-core: OBJ/Morph/Parameter/Evaluator/Fit/Packages/Assets/State; hase-editor: wgpu-Renderer, Orbit-Kamera, egui-UI, Offscreen-Screenshot). Offscreen-Rendering via SwiftShader verifiziert (Morph-Varianten Geschlecht/Alter/Kopf/Kleidung plausibel). Windows-exe per cargo-zigbuild (`app/win/`). GUI-Interaktion im Container nicht testbar. Doku: `docs/02-architecture/geometry-morphing.md`.
