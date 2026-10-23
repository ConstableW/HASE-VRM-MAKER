> **Status:** Rechercheergebnis (Subagent, 2026-10-23), nicht vom Owner geprüft. Maßgeblich für: Asset-Kandidaten, Erreichbarkeit, Blender-MCP/Add-ons. Lizenzprüfung gemäß OD-024 nur bei tatsächlicher Verwendung; dort greift das Provenienzregister `content/PROVENANCE.md`.

# Recherche: lizenzkonforme Open-Source-Assets für den Testcandidate

Abrufdatum aller Quellen: 2026-10-23. Bezug: ADR-0009 (Apache-2.0 Code, Eigen-Assets CC0/CC-BY), ADR-0010 (nur CC0/Unlicense/MIT/BSD, CC-BY nur mit Attribution, kein NC/ND/SA/GPL-Daten).
Label: RESEARCHED = von mir im Primärtext (LICENSE-Datei, Datei-Header, Datei-Metadaten, PyPI-JSON) gelesen. RESEARCHED-SEKUNDÄR = nur aus Websuche/Drittseite. ASSUMED = nicht belegt. Keine Rechtsberatung.

## 0. Kernbefund

1. MPFB2-Repo (`makehumancommunity/mpfb2`, Branch `master`) liefert den MakeHuman-Basiskörper (Quad-Mesh), das Rig und über 1000 Morph-Targets als einzelne Dateien. Die Assets stehen laut `LICENSE.md` Abschnitt C unter CC0 1.0. Alles ist vom Container per `raw.githubusercontent.com` holbar, ohne Allowlist-Änderung.
2. Haare, Augen, Kleidung, Hauttexturen und Zähne liegen nur in MakeHuman-Asset-Packs (ZIP) auf `static.makehumancommunity.org`. Dieser Host ist im Container gesperrt (HTTP 000). Dort ist eine Allowlist-Freigabe nötig.
3. Animationen: Im Container gibt es nur `test.vrma` (11,5 KB, Handarbeit) aus three-vrm. CC0-Animationspakete (Quaternius) liegen auf gesperrten Hosts. Eigene prozedurale Animationen (Eigen-Asset, CC0) sind sofort möglich.
4. Fertige VRM-Beispielmodelle (Seed-san, Twist-Sample) stehen unter der VRM Public License 1.0. Das ist keine SPDX-Lizenz und nicht CC0. Sie taugen als Referenz/Testdaten, nicht als Teil der Standardauslieferung.

## 1. Basismeshes / Körper / Köpfe

### 1.1 MPFB2/MakeHuman-Basismesh (Empfehlung für Body und Kopf)

| Punkt | Befund |
|---|---|
| Lizenz | CC0-1.0. RESEARCHED: `LICENSE.md` §C ("The base mesh and proxies, Targets and modifiers, Textures, Clothes (any MHCLO-based asset), Rigs, poses and expressions ... released under CC0 1.0 Universal"). `LICENSE.ASSETS.md` enthält den CC0-Volltext und nennt keine Ausnahmen. `base.obj`-Header: "explicitly released as CC0 in september 2020". |
| Code-Lizenz | GPLv3 (MPFB2), AGPL (MakeHuman). Wird nicht übernommen (ADR-0009 Regel 4). Nur Daten werden genutzt. |
| Topologie | RESEARCHED (selbst gezählt an `base.obj`): 19.158 Vertices, 18.486 Faces, ausnahmslos Quads, 172 Gruppen. Enthalten sind Körper, `helper-tights` und Joint-Helper-Gruppen. Das sind MakeHuman-Standardtopologie und -Edge-Flow, gut für Gesicht und Gelenke. Die Polyzahl ist mit knapp 19k Quads (ca. 37k Tris) hoch für ein VRM. Eine Dezimierung oder ein Proxy ist nötig (VRM-Richtwerte siehe `docs/04-research`). |
| Morphs | RESEARCHED: `targets/target.json` kategorisiert Targets in 24 Gruppen: arms 50, asym 62, breast 12, cheek 16, chin 15, ears 44, eyebrows 6, eyes 68, feet 30, forehead 8, hands 22, head 27, hip 14, legs 58, mouth 44, neck 20, nose 42, pelvis 4, stomach 8, torso 34 und weitere. Zusätzlich gibt es Makrotargets (`macrodetails/*.target.gz`, z. B. `african-female-young.target.gz` 123 KB) und Ausdrucks-Units (`targets/expression/units/{african,asian,caucasian}`). Die Dateien sind gzip-komprimierte Offsets (`index x y z`) ohne Lizenz-Header. Die Lizenz gilt daher nur über `LICENSE.md` des Repos. Provenienzregister: Repo-Commit-Hash festhalten. |
| Rig | RESEARCHED: `rigs/standard/rig.default.json` (126 KB, 163 Bones inkl. Gesichts-/Finger-/Twist-Bones) und `rig.default_no_toes.json`. Bonenamen sind MakeHuman-spezifisch (z. B. `upperleg01.L`). Sie sind nicht direkt VRM-Humanoid, ein Mapping auf die VRM-Humanoid-Bones ist nötig. Das ist machbar (ASSUMED). Ein reduziertes Game-Engine-Rig existiert in MPFB2 (ASSUMED, Dateiname nicht geprüft). |
| Risiken | Das Mesh ist anatomisch (nackt, mit Brust-/Genital-Targets). Für ein Produkt braucht es Kleidung oder ein bereinigtes Release-Mesh. CC0 §4(c): Persönlichkeits-/Publicity-Rechte sind nicht abgedeckt (kein reales Vorbild bekannt, ASSUMED). Das Quellmesh stammt aus der MakeHuman-Historie. Die CC0-Erklärung von 2020 wird vom Projekt selbst gegeben (siehe auch Hinweis in der MakeHuman-FAQ, Sekundärquelle). |
| Quellen | https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/LICENSE.md, `.../LICENSE.ASSETS.md`, `.../src/mpfb/data/3dobjs/base.obj`, `.../src/mpfb/data/targets/target.json` |

Sekundär bestätigt: MakeHuman-FAQ "What changed regarding the license in 2020" (Basismesh/Targets/Proxies von AGPL auf CC0 umgestellt). Die Seite liegt auf `static.makehumancommunity.org`; Inhalt nur über die Websuche gelesen. Die offizielle Lizenzseite nennt: Assets CC0, MakeHuman AGPL, MPFB GPL.

### 1.2 PyPI-Paket `makehuman` 1.3.2 (nicht empfohlen)

PyPI-JSON: Version 1.3.2, Upload 2025-04-17, Wheel 30,5 MB (`files.pythonhosted.org` erreichbar, HTTP 200). Inhalt (selbst entpackt, nur im Scratchpad): AGPL-Python-Code und `data/targets.npz` (30 MB, 2.561 Einträge, kompilierte Targets). Es enthält kein Basismesh, keine Haare, keine Kleidung. In der Paket-Metadaten steht keine Lizenz. Es ist nur Code plus Binärdaten, daher nicht nützlich (siehe 1.1, einzelne Dateien sind besser nachvollziehbar). Hinweis: `targets.npz` enthält Pickle-Objekte und wurde bewusst nicht geladen.

### 1.3 Weitere Kandidaten

| Kandidat | Lizenz | Eignung | Container |
|---|---|---|---|
| Quaternius "Universal Base Characters" | CC0 (RESEARCHED-SEKUNDÄR: itch.io-Listing laut Websuche; Seite selbst im Container nicht abrufbar). Standard-ZIP 122 MB, Source 599 MB (mit .blend, kostenpflichtig / Pay-what-you-want). | 6 Charaktere (Superhero/Regular/Teen), ca. 13k Tris, "Humanoid rig", 20 Frisuren. Shape Keys / Gesichtsmorphs nicht belegt (ASSUMED: wenige oder keine). Stilisiert, nur Fixmodelle statt parametrischer Morphs. Gut als Vergleichsbasis und zum Testen des Imports. | nein (`quaternius.itch.io` / `quaternius.com` gesperrt) |
| Blender Studio "Human Base Meshes" v1.4.1 | CC0 (Sekundär: CGChannel 2023, blender.org Asset-Bundles, 49 MB, .blend, Blender >= 4.2) | 17 Basemeshes, als Modellier-Referenz brauchbar, keine fertige Morph-/VRM-Pipeline. | nein (`blender.org` gesperrt) |
| VRM-Sample "Seed-san" (VirtualCast) | VRM Public License 1.0 (RESEARCHED: `samples/Seed-san/README.md` und VRMC_vrm-Meta: `allowRedistribution: true`, `modification: allowModificationRedistribution`, `creditNotation: required`, `commercialUsage: corporation`) | 10,9 MB, VRM 1.0, 5 Meshes mit 21 Primitives. Referenz für Export/Validator-Tests. Lizenz nicht CC0/SPDX: nur interne Testdaten, nicht im Repo/Release. | ja: `https://raw.githubusercontent.com/vrm-c/vrm-specification/master/samples/Seed-san/vrm/Seed-san.vrm` (HTTP 200) |
| VRM1_Constraint_Twist_Sample (pixiv) | VRM Public License 1.0 (RESEARCHED: Meta `creditNotation: unnecessary`, `allowRedistribution: true`, `allowAntisocialOrHateUsage: false`) | 10,8 MB, Referenz für Constraints/SpringBones | ja: `https://raw.githubusercontent.com/pixiv/three-vrm/dev/packages/three-vrm/examples/models/VRM1_Constraint_Twist_Sample.vrm` (HTTP 200) |
| VRoid-Studio-CC0-Beispielmodelle (OpenGameArt) | CC0 laut OpenGameArt (Sekundär). Die Seite sagt, AvatarSample A bis C fehlten, weil restriktiver lizenziert. VRM 0.x, aus Alpha-Versionen. | Anime-Stil, andere Topologie, VRM 0.x. Nicht zur Basis geeignet. | nein (`opengameart.org` gesperrt) |
| godot-vrm / UniVRM Samples | nicht geprüft (ASSUMED: je Datei eigene Lizenzen) | nicht verwendet | UniVRM-README per raw erreichbar, kein Modell geprüft |

Poly Haven, ambientCG: laut Drittquellen CC0 (Sekundär); enthalten keine Charakter-Basismeshes, nur HDRIs/Texturen/Props.

## 2. Haare, Augen, Kleidung, Texturen

RESEARCHED-SEKUNDÄR (Websuche zu `static.makehumancommunity.org/assets/assetpacks*`, Seiten selbst im Container nicht abrufbar):

| Pack | Lizenz | Inhalt |
|---|---|---|
| `makehuman_system_assets` (ca. 267 MB, Spiegel mirror1/mirror2) | CC0 | Proxies, Hair, Eyebrows, Eyelashes (ASSUMED), Clothes, Eyes, Teeth, Skins. Hair 01 ist laut Suchtreffer CC0 (low-poly, stilisiert). |
| `hair01` | CC0 | stilisierte Low-Poly-Haare |
| `hair02`, `hair03` | CC-BY | High-Poly-Haare (Elvaerwyn), Alpha-7-Adaptionen: Attribution nötig (Urheber je Datei prüfen) |
| `hats*`, `suits*` | CC-BY (laut Treffer für Hats) | Kleidung mit Attributionspflicht |
| `system_hair_materials01/02`, `system_eye_materials01/02`, `system_clothes_materials01` | je Pack (neuere Packs; laut Treffer "Release 2026-07-17/-24" auch ein neuer CC-BY-Abschnitt "System eye materials 03") | Zusatzmaterialien |
| Einzelne Augentexturen | CC0 (OpenGameArt, 9 Texturen 1024x1024 "taken from MakeHuman System Assets") | Iris-Texturen |

Pro Datei lizenziert sich jedes Asset über seine `.mhclo`/`.mhmat`-Header. Jede Datei ist vor der Aufnahme zu prüfen (ADR-0010: Dateiprüfung). Ich konnte die ZIPs nicht laden. Eine Inhaltsprüfung steht daher aus (ASSUMED).

Alternativen im Container: keine verifizierten Haar-/Augen-/Kleidungsassets über `raw.githubusercontent.com` gefunden. Die Asset-Repos von MakeHuman waren unter den geprüften Namen (`makehuman-assets`, `makehuman-system-assets` u. a.) nicht auffindbar (HTTP 404). Eigene Augen (prozedural, CC0) und KI-/Eigentexturen für Iris/Haut sind im Rahmen von ADR-0010 sofort möglich.

## 3. Animationen

| Quelle | Lizenz | Bewertung | Container |
|---|---|---|---|
| Quaternius Universal Animation Library (Standard: 45 Anim., kostenlos; Pro/Source: 120+ bezahlt) und UAL 2 (130+, OpenGameArt Jan. 2026) | CC0 (Sekundär: itch.io-/OpenGameArt-Treffer) | Beste CC0-Quelle. Rig "universal humanoid" (nicht VRM). Retargeting auf VRM-Humanoid nötig. | nein (itch.io, quaternius.com, opengameart.org gesperrt) |
| pixiv three-vrm `test.vrma` | im MIT-Repo (Copyright pixiv Inc.; RESEARCHED: `LICENSE` MIT, `README` "LICENSE MIT"). Dass die Datei selbst unter MIT fällt, ist ASSUMED. Generator "VRM Add-on for Blender v2.20.4 / handwritten", VRMA specVersion 1.0, Expressions+LookAt+Humanoid. | 11,5 KB. Nur Formattest, keine echte Bewegung (ASSUMED). | ja: `https://raw.githubusercontent.com/pixiv/three-vrm/dev/packages/three-vrm-animation/examples/models/test.vrma` |
| VRoid-Project VRMA_01 bis 07 (BOOTH) | Bedingungen auf der BOOTH-Seite, nicht gelesen (Websuche) | Nicht verwendbar ohne Prüfung. Andere BOOTH-VRMA nur "edits only free, on BOOTH" (nicht kompatibel). | nein |
| CMU Mocap | KEIN CC0. Websuche: Daten in kommerziellen Produkten nutzbar, aber "may not resell this data directly, even in converted form". Konvertierungen: 4TU-FBX unter CC BY-NC 4.0, cgspeed ohne Lizenzangabe gefunden. | Für Redistribution im Package-System ungeeignet (Weiterverkaufs-/Weitergabe-Klausel). Nicht in Testcandidate aufnehmen. | nein (HuggingFace gesperrt) |
| Mixamo | ausgeschlossen (Owner-Vorgabe) | | |
| Eigene, prozedurale Animationen | CC0 (Eigen-Asset) | Idle, Atmen, Kopfwackeln, T-/A-Pose-Test. Sofort möglich. Minimal-Fallback für den Testcandidate. | ja (lokal erzeugt) |

## 4. Blender-MCP und Blender-Add-ons

### 4.1 ahujasid/blender-mcp (jetzt "MCP for Blender")

- Lizenz: MIT. RESEARCHED: `LICENSE` (Copyright 2025 Siddharth Ahuja) und README-Badge. PyPI `mcp-for-blender` 2.2.3 (License-Expression MIT, Python >=3.10, Abhängigkeiten `mcp>=1.26`, `httpx`). PyPI `blender-mcp` 2.0.0 (2026-09-16) ist nur noch eine Umleitung auf `mcp-for-blender`.
- Aktivität: Hoch. `mcp-for-blender` 2.2.3 wurde am 2026-10-23 veröffentlicht (PyPI-Upload 09:17/09:18). Frühere `blender-mcp`-Versionen reichen bis 2.0.0. GitHub-Stars nicht geprüft (github.com gesperrt).
- Funktionsumfang laut README (RESEARCHED, raw README): 9 Tools: `execute_blender_code`, `look` (Viewport-/Kamera-/Multi-Angle-/Animations-Frame), `get_scene_info`, `generate_3d` (Meshy, Tripo, Hyper3D Rodin, Hunyuan3D), `search_assets`/`import_asset` (Poly Haven CC0, Sketchfab mit API-Key, Poly Pizza mit Key, ca. 69 % CC-BY laut README), `get_addon_status`, `disable_telemetry`, `record_trajectory_feedback`.
- Sicherheit (RESEARCHED, README): `execute_blender_code` führt beliebigen Python-Code in Blender aus ("potentially dangerous", "ALWAYS save your work"). Der Socket-Server des Add-ons hat keine Authentifizierung oder Verschlüsselung (nur Localhost oder SSH-Tunnel). Optional `BLENDER_MCP_SAFE_MODE=1` prüft Skripte vor Ausführung (blockiert Datei-/Prozess-/Netzwerkzugriff und persistente Installationen). Das ist eine Heuristik, kein Sandboxing (ASSUMED).
- Datenschutz (RESEARCHED, README "Telemetry Control"): Standardmäßig wird ein anonymer Nutzungsdatensatz gesendet (Install-ID, Session-ID, Toolname, Erfolg, Dauer, Versionen, OS, Zeitstempel). Opt-out über `DISABLE_TELEMETRY=true`. Der Opt-in für Prompts/Code/Screenshots ist standardmäßig aus. Das README nennt eine Weiterverwendung der Daten "to train AI models" bei Opt-in. Es gibt ein kostenpflichtiges "Premium" (gehostete Generatoren). Das fällt unter C-03 (nur mit Owner-Freigabe).
- Installationsweg: `uvx mcp-for-blender` oder `pipx`, plus Blender-Add-on über `install-addon`. Der empfohlene One-Liner `curl ... | sh` von `mcp-for-blender.com` sollte nicht blind ausgeführt werden. Besser: PyPI-Paket per Version pinnen und prüfen.
- Alternativen (RESEARCHED-SEKUNDÄR, Glama-Listings/Blogs, nicht verifiziert): Offizieller Blender-Lab-MCP (`projects.blender.org/lab/blender_mcp`, v1.0.0 2026-04-27, v1.0.3 2026-09-11 laut Blogpost; Lizenz unbekannt, Host im Container gesperrt), `harveyxiacn` Fork (MIT laut Glama), Apache-2.0-Server von C. Arango, `ssoj13/blender-mcp-rs` (27 Tools, Lizenz unbekannt), `dhakalnirajan/blender-open-mcp` (Ollama). Alle vor Einsatz per LICENSE zu prüfen.
- Vereinbarkeit mit ADR-0009: MCP-Server und Blender sind externe optionale Prozesse (Blender GPL nur als Worker). MIT ist unproblematisch. Der Server ist ein Entwicklungswerkzeug, keine Produktabhängigkeit.
- Alternative ohne MCP: das PyPI-Paket `bpy` 5.2.3 (GPL-3.0, Python ==3.13.*, Linux-Wheel 403 MB) erlaubt headless Skripte, aber nur als separater Prozess (GPL).

### 4.2 VRM-Addon-for-Blender (`saturday06`)

- Lizenz: MIT OR GPL-3.0-or-later. RESEARCHED: PyPI `bpy-vrm-format` 4.10.2 (Upload 2026-10-11) gibt "SPDX-License-Identifier: MIT OR GPL-3.0-or-later" an. Die Blender-Extensions-Seite (Sekundär) nennt für 4.5.0 (2026-07-28) "MIT License" und "GPL v3 or later". Die `LICENSE`-Datei konnte unter `main` nicht per raw abgerufen werden (404, Dateiname unbekannt). Das ist offen.
- Blender-Kompatibilität: 4.2 LTS und neuer, laut Extensions-Seite Blender >= 5.3 als nicht unterstützt (Sekundär). README: Import/Export/Editing von VRM sowie Python-Scripting-API, auch als PyPI-Paket `bpy-vrm-format` (Wheel 1,6 MB, `files.pythonhosted.org` erreichbar).
- Für das Produkt: Als Referenz-/Testwerkzeug einsetzbar. Die MIT-Option macht auch eine Übernahme von Code-Konzepten lizenzseitig machbar, formell zuerst Rechtsprüfung per ADR.

### 4.3 MPFB2

- Add-on-Code GPLv3, Assets CC0 (siehe 1.1). Nutzung nur als externes Blender-Add-on (Erzeugungswerkzeug), kein Code im Produkt. Ausgaben (Exporte, Saves, Renderings) beansprucht das Team nicht (`LICENSE.md` §D).

## 5. Erreichbarkeit im Container (selbst gemessen am 2026-10-23, `curl -m 15`)

| Ziel | Ergebnis | Hinweis |
|---|---|---|
| `raw.githubusercontent.com/...` | 200 | alle unten genannten Dateien |
| `pypi.org/pypi/*/json` | 200 | |
| `files.pythonhosted.org` (konkrete Wheel-URL) | 200 | Wurzel-URL 404 ist normal; `makehuman-1.3.2` Wheel (30,5 MB) und `bpy_vrm_format-4.10.2` Wheel geladen/HEAD |
| `registry.npmjs.org` | 200 | enthält nur Pakete, keine VRM-Beispielmodelle (404 für `vrm-samples`, `@vrm-c/vrm-samples`) |
| `github.com`, `api.github.com` | 403 | |
| `cdn.jsdelivr.net`, `unpkg.com` | 000 | |
| `ghcr.io` | 301 | nicht weiter geprüft |
| `static.makehumancommunity.org`, `makehumancommunity.org`, `quaternius.itch.io`, `opengameart.org`, `kenney.nl`, `polyhaven.com`, `ambientcg.com`, `blender.org`/`download.blender.org`, `extensions.blender.org`, `projects.blender.org`, `vrm.dev`, `vrm-addon-for-blender.info`, `mcp-for-blender.com`, `booth.pm`, `astral.sh`, `tuxfamily` | 000 | nicht erreichbar (auch WebFetch nicht) |

Je Kandidat: "im Container direkt holbar"

| Kandidat | Holbar | URL / benötigter Host |
|---|---|---|
| MPFB2 `base.obj` (1,75 MB, CC0) | ja | `https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/src/mpfb/data/3dobjs/base.obj` |
| MPFB2 Rig (`rig.default.json` 126 KB, `rig.default_no_toes.json` 106 KB) | ja | `.../src/mpfb/data/rigs/standard/rig.default.json` |
| MPFB2 Targets (`*.target.gz`) | ja | `.../src/mpfb/data/targets/<gruppe>/<name>.target.gz`, Namen aus `.../targets/target.json` (160 KB). Beispiele: `head/head-age-decr.target.gz` (18 KB), `macrodetails/african-female-young.target.gz` (124 KB). Ein Teil der Pfade ist nicht erraten möglich: Meine Tests auf `expression/units/caucasian/female/mouth-smile.target.gz` ergaben 404 (Dateiname unbekannt, Verzeichnis bekannt). |
| MPFB2 LICENSE / LICENSE.ASSETS | ja | `.../LICENSE.md`, `.../LICENSE.ASSETS.md` |
| MakeHuman System Assets (Haare, Augen, Kleidung, Skins, 267 MB) | nein | Host `static.makehumancommunity.org` plus dessen Spiegel (Spiegelhosts unbekannt, ASSUMED: z. B. tuxfamily) |
| Weitere MH-Packs (hair01-03, hats, suits, Materialien) | nein | gleiche Hosts |
| Quaternius Universal Base Characters / Animation Library | nein | `quaternius.itch.io`, `quaternius.com` (ggf. itch.io-CDN, Host nicht ermittelt), alternativ `opengameart.org` |
| Blender Studio Human Base Meshes | nein | `blender.org` (Download-CDN `download.blender.org`) |
| Seed-san / Twist-Sample (VRM 1.0, VRM Public License) | ja | siehe 1.3 (raw) |
| `test.vrma` | ja | siehe 3 |
| `mcp-for-blender` 2.2.3 | ja (PyPI) | `pypi.org`, `files.pythonhosted.org` |
| `bpy-vrm-format` 4.10.2 | ja (PyPI) | gleich; zusätzlich Blender selbst nötig: `blender.org`/`download.blender.org` (nein), oder `bpy` 5.2.3 über PyPI (403 MB, Python 3.13) |
| VRM-Addon-for-Blender als Blender-Extension | nein | `extensions.blender.org` |
| Poly Haven / ambientCG Texturen | nein | `polyhaven.com`, `dl.polyhaven.org`, `ambientcg.com` |

Empfohlene Allowlist-Freigaben (minimal, in dieser Priorität):
1. `static.makehumancommunity.org` (System Assets, Hair-/Material-Packs) plus die dort verlinkten Spiegel (nach erster Freigabe die Spiegel-Hosts aus der Pack-Seite lesen).
2. `quaternius.itch.io` und `quaternius.com` (Animationen, optionale Vergleichscharaktere). Ersatzweise `opengameart.org`.
3. `download.blender.org` (und `www.blender.org`) falls Blender selbst im Container laufen soll. Alternative: `bpy` per PyPI.
4. Optional: `extensions.blender.org`, `polyhaven.com`/`dl.polyhaven.org`, `ambientcg.com`, `vrm.dev` (Lizenztext VRM Public License).
GitHub-Repos selbst (`github.com`) sind nicht nötig, solange einzelne Dateien per raw reichen. Ein Verzeichnislisting ist nur über Seiten wie `github.com/.../tree/...` möglich. Das gelang mir nur über WebFetch, nicht über den Container.

## 6. Empfohlener Testcandidate-Satz

| Slot | Asset | Lizenz | Status / Risiko |
|---|---|---|---|
| Body + Kopf/Gesicht | MPFB2/MakeHuman `base.obj` + ausgewählte Targets (Makro: Geschlecht/Alter/Ethnie; Gesicht: head, eyes, nose, mouth, chin, ears, cheek, eyebrows) + `rig.default_no_toes.json` oder `rig.default.json` | CC0-1.0 | RESEARCHED, sofort holbar. Risiko mittel: 19k Quads bedeuten Dezimierung/Proxy nötig, MH-Rig muss auf VRM-Humanoid gemappt werden, nacktes Mesh braucht Kleidung. Gesichts-Blendshapes für VRM-Expressions (Visemes, Blink) sind aus Targets/Expression-Units zu bauen (ASSUMED). |
| Haare | `hair01` (CC0, stilisiert) aus MakeHuman-Packs, 1 Frisur | CC0 | RESEARCHED-SEKUNDÄR, benötigt Allowlist (static.makehumancommunity.org). Nur CC0-Frisuren, `hair02/03` (CC-BY) erst nach Attributions-Mechanismus. Fallback: prozedurale Haar-Karte (Eigen-Asset). |
| Augen | MakeHuman-System-Assets Augen (CC0) oder eigene prozedurale Augen + Iris-Textur | CC0 | RESEARCHED-SEKUNDÄR (OpenGameArt-Hinweis), Allowlist nötig. Fallback: eigene Kugel-Augen. |
| Kleidung (1 bis 2) | 1 Oberteil + 1 Hose/Unterteil aus `makehuman_system_assets` (CC0), per `.mhclo`-Proxy auf Basismesh gefittet | CC0 | ASSUMED (Packinhalt nicht gesichtet). Fallback: eigene einfache Kleidung (Zylinder-Proxies/Boolean) als Eigen-Asset. |
| Skin-/Textur | MakeHuman-Skins (CC0) oder eigene/KI-Hauttöne | CC0 | wie oben |
| Animationen | (a) eigene prozedurale Idle-/Pose-Animation (CC0, sofort), (b) Quaternius UAL Standard (CC0) nach Freigabe, per Retargeting | CC0 | (a) kein Risiko, (b) ASSUMED retargetbar |
| Referenz/Test (nicht ausliefern) | Seed-san, Twist-Sample, `test.vrma` | VRM Public License 1.0 bzw. MIT | nur Validator-/Importtests intern; nicht ins Repo legen |
| Werkzeug | Blender (extern) + VRM-Addon-for-Blender (MIT OR GPL-3.0+) + MPFB2 (Code GPL, nur Werkzeug) + optional `mcp-for-blender` mit `DISABLE_TELEMETRY=true` und Safe Mode | MIT / GPL | Entwicklungswerkzeuge, nicht mitgeliefert |

Lizenzübersicht des Satzes: überwiegend CC0-1.0. Nur wenn CC-BY-Teile hinzukommen, ist Attribution im Credits-/VRM-Meta-Export nötig (ADR-0010). Keine NC/ND/SA/GPL-Daten.

Gesamtrisiko:
- Rechtlich niedrig bis mittel. Offen sind (1) die Dateiprüfung der MakeHuman-Packs (einzelne Dateien können CC-BY oder anders sein), (2) das Fehlen eines Dateikopfs in den Targets (Rechteklärung nur über Repo-LICENSE), (3) die MakeHuman-Historie der Basis (Herkunft vor 2020 AGPL; die CC0-Erklärung kommt vom Projekt selbst).
- Technisch mittel: Mapping MH-Rig zu VRM-Humanoid, Dezimierung, Expression-Blendshapes.
- Vorgehensvorschlag (PROPOSED): Ein Spike nach Owner-Freigabe (SP-Nummer vergeben) mit Provenienzregister je Datei (URL, Commit-Hash, SPDX, Hash, Abrufdatum). Danach Allowlist für `static.makehumancommunity.org` beantragen.

## 7. Offene Punkte / nicht belegt

- Inhalt und Dateiliste der MakeHuman-ZIPs, Zeit-/Lizenzstand der einzelnen Frisuren/Kleidung: nicht gesichtet (Host gesperrt).
- Zahlen der Quaternius-Pakete (Standard-Tier-Inhalte, Dateiformate) nur aus Suchtreffern.
- Offizieller Blender-Lab-MCP: Lizenz/Umfang unbekannt.
- `LICENSE`-Datei von VRM-Addon-for-Blender im Repo nicht gelesen (nur PyPI-SPDX und Extensions-Listing).
- VRM Public License 1.0: Volltext nicht gelesen (`vrm.dev` gesperrt), nur die Meta-Felder der Modelle.
- Expression-Unit-Targets: genaue Dateinamen im MPFB2-Repo nicht ermittelt.
