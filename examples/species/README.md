# Beispielpakete (Species/Body/Asset) und Beispielcharakter

Spezifikation: `docs/03-specifications/`. Architektur: `docs/02-architecture/species-system.md`, `package-system.md`.

| Package | Typ | Status |
|---|---|---|
| `org.hase.body.humanoid-mh08` | body | specimen – Daten liegen unter `content/testcandidate/mpfb2-cc0/` (`dev.fileRoot`), Manifest vollständig |
| `org.hase.species.human` / `elf` / `cat-demi` / `dog-demi` | species | specimen/stub – Datenmodell belegt, Geometrie für Ohren/Schwanz/Schnauze fehlt |
| `org.hase.asset.hair.short02`, `clothing.tshirt-basic`, `clothing.wool-pants` | asset | specimen – Daten unter `content/testcandidate/downloads/` |
| `org.hase.asset.bodypart.*`, `morphset.muzzle-dog` | asset | **stub** – nur Manifest |
| `character.example.hase.json` | Charakter | referenziert Body + Mensch + Haar + Shirt + Hose; `lock` per `--write-lock` erzeugt |

Prüfen (Python ≥3.10, `pip install jsonschema`):
```
python tools/validate/validate_packages.py --selftest
```
`dev` (Entwicklungsfeld mit Dateiwurzel) ist nur in Beispielen erlaubt; `--publish` verbietet es.
