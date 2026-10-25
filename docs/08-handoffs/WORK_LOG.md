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
- UI 2.0 Schritt 1: Arbeitsbereiche Charakter/Outfit/Finalisieren, Bereichsliste mit Kamerafokus, Regionen-Regler, Standardcharakter unbekleidet. Echte GUI headless getestet (Xvfb + Lavapipe, simulierte Klicks). `docs/01-product/ux-concept.md`.
- Designsystem (theme.rs, deutsche Parameternamen), Preset-Leiste. Körper-&-Spezies-Studio Erstfassung (studio.rs, validate.rs, Paket-Schreiben/Laden unter user-content/), OD-026/-027, REQ-STUDIO-001…012.
- Owner-Korrekturen: Navigation (Werkzeuge getrennt), Versionsbindung mit Inhalts-Hash und Lade-Dialog, Speziesgrenzen wirksam (Tests), Beispiel-Parameter-IDs migriert + Lock, glTF-Analyse und Körperbasis-Entwurf. Offen/ungetestet: echter Windows-GUI-Test.
- Körperbasis-Aufbereitung: allgemeiner glTF-Body-Loader, Picking/Landmarken/Regionen/Bone-Mapping in der GUI, Inhalts-Fingerprint der Ressourcen, Warntext bei Inhaltsänderung. Ketten-Test an 548-Morph-Körper bestanden; Auto-Regionen ca. 64 % an MakeHuman. Weiterhin ungetestet: echter Windows-GUI-Lauf.
- Zweite Körperbasis (Quaternius + synthetische Shape Keys) als Testfixture; Kette mit zwei Basen per Test; Fix: Symmetrie-Erkennung ("_l" in "_longer"); Spezies pro Körperbasis, Charakter ohne Spezies, Körperbasis-Wechsel beim Laden mit Rückfrage. Fitting auf fremde Topologien zurückgestellt (Owner).
- Produktphase (nach Owner-Windows-Test): Render-Materialsystem, Thumbnail-Worker, Augenstudio, anatomisches Brust-/Gesäßmodell, Haarform/-farbe, Haut-Make-up/Details (Masken aus Morph-Fußabdrücken), Charakterbibliothek, Preset-Leiste mit Thumbnails, Viewport-Werkzeuge. Messwerte: Hautmasken 1,9 ms, Komposition 12 ms. Doku: character-studios.md.
- Finalisieren: natives VRM-1.0-Export (Rig, Gewichte, MToon, Expressions, eingebackene Augentextur) + Posen-/Animationsvorschau (VRMA-Retargeting, CPU-Skinning 1,4 ms/Frame). Validierung: VRM-Schema 0 Fehler, glTF-Validator 0 Fehler, Blender-Import + Tanz-VRMA. Gefundene und behobene Fehler: negative Skin-Gewichte, nicht normierte Normalen, Ruhepose-Rücksetzen.

## 2026-10-24 – Korrekturrunde nach Owner-Test
Posen (A→T-Kompensation), Kleidungskollision, Lippenmaske, Header Speichern/Exportieren, statisches VRM-Öffnen (Lylia), Analyse nach VRM-Humanoid-Map. Windows-exe neu gebaut. Offen: Windows-Test, Cloth-Performance, Spring Bones.

## 2026-10-24 – Anprobe auf importierten VRMs
`dress.rs` (Registrierung, Gewichtsübernahme), Session-Integration (Outfit-Workspace bei geöffnetem Modell, Teileliste, Posen mit Kleidung, Export), VRM-Teile kompaktiert. Test: `test.vrm` (VRoid-Basis) mit Shirt + Shorts.

## 2026-10-24 – UI-Meilenstein nach verbindlichem Mockup
Neues Hauptlayout (Kopfzeile, Navigation links mit Icons, Viewport-Overlays, Akkordeon rechts, Presetleiste, Bibliothek als Panel, Outfit-Kachelraster), 11 Presets für „Mensch“. Details: ux-concept.md.

## 2026-10-25 – Spezieskörper: Einbacken vs. Variation
`packages::SpeciesShape` (baked + Variationen, Sidecar `shape.json`, Hash im Manifest), `Session::{bake_into_species, define_variation, set_variation, refresh_species_layer}`, Studio-Abschnitt „Spezieskörper aus der Formung“ (Tab Körperbasis, nur eigene Spezies), Variationsregler in der Spezieskarte der Charakter-Phase, Persistenz in Charakterdatei (`variations`). Test `species_bake_and_variation`. Nicht per Hand in der Windows-GUI geprüft; Exe nicht neu gebaut.

Windows-Build-Rezept (Container ohne Vorinstallation): `apt-get install gcc-mingw-w64-x86-64` (posix-Variante per update-alternatives), dann `cargo build --release --target x86_64-pc-windows-gnu` in `app/`, Ergebnis nach `app/win/hase-editor.exe` kopieren. Exe-Stand: a787f71+ (Spezieskörper, Basiswechsel, L/R-Variationen).

## 2026-10-25 – Spezies-Kacheln, Standardmerkmale
Speziesauswahl als Kachelraster; Spezies mit Komponenten (Katzen-/Hunde-Demi, Elf) leiten erlaubte Merkmale und Standardmerkmale (Ohren/Schweif mit passendem Preset) aus ihren Komponenten ab und aktivieren sie bei Auswahl. Abgeleitete Spezies übernehmen Formung/Variationen. Tests `species_default_features`, `species_bake_and_variation`.

## 2026-10-25 – Eigene Geometrie der Beispiel-Spezies
Beispiel-Spezies können `shape.json` im Paketordner tragen (Digest = Manifest + Shape). Hunde-/Katzen-Demi haben eine prozedural erzeugte Schnauze eingebacken und den Regler „Schnauzenlänge“ (`cargo test --release generate_species_shapes -- --ignored` erzeugt die Dateien; KI-generiert, keine fremden Meshes). Screenshot `docs/07-quality/screens/dog-demi-muzzle.png`. Noch rudimentär (nur Schnauze); Elf/weitere Spezies folgen.

## 2026-10-25 – Elf-Geometrie, Spezies teilen
Elf: Kinnspitze (eingebacken + Regler), Preset-Index-Korrektur (Elf = 4), seitliche Elfenohren. Spezies-Export/-Import als `.hasespecies` (Manifest + Spezieskörper; Import vergibt neue ID bei Kollision) im Spezies-Studio; Test `user_species_roundtrip` deckt Export→Import ab. Nicht teilbar sind bisher die referenzierten Assets/Körperbasen (nur IDs).

## 2026-10-25 – Weitere Beispiel-Spezies
Orkin (breiter Kiefer, Brauenwülste eingebacken, Regler Kieferbreite, Hörner/Ohren erlaubt) und Zwerg (große breite Nase, Regler Nasengröße), prozedural per `generate_species_shapes` erzeugt (Manifest + shape.json). Effekte bewusst moderat; Feinschliff erst mit echten Körperbasen je Spezies.

## 2026-10-25 – Körperproportionen je Spezies
Orkin (breiter Rumpf/Schultern, kräftige Arme), Zwerg (kurze Beine, breiter Rumpf), Elf (schlank, lange Beine) über Standardwerte der Spezies; Vergleich `docs/07-quality/screens/species-bodies.png`.

## 2026-10-25 – Frisuren-Generator (Merkmal „Frisur (generiert)“)
Neues Merkmal `hair` (features.rs): Strähnenbänder, auf der Schädelellipse verwurzelt, mit Schwerkraft, Welle, Scheitel, Pony-/Seiten-/Hinterkopflänge, Volumen, Dichte; Kollision mit Kopf und grober Schulterellipse; 5 Presets. Arbeitet auf MakeHuman-Basis und Avataren, wird mit exportiert (Kopf-Bone, noch ohne Spring-Bones). Grenzen: Strähnen sind flache Bänder (keine Haarkarten-Texturen), Schulterkollision grob (sichtbare Kante), keine Physik. Screenshot `hair-generated.png`.
Frisur: langes Haar (> 1,2 Kopfeinheiten) bekommt je Azimut-Cluster (12) eine Spring-Bone-Kette aus drei Bones (`J_Sec_Hair_xx_yy`), Gewichte entlang der Strähne (Ansatz am Kopf-Bone); VRM-Export schreibt `VRMC_springBone`, Schema 0 Fehler. Schwingungsverhalten in einem VRM-Viewer noch nicht geprüft (Owner/HASEC).

## 2026-10-25 – Hautton, Spezies-Kacheln mit Körper
Neuer Hautton (Tönung + Stärke) im Style (`SkinStyle.tone/tone_amt`, im Charakter gespeichert, wirkt in Textur und Export); Spezies können einen Standardton vorgeben (`shape.skinTone`: Orkin grünlich, Elf leicht bläulich); Spezieskacheln zeigen jetzt Körperform, Merkmale und Ton. Charakter → „Als neue Spezies speichern“ übernimmt den Ton.

## 2026-10-25 – Exportformate
Export-Panel mit Format-Auswahl: VRM 1.0, schlichtes glTF/GLB (ohne VRMC-/Unlit-Erweiterungen, mit Skelett/Skinning/Morphs) und Wavefront OBJ (+MTL, PNG; statische Ruhepose). Test deckt glTF ohne VRMC und OBJ ab. Offen: FBX, USD, Ziel-Presets (z. B. VRChat/Unity), Blender-Prüfung des schlichten GLB.

Frisur: Zopf/Pferdeschwanz (Parameter „Zusammenbinden“, „Zopf-Höhe“, Preset), Schulterkollision ohne Kante (Ellipse erst unterhalb des Halses, Nackenwurzeln ausgelassen). Screenshot hair-ponytail.png.

## 2026-10-25 – Outfit-Vorlagen
Outfit-Studio: „Meine Outfits“ – getragene Kleidung samt Farben als benannte Vorlage speichern (user-content/outfits.json), anwenden (lädt Teile, setzt Farben, legt andere Kleidung ab) und löschen. Test outfit_roundtrip.

Studio: Auswahl einer Spezies zeigt sie im Viewport; Tab Körperbasis enthält dieselben Werkzeuge wie die Charakter-Phase (Pinsel, Merkmale inkl. Frisur) – eine Toolbox; fehlende Glyphen (→) durch ASCII ersetzt.
