# CURRENT_STATE

*Zuletzt aktualisiert: 2026-10-25*

## Projektphase
Discovery & Architecture (Konzept) **plus** vom Owner freigegebene Asset-/Testcandidate-Aufbereitung (OD-023, OD-024). **Erste lauffähige Editor-Version vorhanden** (`app/`, Rust/wgpu/egui: 3D-Viewport, Orbit-Kamera, ~294 Morph-Regler, Spezies/Presets, Haare/Kleidung, Haut/Augenfarbe, Undo/Redo, Save/Load, F3-Profiling). UI 2.0 begonnen (3 Arbeitsbereiche, Bereichsnavigation, siehe `docs/01-product/ux-concept.md`). Windows-exe `app/win/` (Start: `Starten.bat`) **nicht auf echter Windows-GPU getestet**. Körper-&-Spezies-Studio (Erstfassung: `docs/02-architecture/body-species-studio.md`). Kette Import→Körperbasis→Spezies→Charakter→Speichern/Laden mit **zwei** Körperbasen per Test belegt (siehe body-species-studio.md); Fitting auf fremde Topologien bewusst zurückgestellt. Studios (Augen, Haar, Haut/Make-up/Details, anatomische Brust/Gesäß-Morphs), Charakterbibliothek, Thumbnail-Dienst, Viewport-Werkzeuge: siehe `docs/02-architecture/character-studios.md`. Finalisieren: VRM-1.0-Export und Posen-/Animationsvorschau (Erstfassung). Nicht enthalten: Export importierter Körperbasen, Spring Bones/Haarphysik, VRM-Export aus dem Editor. Alle Arbeit direkt auf `main` (kein Feature-Branch).

## Verifizierter Istzustand (alles auf `main`)
- **Konzeptdokumente:** `docs/` vollständig gerüstet; ausgearbeitet: Anforderungen (146 IDs), Owner-Entscheidungen OD-001…024, Zerlegung, Risiken, offene Fragen, Roadmap, Architekturüberblick, drei Rechercheberichte + Asset-Recherche, ADR-0001/-0004/-0005/-0009/-0010; **Species-/Package-System-Erstfassung** (Architektur, Schemas, Beispiele, Validator, 2026-10-24).
- **Testcandidate (`content/testcandidate/`):**
  - `avatar_a_quaternius_female.vrm` (Quaternius-Basischarakter + Haare, VRM 1.0)
  - `avatar_b_makehuman.vrm` (MakeHuman-CC0-Pipeline: Macro-Morph, Rig aus Gelenk-Würfeln, Gewichte, 6 Proxys; 73k Tris)
  - `animations/UAL1`, `UAL2`: 84 CC0-Animationen als VRMA
  - Prüfungen: VRM-Schema 0 Fehler, glTF-Validator 0 Fehler, Skinning-Render-Test mit VRMA (`previews/`); **Owner-Test in HASEC: Avatar A lädt, Laufen/Tanzen/Springen mit den CC0-VRMA funktionieren** (Avatar B noch ungetestet)
  - Herkunft: `content/PROVENANCE.md`, `content/PROVENANCE_downloads.csv` (158 CC0, 3 CC-BY, 14 wegen AGPL-Header gesperrt)
- **Werkzeuge (`tools/`):** bpy-Build-Skripte (`assets/`), VRM-Schema-Validator (`validate/`). Benötigen PyPI `bpy` 5.2.3 und `bpy-vrm-format` 4.10.2 (nur lokal, GPL/MIT-or-GPL, nicht Teil des Produkts), Python 3.13.
- **Entfernt:** Mixamo-abgeleitete `assets/animation-*` und Duplikate (Historie bleibt).
- **Repo-Größe:** Klon ca. 770 MB (Owner-Upload der Rohpacks in `content/testcandidate/downloads/` inkl. Historie). Kein History-Rewrite vorgenommen.

## Editor-Stand (2026-10-25, `app/`, Details in WORK_LOG)
Ein Editor, drei Workspaces (Charakter/Outfit/Finalisieren) nach Mockup. **Körperbasis → Speziesmodell → Charaktere:** Speziespakete tragen Standardwerte, Grenzen, Presets, erlaubte/Standard-Merkmale, Hautton und einen Spezieskörper (`shape.json`: eingebackene Grundform + benannte Variationen mit Bereich, optional links/rechts). Studio: Pinsel-Formung einbacken oder als Variation definieren, Spezies ableiten/duplizieren/exportieren/importieren (`.hasespecies`). Charakter-Phase: Speziesauswahl als Kacheln (wechselt bei Bedarf die Körperbasis), nur Variationen der Spezies als Zusatzregler. Beispiel-Spezies: Humanoid, Elf, Orkin, Zwerg, Katzen-/Hunde-Demi (Geometrie prozedural erzeugt). Merkmale: Ohren, Schweif, Hörner, **Frisur (generiert, Strähnen, Spring Bones)**. Augen links/rechts getrennt. Export: VRM 1.0, glTF/GLB, OBJ. Importierte VRMs: Anprobe, parametrische Avatar-Regler, Formung, Merkmale.
**Grenzen:** nur eine echte Körperbasis (MakeHuman hm08); Spezies-Geometrie ist moderat (Gesicht/Proportionen); Windows-GUI-Test steht aus (alles headless unter Linux/Lavapipe geprüft); Frisur-Physik im Viewer nicht geprüft.

## Performance (OD-025, 2026-10-24)
Zielwerte vorab: `docs/07-quality/performance-targets.md`. Spike-Teil 1 gemessen (`docs/04-research/benchmarks.md`): CPU-Morph/Speicher/UI weit innerhalb der Ziele; **GPU-Frame-Times offen** – Windows-Lauf nötig (NEXT_ACTIONS 14). ADR-0002 (Stack) PROPOSED, nicht freigabefähig bis dahin. Architekturregeln: `docs/02-architecture/performance-architecture.md`.

## Umgebung / Randbedingungen
Container ohne GPU und ohne Windows. Netzwerk-Allowlist vom Owner erweitert (MakeHuman, Quaternius); `github.com`-Clone bleibt gesperrt, `raw.githubusercontent.com` und PyPI funktionieren (C-10).

## Offene Punkte am Testcandidate
MToon-Materialien (aktuell Standard-PBR), Expression-Blendshapes, Polygonreduktion (Brauen/Wimpern), echter Viewer-Test von Avatar B (Owner; A bestanden), Shirt/Hose-Texturqualität, gesperrte AGPL-Assets nicht verwendet.

## Offene Owner-Entscheidungen
`docs/06-planning/open-questions.md`: OQ-01 (Stack, nach Konzeptarbeit), OQ-04 (Spikes SP-01…09 einzeln), OQ-06…10. Geschlossen heute: OQ-02, -03, -05, -11, -26, -27.

## Recherchequalität
Berichte stammen von Subagenten und wurden nur gelesen/integriert; [R]/[A]-Kennzeichnung beachten. Einzelne Primärquellen waren nicht abrufbar.

## Risiken
`docs/06-planning/risks.md` (Top: R-03 Kleidungsfitting, R-01 speziesübergreifende Geometrie, R-05 Nichtdestruktivität, R-13 Content-Aufwand). Hinweis: R-13 ist durch die CC0-Basis deutlich entschärft.

## Nächste Schritte
`NEXT_ACTIONS.md`.
