# Übersicht der notwendigen Architekturuntersuchungen

**Maßgebliche Quelle für den Untersuchungsplan** (Mandat 5, Bereiche 1–14, plus die frühen Risikothemen aus Mandat 11). Ergebnisse stehen in den verlinkten Dokumenten; Entscheidungen als ADR.

| ID | Untersuchung | Mandat 5 | Risiken | Ergebnisdokument | ADR | Status |
|---|---|---|---|---|---|---|
| AI-01 | Native Desktop vs. engine-basiert vs. Web-Hybrid | 1 | R-10 | `../04-research/technology-evaluation.md` | ADR-0002 | Recherche vorhanden, Entscheidung offen |
| AI-02 | Rust / C++ / Hybrid | 2 | R-10 | ebd. | ADR-0002 | wie AI-01 |
| AI-03 | Rendering-Engines, GPU-Abstraktion, MToon-Machbarkeit | 3 | R-10, R-12 | ebd. | ADR-0002 | wie AI-01 |
| AI-04 | Parametrische Geometrie: Morph-Räume, Topologie-Familien, Archetype-Kriterien | 4 | R-01 | `geometry-morphing.md`, `species-system.md` | ADR-0004 | offen (K2), Spike SP-01 |
| AI-05 | Morph Targets, Skinning, Deformation, Korrekturen, Direktmanipulation | 5 | R-04, R-05 | `geometry-morphing.md`, `rigging-animation.md` | ADR-0004/-0006 | offen (K2/K4), SP-03/-04 |
| AI-06 | Asset-/Ressourcenverwaltung, Provenienz, Import-Pipeline | 6 | R-11 | `asset-pipeline.md` | ADR-0007 | offen (K4) |
| AI-07 | Plugin- und Package-System, Sandboxing, Versionierung | 7 | R-02, R-09 | `package-system.md`, `../03-specifications/package-manifest.md` | ADR-0005 | offen (K2/K3), SP-06 |
| AI-08 | UI-Frameworks und Viewport-Integration, Barrierefreiheit | 8 | R-10, R-15 | `technology-evaluation.md` | ADR-0002 | Recherche vorhanden, offen |
| AI-09 | Blender als optionales Backend | 9 | R-16 | `../04-research/feasibility.md` | ADR-0008 | Recherche läuft, SP-09 |
| AI-10 | VRM-/glTF-/Animationsbibliotheken, FBX | 10 | R-07, R-08 | `../04-research/feasibility.md` | ADR-0003 | Recherche läuft |
| AI-11 | Tests und visuelle Regressionstests | 11 | R-08 | `../07-quality/test-strategy.md` | – | offen (K6) |
| AI-12 | Performance, Speicher, Skalierbarkeit | 12 | R-12 | `../04-research/benchmarks.md` | – | offen, Zielwerte OQ-09 |
| AI-13 | Plattformstrategie Windows primär | 13 | – | `technology-evaluation.md` | ADR-0002 | in AI-01 enthalten |
| AI-14 | Lizenz- und Distributionsprüfung aller Abhängigkeiten | 14 | R-11 | `../04-research/references.md`, Lizenzregister | – | blockiert durch OQ-02 |
| AI-15 | Kleidungs-Fitting über Körper/Spezies | 11 (früh) | R-03 | `geometry-morphing.md` | ADR-0006 | offen, SP-02 |
| AI-16 | Rigging/Retargeting über Anatomien, VRM-Grenzen für Nicht-Humanoide | 11 (früh) | R-06, R-07 | `rigging-animation.md`, `export-pipeline.md` | ADR-0006 | offen, SP-05/-07 |
| AI-17 | Reproduzierbarer VRM-Export | 11 (früh) | R-08 | `export-pipeline.md` | ADR-0003 | offen, SP-07 |

ADR-Nummern sind reserviert (siehe `../05-decisions/README.md`); Inhalte entstehen erst mit der jeweiligen Untersuchung.
