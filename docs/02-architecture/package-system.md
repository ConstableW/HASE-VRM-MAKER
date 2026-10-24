# Package-System

**Maßgebliche Quelle für Aufbau, Versionierung, Auflösung, Integrität und Sicherheit von HASE-Packages.** Status: `PROPOSED` (ADR-0005). Formale Schemas: `../03-specifications/`. Anforderungen: REQ-PKG-001…013, REQ-ARCH-010, OD-008/-009. Vorbilder und Quellen: `../04-research/references.md` Teil B.

## 1. Pakettypen

| Typ | Inhalt | Schema-Abschnitt |
|---|---|---|
| `body` | Basismesh, Topologie, Skeleton/Rig, Gewichte, Morph-Parameter + Target-Daten, Slots, Fit-Methoden, Export-Fähigkeiten | `body` |
| `species` | Defaults, Limits, Traits, Komponenten, Presets, Capabilities | `species` |
| `asset` | Haare, Augen, Brauen, Wimpern, Kleidung, Accessoires, Bodyparts, Morphsets, Materialien, Animationen, Posen, Presets | `asset` (Feld `kind`) |

Auch die mitgelieferten Standardspezies sind Packages derselben Infrastruktur (REQ-PKG-001). Der Kern enthält **keine** Spezies-Daten.

## 2. Aufbau

```
org.hase.body.humanoid-mh08/
├── hase-package.json        # Manifest (Pflicht)
├── data/…                   # Meshes, Targets, Rigs, Texturen
├── LICENSES/                # Lizenztexte (optional, empfohlen)
└── PROVENANCE.json          # optional, Detailherkunft (sonst im Manifest unter `provenance`)
```
Verteilung als `.hasepkg` (ZIP, deterministisch: sortierte Einträge, feste Zeitstempel). Lokale Entwicklung als Ordner.

## 3. Identität, Version, Integrität

- **ID:** Reverse-DNS (`org.hase.species.elf`). Eindeutig, unveränderlich.
- **Version:** SemVer. **Schnittstellenversion** (`provides.version` im Body) ist unabhängig von der Package-Version.
- **Digest:** `sha256` über das *kanonisierte* Manifest (sortierte Schlüssel, ohne `dev`/`$schema`); das Manifest enthält `files[]` mit je SHA-256 → Digest deckt Inhalt ab (Merkle-artig). Referenz im Projekt: `id` + `version` + `digest` im **Lockfile-Block** der Charakterdatei (`lock`).
- **Signatur (optional, später):** Ed25519 über den Digest. Vertrauensmodell Registry (TUF-artige Rollen) ist offen (OQ-07, OQ-08).
- **Lizenz/Herkunft:** Pflicht `license.spdx`; Datei-Herkunft in `provenance` (Quelle, Urheber, Lizenz, Datum, Generator bei KI/Skript). Der Editor aggregiert und zeigt sie, der Export schreibt sie in `VRMC_vrm.meta` (REQ-PKG-011).

## 4. Abhängigkeiten und Auflösung

- `requires` (Pflicht), `optional`, `conflicts`, jeweils `{id, range}` mit SemVer-Bereichen (`^1.2`, `~1.2.3`, `>=1.0 <2.0`).
- Auflösung: PubGrub-artig (Konfliktableitung, erklärbare Fehlerkette), deterministisch (stabile Kandidatenreihenfolge, höchste kompatible Version). Referenzimplementierung im Prototyp: `tools/validate/validate_packages.py` (Bereichsprüfung, Interface-/Slot-/Capability-Regeln; kein vollständiger Solver).
- Unabhängige Updates: Ein Asset gegen `humanoid-mh08 ^1.0` baut weiter, solange die Schnittstelle MINOR/PATCH bleibt. Umbenennungen von Parametern werden über deklarative **Migrations-Maps** aufgefangen (geplant, Feld `migrations`).

## 5. Kompatibilitätsregeln
Siehe `../03-specifications/compatibility-rules.md` (Regeln C1…C9, mit Fehlercodes). Kurz: Interface-Treffer, Capabilities, Slots (exklusiv/Layer), Spezies-Freigabe, Parameter-Existenz/-Bereiche, Lizenz-Aggregation, Export-Profil-Prüfung.

## 6. Sicherheit
- **Daten vor Code:** Packages sind deklarativ. Das Manifest erlaubt keine Skripte. Erweiterungen laufen als `extensions` mit Präfix `x-` und werden vom Kern ignoriert, wenn unbekannt.
- **Ausführbarer Code** (z. B. prozedurale Generatoren) nur später und nur als **WASM-Komponente** in einer Sandbox mit explizitem Capability-Manifest, Speicher-/Zeit-Limits (wasmtime laut `../04-research/technology-evaluation.md`). Native Plugins nur für vertrauenswürdige, signierte Quellen (OQ-08).
- Mesh-/Bilddaten werden beim Laden validiert (Größenlimits, Index-Grenzen, Format) – Parser sind der Angriffsvektor, nicht das Manifest.
- Entpacken: Pfadtraversal (`..`, absolute Pfade) und Symlinks im Archiv verboten; `dev`-Feld in veröffentlichten Packages verboten (`--publish`).

## 7. SDK und Builder (Konzept, REQ-PKG-007/-008/-009)

| Baustein | Aufgabe | Status |
|---|---|---|
| **HASE Package SDK (CLI + Bibliothek)** | `validate`, `pack`, `inspect`, `lock`, `new-species`, `import-mhclo` | Prototyp: `validate_packages.py`; Rest geplant |
| **Importer** | MakeHuman-Targets/.mhclo, glTF/GLB/VRM, FBX (ufbx), BVH/VRMA, Texturen | Teilprototyp: `tools/assets/*.py` (bpy) |
| **Blender-Add-on (optional, separater Prozess)** | Export von Meshes/Shape Keys/Rig in Package-Struktur | geplant |
| **Visueller Package Builder** | Geführte Erstellung von Species-/Asset-Packages inkl. Testvorschau und Validierung | geplant (Phase I4) |

## 8. Offene Punkte
Registry/Distribution (OQ-07), Signatur-/Vertrauensmodell (OQ-08), Migrations-Map-Format, vollständiger Solver, Umgang mit Binärdaten-Größe (Delta-Updates), `uses`-Mechanismus für Trait-Import (OQ-30).
