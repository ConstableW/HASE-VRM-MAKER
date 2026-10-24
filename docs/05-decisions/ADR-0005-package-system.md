# ADR-0005: Package-System (Manifest, Versionierung, Auflösung, Sicherheit)

- **Status:** PROPOSED (Owner-Freigabe offen)
- **Datum:** 2026-10-24
- **Betrifft:** REQ-PKG-001…013, REQ-ARCH-010, R-02, R-09, OQ-07/-08/-30
- **Evidenz:** `../02-architecture/package-system.md`, `../03-specifications/`, `examples/species/`, `tools/validate/validate_packages.py` (Selbsttests), Vorbilder in `../04-research/references.md` Teil B

## Kontext
Spezies und Assets sollen installierbar, versioniert, validierbar und unabhängig aktualisierbar sein; Standardspezies nutzen dieselbe Infrastruktur.

## Alternativen
| Option | Pro | Contra |
|---|---|---|
| A. Ordner ohne Manifest/Versionen | trivial | keine Kompatibilität, keine Reproduzierbarkeit |
| B. Plugins mit ausführbarem Code | mächtig | Sicherheit, Stabilität |
| **C. Deklarative Packages mit Manifest, SemVer, Interface-Verträgen, Digest + Lockfile, Code nur sandboxed später (gewählt, PROPOSED)** | sicher, reproduzierbar, erweiterbar | Resolver- und Validatorpflege |

## Entscheidung (Empfehlung)
Option C: `hase-package.json` (Schema 1.0), Reverse-DNS-IDs, SemVer, getrennte Schnittstellenversion, SHA-256-Digest über kanonisiertes Manifest + Dateihashes, Lockfile im Charakter, Regeln C1…C9, optionale Ed25519-Signatur, WASM-Sandbox für künftigen Code.

## Konsequenzen
- Format ist PROPOSED und kann bis zur ersten veröffentlichten Version geändert werden (`schemaVersion`).
- Registry/Distribution und Vertrauensmodell sind getrennt zu entscheiden (OQ-07/-08).
- Der Prototyp-Validator ist kein vollständiger Solver; PubGrub-artige Implementierung folgt (SP-06).
