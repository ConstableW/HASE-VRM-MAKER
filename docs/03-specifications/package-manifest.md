# Package-Manifest (hase-package.json) v1.0

**Maßgebliche Quelle für das Manifestformat.** Status: `PROPOSED`. Normativ ist das JSON-Schema `schemas/package.schema.json`; dieses Dokument erklärt es. Architektur: `../02-architecture/package-system.md`. Beispiele: `../../examples/species/packages/`.

## Gemeinsame Felder

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `schemaVersion` | ja | Fest `"1.0"` |
| `id` | ja | Reverse-DNS (`org.hase.species.elf`), Verzeichnisname = id |
| `type` | ja | `body` \| `species` \| `asset`; genau der passende Abschnitt (`body`/`species`/`asset`) muss vorhanden sein |
| `version` | ja | SemVer des Packages |
| `name`, `description` | ja/nein | String oder Sprach-Map (`{"de":…,"en":…}`) |
| `license` | ja | `spdx` (Pflicht), `attribution`, `redistribution` |
| `provenance[]` | empfohlen | Pro Datei/Ordner: `path`, `source`, `author`, `license`, `retrieved`, `generator` (bei KI/Skript) |
| `requires[]`, `optional[]`, `conflicts[]` | nein | `{id, range}` |
| `files[]` | bei Veröffentlichung | `{path, sha256, size}`; Basis des Digests |
| `dev` | nein | `fileRoot`, `status` (`stub`/`specimen`/`complete`) – nur im Repository, in veröffentlichten Packages **verboten** |
| `extensions` | nein | nur Schlüssel mit Präfix `x-`; vom Kern bei Unkenntnis ignoriert |

## Digest
`sha256:` + SHA-256 über das kanonisierte Manifest (UTF-8, sortierte Schlüssel, Kompaktdarstellung, ohne `$schema` und `dev`). Implementierung: `tools/validate/validate_packages.py::digest`.

## SemVer-Bereiche
`^1.2` (≥1.2.0 <2.0.0), `~1.2.3` (≥1.2.3 <1.3.0), `>=1.0 <2.0`, `1.2.3` (exakt). Bei `0.x` gilt `^0.2` = ≥0.2.0 <0.3.0.

## Typen im Detail
- **Body:** `species-schema.md` verweist auf den Body-Vertrag; Body-Felder siehe Schema-Abschnitt `$defs/body` (Topologie, Skeleton + Humanoid-Mapping, Morph-Parameter, Slots, Fit, Export).
- **Species:** `species-schema.md`.
- **Asset:** `asset-schema.md`.

## Versionierungsregeln (MAJOR/MINOR/PATCH)
| Änderung | Package | Body-Schnittstelle |
|---|---|---|
| Parameter hinzufügen, Slot hinzufügen, Target verbessern (gleiche Namen) | MINOR/PATCH | MINOR |
| Parameter-/Slot-/Bone-Namen umbenennen/entfernen, Topologie/Skeleton wechseln, Wertebereich einengen | MAJOR | MAJOR |
| Texturen/Meshes tauschen ohne Namens- oder Topologieänderung | MINOR | – |
