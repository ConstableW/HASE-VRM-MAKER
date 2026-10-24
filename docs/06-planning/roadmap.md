# Plan: Konzeptphase und spätere Umsetzung

**Maßgebliche Quelle für Planung.** Status: `PROPOSED`. Umsetzungsphasen sind erst nach Owner-Abnahme und gesonderter Beauftragung wirksam (REQ-GOV-001).

## 1. Plan der Konzeptausarbeitung

| Schritt | Inhalt | Ergebnisdokumente | Status |
|---|---|---|---|
| K0 | Mandat konsolidieren; Anforderungsmatrix, Owner-Entscheidungen, Zerlegung, Risiken, Untersuchungsübersicht, offene Fragen, Plan; Handoff-Struktur | `00-project/*`, `06-planning/*`, `08-handoffs/*` | **erledigt (Erstfassung)** |
| K1 | Recherche: Technologien, Formate/VRM, Geometrieverfahren, Referenzprodukte, Package-Vorbilder; Quellen mit Versionsstand | `04-research/*` | in Arbeit (3 Recherchestränge delegiert) |
| K2 | Kernarchitektur: Overview, Character Core, Species-System, Package-System, Geometry/Morphing | `02-architecture/*`, ADR-0003…0006 | offen |
| K3 | Spezifikationen: Projektformat, Package-Manifest, Species-/Asset-Schema, Kompatibilitätsregeln; Beispielpakete | `03-specifications/*`, `examples/species/*` | offen |
| K4 | Rigging/Animation, Rendering, Asset- und Export-Pipeline | `02-architecture/*` | offen |
| K5 | Produkt: Feature-Katalog mit Abhängigkeiten/Akzeptanzkriterien, User Journeys, UX-Konzept, Screen-Entwürfe | `01-product/*` | offen |
| K6 | Qualität: Qualitätsattribute, Teststrategie, Validierungsstrategie | `07-quality/*` | offen |
| K7 | Technologieentscheidung als ADR mit Alternativen + Empfehlung; Spike-Spezifikationen finalisieren; Abhängigkeits-/Meilensteinplan | ADR-0002, `06-planning/*` | offen |
| K8 | Konsistenzreview (Traceability REQ ↔ Komponente ↔ ADR ↔ Test), Owner-Entscheidungsvorlage, Abnahmepaket | alle | offen |

Reihenfolgeprinzip: Risiko zuerst (R-03, R-01, R-05, R-13), Entscheidungen mit hoher Rückbaukosten (Stack, Datenmodell, Package-Format) vor Detailarbeit.

## 2. Technische Spikes (nicht freigegeben – OQ-04)

Jeder Spike: zeitlich begrenzt, Wegwerf-Code, Ergebnisbericht in `04-research/benchmarks.md`. Erfolgskriterien sind **Vorschläge** (`PROPOSED`) und müssen vor Freigabe vom Owner bestätigt werden.

| ID | Frage | Risiko | Erfolgskriterium (messbar) |
|---|---|---|---|
| SP-01 | Tragen zwei Körperbasen (z. B. Mensch + Katzen-Demi) eine gemeinsame Morph-Parametrisierung? | R-01 | ≥ 80 % der gemeinsamen Parameter erzeugen auf beiden Körpern visuell/anatomisch korrespondierende Deformation (Landmarken-Abweichung < definierte Toleranz); Liste nicht übertragbarer Parameter dokumentiert |
| SP-02 | Kleidungs-Fitting an Morph-Extremen und zweiter Spezies | R-03 | Bei 100 Zufallskombinationen und 10 Extremfällen: Durchdringungsfläche < 0,5 % der Kleidungsfläche nach automatischem Fit; Fit-Zeit < 500 ms pro Teil (Zielhardware OQ-09) |
| SP-03 | Direkte Manipulation → Parameter | R-04 | Drag auf Mesh-Handle liefert in < 16 ms Parameterupdate; Rückprojektion Abweichung < 2 mm Zielpunkt bei erreichbaren Zielen; kein Parameterdrift nach 20 Drag-Zyklen |
| SP-04 | Expert-Edit (Sculpt-Delta) bei späterer Parameteränderung | R-05 | Nach Parameteränderung bleibt Delta-Layer plausibel (keine Selbstdurchdringung) in ≥ 95 % von 50 Testfällen; Invalidierungsregel testbar |
| SP-05 | Rig-Fitting + Retargeting über Proportionen/Digitigrade | R-06 | VRMA-Animationen aus `assets/` (Lizenz beachten) laufen auf 3 Körpern ohne sichtbare Fußgleiter > 2 cm und ohne Gelenkflips |
| SP-06 | Package-Resolver + Kompatibilität | R-02, R-09 | 1000 synthetische Packages: Auflösung < 1 s; Konflikte werden deterministisch gemeldet; manipulierte Hashes werden zu 100 % erkannt |
| SP-07 | Deterministischer VRM-1.0-Export + Validierung in Referenz-Loadern | R-07, R-08 | Zwei Exporte desselben Projekts auf zwei Maschinen sind byte-identisch; Laden in ≥ 3 Referenz-Viewern fehlerfrei; Verlustreport vollständig |
| SP-08 **(Teil 1 erledigt 2026-10-24: `bench/morph-spike`, siehe `../04-research/benchmarks.md`; Teil 2 = Lauf auf Windows-GPU offen)** | Walking Skeleton des Kandidaten-Stacks (Viewport, Property-UI, 100 Morphs auf 60k-Tri-Mesh, Skinning, Docking-UI, Accessibility-Grundtest) | R-10, R-12, R-15 | ≥ 60 fps bei Zielhardware; Morph-Update < 5 ms; UI-Tastaturbedienung und Screenreader-Basisfunktion nachweisbar; Lizenzcheck bestanden |
| SP-09 | Blender als optionales Backend (headless) | R-16 | Batch-Import/Konvertierung eines Test-Assets automatisiert, ohne gebündelte Blender-Binaries |

## 3. Grobe Umsetzungsphasen (Hypothese, nach Abnahme zu detaillieren)

| Phase | Ziel | Hauptinhalt |
|---|---|---|
| I0 | Walking Skeleton | Core-Zustand, Undo, Projektformat v0, Viewport, Package-Loader |
| I1 | Humanoid-Vertical-Slice | Body-Package + 2 Spezies über Package-Pfad, Face/Body-Morphs, VRM-Export mit Report |
| I2 | Fitting & Materialien | Kleidung, Haare (Basis), Material-Layer, Spring Bones, Animationstest |
| I3 | Designer-UX vollständig | Wizard, Bibliotheken, Variantenvergleich, Presets |
| I4 | Package SDK/Builder | CLI, Blender-Add-on, visueller Builder |
| I5 | Expert-Modus | Sculpt, Weight Painting, UV, Physik-Editor |
| I6 | Erweiterte Anatomien/Simulation | Zusatzgliedmaßen-Archetype, Cloth, Strand-Haar |

Meilensteine mit messbaren Abnahmekriterien werden in K7 aus den Spike-Ergebnissen abgeleitet.
