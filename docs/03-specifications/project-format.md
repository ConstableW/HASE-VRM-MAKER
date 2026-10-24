# Projektformat (Character-Datei `character.hase.json`) v1.0

**Maßgebliche Quelle für das native Charakter-/Projektformat.** Status: `PROPOSED` (Normativ: `schemas/character.schema.json`). Anforderungen: REQ-ARCH-008/-009, REQ-ADV-010, REQ-EXP-007, REQ-UX-015. Hinweis: Das Format ist **nicht** VRM (OD-011); es erhält alle editierbaren Quelldaten.

| Feld | Bedeutung |
|---|---|
| `formatVersion`, `id`, `name`, `created`, `modified` | Metadaten |
| `body`, `species` | Referenzen `{id, range}` |
| `assets[]` | Instanzen: `ref`, `instance`, `slot`, `enabled`, `params` (Tönung, Skalierung) |
| `parameters` | **sparse** Map Parameter-ID → Wert; Fehlendes = Species-/Body-Default |
| `symmetry` | Symmetrie-Einstellungen (Standard an, pro Parameter überschreibbar) |
| `deltaLayers[]` | Nicht-parametrische Expert-Edits (Sculpt/Weights/UV/Morph) als Dateien **relativ zur evaluierten Basis**; `basis` = Digest des parametrischen Zustands; `invalidation`: `keep`/`reproject`/`warn` (R-05, SP-04) |
| `materialOverrides`, `expressions` | Überschreibungen pro Material/Expression |
| `vrmMeta` | Eingaben für `VRMC_vrm.meta`; Lizenzfelder werden gegen Package-Lizenzen geprüft |
| `lock[]` | exakt aufgelöste Packages `{id, version, digest}` |

## Eigenschaften
- **Single Source of Truth:** Designer- und Expert-Modus lesen/schreiben dieselbe Datei (OD-010). Das evaluierte Mesh wird nie gespeichert, nur abgeleitet.
- **Reproduzierbarkeit:** gleiche Datei + gleiche gelockte Packages ⇒ gleiches Character-Intermediate und gleicher Export (REQ-EXP-007).
- **Migrierbar:** `formatVersion` + Migrationsschritte; alte Dateien bleiben lesbar.
- **Wiederherstellung:** Autosave schreibt temporäre Generationen neben die Datei (UX-Konzept, K5).

Beispiel: `../../examples/species/character.example.hase.json` (validiert mit `tools/validate/validate_packages.py`).
