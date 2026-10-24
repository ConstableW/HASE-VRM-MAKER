# Asset-Schema

**Maßgebliche Quelle für den Abschnitt `asset`** (Typ `asset`). Normativ: `schemas/package.schema.json` (`$defs/asset`).

| Feld | Bedeutung |
|---|---|
| `kind` | `hair`, `eyes`, `eyebrows`, `eyelashes`, `clothing`, `accessory`, `bodypart`, `morphset`, `material`, `animation`, `pose`, `preset` |
| `targets.bodyInterface` | Pflicht: `{id, range}` der Body-Schnittstelle |
| `targets.species` | optional: Liste erlaubter Spezies-IDs |
| `occupies[]` | belegte Slot-IDs des Bodys (Exklusivität/Layer laut Body) |
| `layer` | Kleidungs-Layer innerhalb eines Slots |
| `requiresCapabilities[]` | Fähigkeiten, die Body bzw. Spezies bieten müssen (z. B. `spring-extras`) |
| `fit.method` | `mhclo-barycentric` (an Basismesh-Vertices gebunden), `weight-transfer`, `rigid-attach` (starr an Bone), `none` |
| `fit.definition/mesh/basemesh` | Dateien der Fit-Definition (z. B. `.mhclo` + `.obj`) und Basis-Familie |
| `materials[]` | `{id, shader: mtoon1\|pbr, texture, baseColor, alphaMode, tintable}` |
| `extraBones[]` | Zusatz-Bones `{name, parent, springChain}` (Ohren, Schwanz, Haar) |
| `morphs` | zusätzliche Parameter (nur bei `morphset`/`bodypart`) mit Namespace |
| `files` | benannte Dateizuordnungen |

## Regeln
- Ein Asset darf nur Slots belegen, die der Ziel-Body definiert (C3).
- `bodypart`/`accessory` mit `extraBones` werden beim VRM-Export zu Spring-Chains bzw. Extra-Nodes (`../02-architecture/overview.md` P6).
- `animation`-Assets (VRMA) hängen nur von der Humanoid-Capability ab, nicht vom Mesh.
- Kleidung bleibt über Proportionen hinweg wiederverwendbar, solange T0/T1-Topologie (`species-system.md` §3); darüber hinaus nur mit eigener Fit-Methode.

Beispiele: `../../examples/species/packages/org.hase.asset.*`.
