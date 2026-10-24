# Species-Schema

**Maßgebliche Quelle für den Abschnitt `species`** (Typ `species`). Normativ: `schemas/package.schema.json` (`$defs/species`). Konzept: `../02-architecture/species-system.md`.

| Feld | Bedeutung |
|---|---|
| `targets.bodyInterface` | `{id, range}` – Body-Schnittstelle, auf der die Spezies aufbaut (Pflicht) |
| `defaults` | Parameter-ID → Startwert (überschreibt Body-Default) |
| `limits` | Parameter-ID → `[min, max]`; muss **innerhalb** des Body-Bereichs liegen (Regel C6) |
| `traits` | Name → `{label, set:{Parameter-ID: Wert}}` – wiederverwendbare Bündel, im Editor als Schalter/Schieber |
| `components[]` | Assets, die automatisch angebracht werden: `{asset, slot, optional}`; Slot muss vom Asset belegt und im Body vorhanden sein (C3) |
| `capabilities[]` | zusätzliche Fähigkeiten, die Komponenten voraussetzen können (z. B. `trait:spring-extras`) |
| `presets[]` | benannte Parameter-/Asset-Sets (`parameters`, `assets`) |

Parameter-IDs haben die Form `namespace:name` (z. B. `mh:head-age`) und müssen im Body (oder in einem Morphset-Asset) existieren.

## Beispiel (gekürzt)
```json
{ "id": "org.hase.species.cat-demi", "type": "species", "version": "1.0.0",
  "requires": [{"id":"org.hase.body.humanoid-mh08","range":"^1.0"}, {"id":"org.hase.asset.bodypart.cat-ears","range":"^1.0"}],
  "species": {
    "targets": {"bodyInterface": {"id":"org.hase.iface.humanoid-mh08","range":"^1.0"}},
    "defaults": {"mh:macro-age": 0.35},
    "traits": {"cat-eyes": {"set": {"mh:l-eye-height1": -0.3, "mh:r-eye-height1": -0.3}}},
    "components": [{"asset":"org.hase.asset.bodypart.cat-ears","slot":"head.ears"}, {"asset":"org.hase.asset.bodypart.cat-tail","slot":"back.tail"}],
    "capabilities": ["trait:spring-extras"] } }
```
Vollständig: `../../examples/species/packages/org.hase.species.*/`.
