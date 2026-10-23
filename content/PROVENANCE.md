# Provenienzregister (Content)

**Maßgebliche Quelle für Herkunft und Lizenz aller Content-Dateien.** Grundlage: ADR-0010, OD-024. Pflege: bei jeder neuen Fremd- oder KI-Datei ergänzen. Dateihashes stehen in den `MANIFEST.sha256` je Quellverzeichnis.

| Verzeichnis / Datei | Quelle | Lizenz | Abruf | Verwendung |
|---|---|---|---|---|
| `testcandidate/mpfb2-cc0/base.obj` | https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/src/mpfb/data/3dobjs/base.obj | CC0-1.0 (MPFB2 `LICENSE.md` §C, Kopie im Verzeichnis; Datei-Header „released as CC0 in september 2020“) | 2026-10-23 (branch `master`, Commit-Hash nicht ermittelbar: GitHub-API im Container gesperrt) | Basis-Körpermesh des Testcandidate |
| `testcandidate/mpfb2-cc0/targets/**` (609 Morph-Targets, `*.target.gz` + `target.json`) | `.../src/mpfb/data/targets/` | CC0-1.0 (Repo-`LICENSE.md` §C; Dateien ohne eigenen Header). Weggelassen: Gruppe `genitals` | 2026-10-23 | Morph-Set (Körper, Gesicht, Makro) |
| `testcandidate/mpfb2-cc0/rigs/*.json` | `.../src/mpfb/data/rigs/standard/` | CC0-1.0 (§C „Rigs“) | 2026-10-23 | MakeHuman-Standardrig (163 Bones), Mapping auf VRM-Humanoid steht aus |
| `testcandidate/mpfb2-cc0/LICENSE.md`, `LICENSE.ASSETS.md` | MPFB2-Repo | GPL-3.0 (Code) / CC0 (Assets) | 2026-10-23 | Lizenzbeleg. **MPFB2-Code wird nicht verwendet.** |
| `testcandidate/body_mpfb2_testcandidate.glb` | abgeleitet via `tools/assets/build_testcandidate_body.py` | CC0-1.0 (abgeleitet) | – | Build-Artefakt (45 MB), nicht versioniert (`.gitignore`) |
| `testcandidate/avatar_a_quaternius_female.vrm` | abgeleitet aus Quaternius „Universal Base Characters [Standard]“ (Superhero_Female + Hair_Buns) via `tools/assets/build_quaternius_vrm.py` | CC0-1.0 (abgeleitet; Quelle `License_Standard.txt`) | 2026-10-23 | Test-Avatar A (VRM 1.0, 52 Humanoid-Bones) |
| `testcandidate/animations/UAL1`, `UAL2` (84 .vrma) | abgeleitet aus Quaternius „Universal Animation Library 1/2 [Standard]“ via `tools/assets/build_quaternius_vrma.py` | CC0-1.0 (abgeleitet; Quelle `License.txt`) | 2026-10-23 | CC0-Animationsbibliothek als VRMA, Ersatz für `assets/animation-*` |
| `assets/animation-*` | SillyTavern-Pack (Mixamo-Herkunft) | ungeklärt | – | nur interne Tests (OQ-03) |

## MakeHuman-Asset-Packs (Owner-Upload 2026-10-23, `testcandidate/downloads/`)
Quelle: https://static.makehumancommunity.org/assets/assetpacks.html (Packs `makehuman_system_assets`, `hair01`, `shirts01`, `pants01`, `shoes01`, `eyebrows01`, `eyelashes01`; Metadaten je Pack in `downloads/packs/*.json`). Pro-Asset-Liste mit Autor, Lizenz und Status: **`PROVENANCE_downloads.csv`** (175 Einträge).
- Packliste: 174 × CC0, 1 × CC-BY (`culturalibre_heroine_boots_4`, Attribution nötig).
- **Abweichung in Dateiheadern:** 14 `.mhclo`-Dateien (Haare `culturalibre_hair_01/02`, `elvs_*_braid*`, `learning_anime_hair`, `littleright_bobcut_hair`, `rehmanpolanski_hair_bun_brown`, `sonntag78_*`; Kleidung `grinsegold_female_pirate_boots`, `scailman_*_boots`, `skalldyrssuppe_tube_top_funky_colors`) tragen den Header `AGPL3`, obwohl die Packliste CC0 angibt (ältere Header, vermutlich nicht aktualisiert). Status in der CSV: **GESPERRT**. Der Testcandidate verwendet sie nicht, bis die Lizenz geklärt ist (OD-024: Prüfung nur bei tatsächlicher Verwendung). 2 Haare tragen CC-BY (`elvs_reverse_french_braid_bun`, `o4saken_long01`): Attribution nötig.
- Alle übrigen Assets: CC0 laut Pack und Header (bzw. Header ohne Lizenzzeile → Packliste gilt).
- Quaternius-Packs (`Universal Animation Library`, `…Library 2`, `Universal Base Characters`, je [Standard]): CC0 laut mitgelieferter `License.txt`. Liegen zusätzlich unter `assets/` (Duplikat, später bereinigen).

## Offen / noch nicht beschafft (Container-Netz, siehe C-10)
Nichts Grundlegendes mehr. Offene technische Arbeit: Rig-Mapping, Reduzierung, VRM-Export. Details: `docs/04-research/assets-candidates.md`.
