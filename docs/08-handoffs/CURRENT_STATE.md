# CURRENT_STATE

*Zuletzt aktualisiert: 2026-10-23 (Arbeitslauf 1, spät)*

## Projektphase
Discovery & Architecture (Konzept) **plus** vom Owner freigegebene Asset-/Testcandidate-Aufbereitung (OD-023, OD-024). Produktcode: keiner. Alle Arbeit direkt auf `main` (kein Feature-Branch).

## Verifizierter Istzustand (alles auf `main`)
- **Konzeptdokumente:** `docs/` vollständig gerüstet; ausgearbeitet: Anforderungen (146 IDs), Owner-Entscheidungen OD-001…024, Zerlegung, Risiken, offene Fragen, Roadmap, Architekturüberblick, drei Rechercheberichte + Asset-Recherche, ADR-0001/-0009/-0010.
- **Testcandidate (`content/testcandidate/`):**
  - `avatar_a_quaternius_female.vrm` (Quaternius-Basischarakter + Haare, VRM 1.0)
  - `avatar_b_makehuman.vrm` (MakeHuman-CC0-Pipeline: Macro-Morph, Rig aus Gelenk-Würfeln, Gewichte, 6 Proxys; 73k Tris)
  - `animations/UAL1`, `UAL2`: 84 CC0-Animationen als VRMA
  - Prüfungen: VRM-Schema 0 Fehler, glTF-Validator 0 Fehler, Skinning-Render-Test mit VRMA (`previews/`)
  - Herkunft: `content/PROVENANCE.md`, `content/PROVENANCE_downloads.csv` (158 CC0, 3 CC-BY, 14 wegen AGPL-Header gesperrt)
- **Werkzeuge (`tools/`):** bpy-Build-Skripte (`assets/`), VRM-Schema-Validator (`validate/`). Benötigen PyPI `bpy` 5.2.3 und `bpy-vrm-format` 4.10.2 (nur lokal, GPL/MIT-or-GPL, nicht Teil des Produkts), Python 3.13.
- **Entfernt:** Mixamo-abgeleitete `assets/animation-*` und Duplikate (Historie bleibt).
- **Repo-Größe:** Klon ca. 770 MB (Owner-Upload der Rohpacks in `content/testcandidate/downloads/` inkl. Historie). Kein History-Rewrite vorgenommen.

## Umgebung / Randbedingungen
Container ohne GPU und ohne Windows. Netzwerk-Allowlist vom Owner erweitert (MakeHuman, Quaternius); `github.com`-Clone bleibt gesperrt, `raw.githubusercontent.com` und PyPI funktionieren (C-10).

## Offene Punkte am Testcandidate
MToon-Materialien (aktuell Standard-PBR), Expression-Blendshapes, Polygonreduktion (Brauen/Wimpern), echter Viewer-Test (Owner), Shirt/Hose-Texturqualität, gesperrte AGPL-Assets nicht verwendet.

## Offene Owner-Entscheidungen
`docs/06-planning/open-questions.md`: OQ-01 (Stack, nach Konzeptarbeit), OQ-04 (Spikes SP-01…09 einzeln), OQ-06…10. Geschlossen heute: OQ-02, -03, -05, -11, -26, -27.

## Recherchequalität
Berichte stammen von Subagenten und wurden nur gelesen/integriert; [R]/[A]-Kennzeichnung beachten. Einzelne Primärquellen waren nicht abrufbar.

## Risiken
`docs/06-planning/risks.md` (Top: R-03 Kleidungsfitting, R-01 speziesübergreifende Geometrie, R-05 Nichtdestruktivität, R-13 Content-Aufwand). Hinweis: R-13 ist durch die CC0-Basis deutlich entschärft.

## Nächste Schritte
`NEXT_ACTIONS.md`.
