# Provenienzregister (Content)

**Maßgebliche Quelle für Herkunft und Lizenz aller Content-Dateien.** Grundlage: ADR-0010, OD-024. Pflege: bei jeder neuen Fremd- oder KI-Datei ergänzen. Dateihashes stehen in den `MANIFEST.sha256` je Quellverzeichnis.

| Verzeichnis / Datei | Quelle | Lizenz | Abruf | Verwendung |
|---|---|---|---|---|
| `testcandidate/mpfb2-cc0/base.obj` | https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/src/mpfb/data/3dobjs/base.obj | CC0-1.0 (MPFB2 `LICENSE.md` §C, Kopie im Verzeichnis; Datei-Header „released as CC0 in september 2020“) | 2026-10-23 (branch `master`, Commit-Hash nicht ermittelbar: GitHub-API im Container gesperrt) | Basis-Körpermesh des Testcandidate |
| `testcandidate/mpfb2-cc0/targets/**` (609 Morph-Targets, `*.target.gz` + `target.json`) | `.../src/mpfb/data/targets/` | CC0-1.0 (Repo-`LICENSE.md` §C; Dateien ohne eigenen Header). Weggelassen: Gruppe `genitals` | 2026-10-23 | Morph-Set (Körper, Gesicht, Makro) |
| `testcandidate/mpfb2-cc0/rigs/*.json` | `.../src/mpfb/data/rigs/standard/` | CC0-1.0 (§C „Rigs“) | 2026-10-23 | MakeHuman-Standardrig (163 Bones), Mapping auf VRM-Humanoid steht aus |
| `testcandidate/mpfb2-cc0/LICENSE.md`, `LICENSE.ASSETS.md` | MPFB2-Repo | GPL-3.0 (Code) / CC0 (Assets) | 2026-10-23 | Lizenzbeleg. **MPFB2-Code wird nicht verwendet.** |
| `testcandidate/body_mpfb2_testcandidate.glb` | abgeleitet via `tools/assets/build_testcandidate_body.py` | CC0-1.0 (abgeleitet) | – | Build-Artefakt (45 MB), nicht versioniert (`.gitignore`) |
| `assets/animation-*` | SillyTavern-Pack (Mixamo-Herkunft) | ungeklärt | – | nur interne Tests (OQ-03) |

## Offen / noch nicht beschafft (Container-Netz, siehe C-10)
Haare, Augen, Kleidung, Skins (MakeHuman System Assets, Host `static.makehumancommunity.org`), CC0-Animationen (Quaternius). Fallbacks: eigene prozedurale Assets. Details: `docs/04-research/assets-candidates.md`.
