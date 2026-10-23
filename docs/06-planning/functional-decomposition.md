# Funktionale Zerlegung (Erstfassung)

**Maßgebliche Quelle für die Komponentenliste und ihre Anforderungsabdeckung.** Status: `PROPOSED`. Die Zerlegung ist technologieneutral; Technologiewahl siehe `../04-research/technology-evaluation.md`. Detaillierung folgt in `../02-architecture/overview.md`.

Leitprinzip: Datenmodell (Projektzustand, Packages) und Verarbeitung (Evaluation, Export) sind von UI und Rendering getrennt, damit Designer-/Expert-Modus, Headless-Export (CLI/CI) und Package-SDK denselben Core nutzen (REQ-ARCH-008, REQ-EXP-007).

| ID | Komponente | Verantwortung | Wichtige Abhängigkeiten | Abdeckt |
|---|---|---|---|---|
| F-01 | **Character Core** | Projektzustand (Character = Archetype-Ref + Species-Ref + Parameter + Asset-Slots + Overrides); Befehlsmodell, Undo/Redo, Änderungsverlauf, Events | – | ARCH-001/-008/-009, UX-009 |
| F-02 | **Project Format & Persistenz** | Natives, versioniertes, migrierbares Projektformat; Autosave, Recovery | F-01, F-03 | ARCH-009, UX-015 |
| F-03 | **Package Manager** | Installation, Auflösung, Versionen, Lockfile, Integrität, Kompatibilitätsprüfung, Lizenzregister | F-04 | PKG-001…013 |
| F-04 | **Schema & Validation** | Schemas für Body/Species/Asset-Manifeste; Validatoren; Fehlerberichte | – | PKG-005, EXP-005 |
| F-05 | **Species/Archetype Registry** | Laden von Body-/Species-Definitionen als Daten; Capability- und Slot-Modell | F-03, F-04 | ARCH-001…007 |
| F-06 | **Geometry & Morph Engine** | Basis-Mesh, Morph-Targets/Delta-Layer, Zonen, Constraints, Korrektur extremer Kombinationen; Evaluation Parameter → Mesh | F-01, F-05 | FACE-*, BODY-*, ADV-004/-010 |
| F-07 | **Manipulation Solver** | Direkte 3D-Manipulation → Parameter (inverses Problem), Symmetrie-Handling | F-06 | FACE-012, UX-005 |
| F-08 | **Rig & Skinning Engine** | Skeleton-Profile, Bone-Mapping, Joint-Fitting an Morphs, Skinning, Corrective Shapes, Rig-Validierung | F-05, F-06 | RIG-*, BODY-008 |
| F-09 | **Fitting Engine (Kleidung/Haar/Accessoires)** | Anpassung an Körper, Weight Transfer, Kollisionsprüfung, Layering, Attachments | F-06, F-08 | CLO-*, HAIR-* |
| F-10 | **Material & Texture System** | Material-Layer, Zonen, Muster, Shader-Abstraktion (MToon-Mapping), Baking | F-01 | MAT-* |
| F-11 | **Simulation** | Spring Bones, Haarphysik, optional Cloth, Kollider | F-08 | ANI-007, HAIR-006, CLO-010 |
| F-12 | **Animation System** | Playback, Posing, Expressions, LookAt, Viseme, Retargeting, VRMA-Import/-Export, Posenbibliothek | F-08 | ANI-*, RIG-006 |
| F-13 | **Render/Viewport** | Echtzeit-Darstellung, Kamera/Licht-Presets, Mehrfachansichten, Picking/Gizmos | F-06, F-10 | UX-010/-011, PROD-002 |
| F-14 | **Export Pipeline** | Kanonisches Intermediate → VRM 1.0 / glTF / FBX; Optimierung, LOD, Budgets, Export-Report mit Verlusten, Determinismus | F-06…F-12 | EXP-* |
| F-15 | **Import Pipeline** | glTF/GLB/VRM/FBX/BVH/VRMA, Texturen, Bone-Mapping-Assistent, Provenienz | F-04 | PKG-007, ANI-009/-010, HAIR-007 |
| F-16 | **Validation & Diagnostics** | Zentrale Prüfregeln (Rig, Mesh, Material, VRM, Budgets), Schweregrade, Auto-Fix-Vorschläge | F-04 | BODY-010, RIG-009, EXP-005 |
| F-17 | **UI Shell – Designer** | Wizard, Kategorie-/Preset-Navigation, Bibliotheken, Variantenvergleich | F-01, F-13 | UX-001…012 |
| F-18 | **UI Shell – Expert** | Mesh/Sculpt/Morph/UV/Weight/Physics-Editoren, Material-Editor | F-01, F-13 | ADV-*, UX-013 |
| F-19 | **Package SDK & CLI** | Bibliothek, CLI (validate/build/pack/inspect), Vorlagen, Doku, Blender-Add-on | F-03, F-04 | PKG-007/-008 |
| F-20 | **Package Builder (visuell)** | Geführter Editor für Species-/Asset-Packages inkl. Prüfung und Testvorschau | F-19, F-17/18 | PKG-009 |
| F-21 | **Extension Host** | Plugins/Skripte unter Sandbox, Berechtigungsmodell | F-03 | ARCH-010, PKG-013 |
| F-22 | **Blender Bridge (optional)** | Externer Prozess für Import/Konvertierung/Automation | F-15 | PKG-007 |
| F-23 | **Test & Reproducibility Harness** | Headless-Rendering, Referenzbilder, Golden-Exports, Benchmarks | alle | QA-* |
| F-24 | **Accessibility & Input** | Tastatur, Controller, Screenreader, Skalierung, Eingabegeräte | F-17/18 | UX-014 |

## Datenfluss (grob)

```
Package Manager ─► Registry ─► Character Core (Zustand)
                                    │ Befehle / Undo
                   ┌────────────────┼───────────────────┐
                   ▼                ▼                   ▼
            Geometry/Morph ─► Rig/Skinning ─► Fitting ─► Simulation
                   └──────────────┬─────────────────────┘
                                  ▼
                   Evaluiertes Character-Intermediate
                      │                       │
                      ▼                       ▼
                Render/Viewport        Export-Pipeline ─► VRM/glTF/FBX + Report
```

Kernidee (`PROPOSED`): Der **Character Core** hält nur Parameter und Referenzen; das evaluierte Mesh ist abgeleitet. Expert-Edits werden als Delta-Layer im selben Zustand gespeichert (R-05).
