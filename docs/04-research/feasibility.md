> **Status:** Rechercheergebnis (delegierter Analyseagent, 2026-10-23), von Claude integriert, **nicht** vom Owner geprüft. [R] = RESEARCHED, [A] = ASSUMED. Empfehlungen sind PROPOSED.
> **Maßgebliche Quelle für:** VRM-/glTF-Format-Machbarkeit, Bibliotheken, Blender-Backend, Morph-/Fitting-/Rigging-Verfahren (Mandat 5, Punkte 4, 5, 9, 10).

# HASE VRM MAKER – Recherche: Formate, Geometrie, Rigging (Stand 2026-10-23)

Kennzeichnung: **[R]** = RESEARCHED (Quelle + Datum/Version), **[A]** = ASSUMED (Trainingswissen/Schlussfolgerung, nicht verifiziert).
Hinweis: Viele GitHub-Seiten zeigten per WebFetch keine Release-Daten (Releases-Sektion leer, API 403). Fehlende Versionen sind als "nicht ermittelt" markiert, nicht geraten. Alle Lizenzaussagen sind keine Rechtsberatung.

---

## 1. VRM 1.0 Spezifikation

Quelle (alle abgerufen 2026-10-23): https://github.com/vrm-c/vrm-specification (master, 904 Commits laut Seite; Repo-Lizenz auf der Seite nicht ausgewiesen, Spec-Doku-Lizenz [A] CC-BY/Creative-Commons-artig – prüfen).

Spezifikationsordner im Repo [R]: `VRMC_vrm-1.0`, `VRMC_vrm_animation-1.0`, `VRMC_springBone-1.0`, `VRMC_springBone_extended_collider-1.0`, `VRMC_springBone_limit-1.0`, `VRMC_node_constraint-1.0`, `VRMC_materials_mtoon-1.0`, `VRMC_materials_hdr_emissiveMultiplier-1.0`, `0.0`.

### 1.1 VRMC_vrm (Kern)
- [R] Datei ist `.glb` mit Endung `.vrm`; `extensions.VRMC_vrm.specVersion` = "1.0". Pflicht: `humanoid`, `meta`. Optional: `firstPerson`, `expressions`, `lookAt`. Metrische Einheiten. Companion-Extensions: KHR_materials_unlit, KHR_texture_transform, KHR_materials_emissive_strength, VRMC_materials_mtoon, VRMC_springBone, VRMC_node_constraint. glTF-Animationen und Kameras werden im VRM-Modell nicht genutzt. (…/VRMC_vrm-1.0/README.md)
- [R] Morph-Target-Namen stehen in `meshes[*].extras.targetNames`. Tangenten dürfen fehlen (MikkTSpace beim Import).
- [R] Update-Reihenfolge: Humanoid → LookAt → Expressions → Constraints → Spring Bones.

### 1.2 Humanoid (humanoid.md)
- [R] **Pflicht-Bones (15):** hips, spine, head, leftUpperLeg, leftLowerLeg, leftFoot, rightUpperLeg, rightLowerLeg, rightFoot, leftUpperArm, leftLowerArm, leftHand, rightUpperArm, rightLowerArm, rightHand.
- [R] **Optional:** chest, upperChest, neck, leftShoulder/rightShoulder, leftEye/rightEye, jaw, leftToes/rightToes, alle Finger (je 3 Gelenke; Daumen metacarpal/proximal/distal). Chest/Neck waren in 0.x Pflicht; 1.0-Status aus der Extraktion unklar – [A] in 1.0 optional, am Originaldokument verifizieren.
- [R] Hierarchie fest: Hips = Wurzel; fehlt ein optionaler Parent, wird zum Parent-des-Parents gesprungen; upperChest nur mit chest. Jeder Humanoid-Bone eindeutig; Skalierung positiv; "inverted pelvis" deprecated.
- [R] **Nicht-humanoide Nodes dürfen zwischen Humanoid-Bones liegen** (z. B. zwischen Ober-/Unterschenkel). Ob zusätzliche Bones an beliebigen Stellen erlaubt sind, nennt die Seite nicht explizit.
- [A] Es gibt genau ein `humanoid`-Objekt je Datei, also genau ein Humanoid-Mapping. Ein Vierbeiner/Nicht-Humanoid ist als VRM-1.0-konform nicht darstellbar (die 15 Pflicht-Bones erzwingen Zwei-Arm/Zwei-Bein-Topologie). Katzen-/Hunde-Demi (humanoider Körper + Tierohren/Schwanz) ist dagegen konform.

### 1.3 Meta / Lizenzfelder (meta.md)
- [R] Pflicht: `name`, `authors`, `licenseUrl` (Standard `https://vrm.dev/licenses/1.0/`). Weitere: version, copyrightInformation, contactInformation, references, thirdPartyLicenses, thumbnailImage, otherLicenseUrl.
- [R] Enums: `avatarPermission` (onlyAuthor [Default] / onlySeparatelyLicensedPerson / everyone); `commercialUsage` (personalNonProfit [Default] / personalProfit / corporation); `creditNotation` (required [Default] / unnecessary); `modification` (prohibited [Default] / allowModification / allowModificationRedistribution); Booleans, alle Default false: allowExcessivelyViolentUsage, allowExcessivelySexualUsage, allowPoliticalOrReligiousUsage, allowAntisocialOrHateUsage, allowRedistribution.
- Konsequenz [A]: Der Editor braucht ein Pflicht-Metadatenformular und muss Lizenzen von Fremd-Assets (Kleidung, Texturen) in `thirdPartyLicenses` aggregieren; restriktivste Asset-Lizenz sollte Export-Defaults begrenzen.

### 1.4 Expressions (expressions.md)
- [R] Presets (alle optional): happy, angry, sad, relaxed, surprised; aa, ih, ou, ee, oh; blink, blinkLeft, blinkRight; lookUp/Down/Left/Right; neutral. Custom in `expressions.custom` (kein Preset-Name).
- [R] Binds: morphTargetBinds (node, index, weight 0–1), materialColorBinds, textureTransformBinds. `isBinary`, Override-Typen none/block/blend für Mund/Blink/LookAt.
- Konsequenz [A]: Ohren-/Schwanz-Posen oder Tier-Gesichtsvarianten können als Custom Expressions über Morph-Targets laufen; Morph-Targets sind bei VRM also das Standardmittel für Gesicht UND "Blendshape-Varianten", aber Körper-Morphs (Species-Parameter) sollten **eingebacken** (baked) werden, nicht als Expressions exportiert (Expressions werden live von Runtimes überschrieben).

### 1.5 LookAt (lookAt.md)
- [R] Typen `bone` (Augenbones, Euler) oder `expression` (lookUp/Down/Left/Right-Expressions); `offsetFromHeadBone` (Beispiel [0,0.06,0]); vier Range-Maps (horizontalInner/Outer, verticalDown/Up) mit inputMaxValue/outputScale.
- Konsequenz [A]: Tieraugen (seitlich/schlitzförmig) = Expression-Typ ist robuster als Bone-Typ.

### 1.6 VRMC_springBone 1.0 (+Erweiterungen)
- [R] Struktur: `colliders` (Sphere/Capsule, an `node`), `colliderGroups`, `springs` (Chain aus Joints; jeder Joint Vorfahre des nächsten; optional `center`). Joint-Parameter u. a. stiffness ≥ 0, dragForce 0–1, hitRadius (m), gravity. Joint in nur einer Chain; **verzweigte Chains sind undefiniert** (Runtime darf Äste als getrennte Chains behandeln).
- [R] `VRMC_springBone_extended_collider` 1.0: Inside-Sphere, Inside-Capsule, Plane; zwingend ein Fallback-Basiscollider (z. B. Radius-0-Sphere bei [0,-10000,0]).
- [R] `VRMC_springBone_limit` 1.0 (Status "Complete"): cone/hinge/spherical-Limits (Winkel in rad, yaw ≤ π/2); nicht am letzten Joint der Chain.
- Konsequenz Katzen-/Hunde-Demi [A]: Ohren und Schwanz = Extra-Nodes unter head bzw. hips/spine, **keine** Humanoid-Bones, als Spring-Chains (lineare Ketten, nicht verzweigt, ggf. mehrere Ketten pro Ohr statt Baum). Schwanz-Sway mit Limit-Extension (Runtime-Support uneinheitlich → Fallback ohne Limit akzeptieren). Collider an Körper/Kopf zur Vermeidung von Durchdringung; Inside-Collider/Plane nur mit Fallback.

### 1.7 VRMC_node_constraint 1.0
- [R] Auf einem Node; genau eines aus `roll` (source, rollAxis X/Y/Z), `aim` (source, aimAxis ±X/Y/Z), `rotation` (source); `weight` 0–1 (Default 1); keine Selbstreferenz, keine Zyklen.
- Konsequenz [A]: Nutzbar für Twist-Bones (roll), Ohr-Mitbewegung mit Kopf (rotation, kleiner weight), Augen-Aim. Von Runtimes teils unvollständig unterstützt → in Kompatibilitätstests prüfen.

### 1.8 VRMC_materials_mtoon 1.0
- [R] Einzige Pflicht: `specVersion` "1.0". Parameter u. a. shadeColorFactor, shadingShiftFactor (0.0), shadingToonyFactor (0.9), giEqualizationFactor (0.9), Rim (matcap, parametricRim*), Outline (outlineWidthMode none/worldCoordinates/screenCoordinates, outlineWidthFactor, outlineColorFactor), UV-Animation. Vertex Colors ignoriert; Outlines immer Front-Face-Culling. Erlaubte Fallbacks: Outline weglassen, Alpha-Blend vereinfachen. Spec hat interne Inkonsistenzen (z. B. `shadingToonyMultiplyTexture` referenziert, aber nicht definiert).
- Konsequenz [A]: Materialsystem des Studios sollte MToon-Parameter 1:1 abbilden (PBR → MToon-Konversion nur für Preview).

### 1.9 VRMC_vrm_animation (VRMA) 1.0
- [R] Eigene Animations-glTF (nicht für VRM-Modelle selbst). Pflicht: `specVersion`; `humanoid.humanBones` (Pflicht-Bones müssen enthalten sein); leftEye/rightEye nicht erlaubt (→ lookAt); Rest-Pose = "VRM T-pose"; keine Skalierungsanimation; nur Hips darf Translation haben; Expressions über X-Translation eines Nodes (0–1), Preset lookX nicht definierbar; lookAt-Node mit Rotation (ZXY-Euler); Empfehlung 30 FPS; erste Animation wird geladen.
- Konsequenz [A]: VRMA ist rein humanoid (Hips-Translation, nur Humanoid-Bones). **Ohren-/Schwanzanimation ist in VRMA nicht abbildbar** → läuft über Spring Bones/Constraints oder Expressions. Retargeting-Schicht muss Proportionsunterschiede (Hips-Translation skalieren) selbst lösen.

---

## 2. Bibliotheken

| Bereich | Lib | Lizenz | Version/Datum | Quelle |
|---|---|---|---|---|
| glTF Rust | gltf (gltf-rs) | MIT OR Apache-2.0 | 1.4.1, zuletzt aktualisiert 2024-05-10 (crates.io-API) – wirkt wenig aktiv [A] | https://crates.io/api/v1/crates/gltf |
| glTF C | cgltf | MIT | Version nicht ermittelt; Loader+Writer, Morph Targets, Skins | https://github.com/jkuhlmann/cgltf |
| glTF C++ | fastgltf | MIT (simdjson/simdutf MIT) | Version nicht ermittelt; v0.9.x = C++17, neuer = C++20 | https://github.com/spnda/fastgltf |
| glTF C/C++ | tinygltf | MIT | Version nicht ermittelt; "v3" in C11, v1/v2 in `attic/` | https://github.com/syoyo/tinygltf |
| glTF JS/TS | glTF-Transform | MIT | Version nicht ermittelt; CLI inkl. Draco/Meshopt/KTX2; VRM-Support nicht erwähnt | https://github.com/donmccurdy/glTF-Transform |
| VRM Web | three-vrm | MIT | v3.5.5 (Juli, "Latest"); v3.6.0-beta.0 (29. Sep) mit VRMC_springBone_limit | https://github.com/pixiv/three-vrm/releases |
| VRM Unity | UniVRM | Lizenz auf Seite nicht ausgewiesen ([A] MIT) | v0.131.4 (19. Okt, Unity 2022.3 LTS+) | https://github.com/vrm-c/UniVRM/releases |
| VRM Godot | godot-vrm (V-Sekai) | nicht ermittelt ([A] MIT) | Import+Export bis VRM 1.0, Godot 4.0+ | https://github.com/V-Sekai/godot-vrm |
| VRM Rust | vrm_spec, serde_vrm, gltf_kun_vrm, bevy_vrm 0.4.0 (Bevy 0.19) | nicht ermittelt | Abdeckung/Wartung nicht geprüft | https://docs.rs/crate/bevy_vrm/latest , https://docs.rs/vrm-spec |
| VRM C++ | – | – | Keine etablierte C++-VRM-Lib gefunden/geprüft; Bau auf fastgltf/cgltf + eigenes JSON-Schema [A] | – |
| FBX | ufbx | MIT oder Public Domain (Wahl) | tag v0.23.1, 2026-09-25 | https://github.com/ufbx/ufbx |
| FBX | Autodesk FBX SDK | proprietär | Zitat aus Forum: "cannot be redistributed or repackaged without written permission" – Sekundärquelle, unbestätigt | https://forums.autodesk.com/t5/fbx-forum/about-fbx-sdk-2020-license/m-p/11655924 |
| Import-Hub | Assimp | modifizierte 3-Klausel-BSD | Version nicht ermittelt | https://github.com/assimp/assimp |
| Animation | ozz-animation | MIT | Version nicht ermittelt; Runtime Sampling/Blending; Retargeting/IK nicht erwähnt | https://github.com/guillaumeblanc/ozz-animation |
| Mesh-Opt | meshoptimizer | MIT | v1.3 (25. Sep 2026) | https://github.com/zeux/meshoptimizer/releases |
| Texturen | Basis Universal (KTX2) | Apache-2.0 | README "v2.5" (Stand 2026) | https://github.com/BinomialLLC/basis_universal |
| Geometrie | libigl | MPL-2.0 (Repo enthält auch GPL-3.0-Datei) | v2.5.0 | https://github.com/libigl/libigl |
| Geometrie | geometry-central | MIT | Version nicht ermittelt | https://github.com/nmwsharp/geometry-central |
| Geometrie | CGAL | Duale Lizenz: Basis LGPL, höhere Pakete GPL, kommerziell via GeometryFactory | 6.1 (Okt 2025, Wikipedia-Angabe); 6.0.1 Paketmetadaten "LGPL-3.0+ AND GPL-3.0+ AND BSL-1.0" | https://doc.cgal.org/6.1/Manual/license.html |
| Geometrie | OpenMesh | [A] BSD-3-Clause (nicht geprüft) | – | – |
| BVH-Parser | – | – | Nicht recherchiert; BVH ist trivial parsebar → [A] Eigenbau (<500 Zeilen) | – |

Bewertung:
- [A] Für einen **Windows-Desktop-Kern in C++ oder Rust** ist ein permissiver Stack (cgltf/fastgltf + ufbx + meshoptimizer + ozz + Eigen + geometry-central/libigl-Algorithmen) lizenzseitig unkritisch. **Vermeiden:** FBX SDK (Redistribution), CGAL-GPL-Pakete (Copyleft), libigl nur MPL (dateibasiert, ok, aber GPL-Teile separat prüfen).
- [R] gltf-rs: letzte crates.io-Aktualisierung 2024-05 → Wartungsrisiko; VRM-Extension-Daten müssten über `extras/extensions` JSON selbst gehandhabt werden. [A] glTF-Daten + VRM-Extension-JSON mit eigenem serde-Modell halten ist robuster.
- [R] Keine der geprüften Libs liefert Retargeting/Rig-Fitting; ozz und die Loader sind reine Container/Runtime. Referenzimplementierungen für VRM-Export: VRM-Addon-for-Blender (siehe 3), UniVRM, godot-vrm (Export vorhanden).

---

## 3. Blender als Backend

- **Version [R]:** PyPI `bpy` 5.2.3, veröffentlicht 2026-10-22, benötigt exakt Python 3.13, Lizenz GPL-3.0, Wheels für Windows x64/ARM64, Linux glibc 2.28+, macOS ARM64 (https://pypi.org/project/bpy/). Blender-Doku/Website waren nicht abrufbar (DNS) – Blender-Release-Zyklus/LTS-Daten daher nicht verifiziert.
- **Lizenz [R]:** Blender: nur GPL ("Apart from the GNU GPL, Blender is not available under other licenses", COPYING, https://github.com/blender/blender/blob/main/COPYING).
- **Output [R, Sekundärquelle]:** Blender-Foundation-FAQ (2004, Mailingliste) und Ton Roosendaal (2009): Alles, was man mit Blender erzeugt (Exporte, .blend), gehört dem Nutzer; Ausnahme: in Standalone-Executables eingebettete Daten. Offizielle aktuelle FAQ nicht abrufbar → vor Release rechtlich prüfen. https://archive.blender.org/lists/bf-committers/2004-November/008253.html
- **Prozessgrenze [R/A]:** Die 2009-Diskussion nennt Verknüpfung (Python-API/Linking) mit nicht-GPL-Code als GPL-auslösend; proprietäre Add-ons gelten nur als unproblematisch, wenn sie außerhalb Blenders ohne Blender-Source/API-Aufrufe arbeiten. [A] Konsequenz:
  - **`import bpy` im eigenen Hauptprozess = GPL-Ableitung des gesamten Prozesses → vermeiden**, außer HASE VRM MAKER selbst wird GPL-3.
  - **Externer Prozess** (`blender --background --python script.py` oder separater Worker mit `bpy`), Kommunikation über Dateien/JSON/stdio/Pipe, ist die übliche "arm's-length"-Trennung. Eigene Worker-Skripte, die `bpy` aufrufen, sind selbst GPL-pflichtig bei Weitergabe [A] → als separate, GPL-lizenzierte "Blender-Plugin"-Komponente ausliefern; Blender selbst nicht mitliefern oder mit GPL-Quellangebot.
  - Blender nur **optional** installierbar machen (Pfad-Erkennung); Kernpfad ohne Blender funktionsfähig.
- **Headless [R/A]:** `bpy` als Modul ist offiziell vorgesehen (PyPI-Wheel existiert); Doku-Seite war nicht abrufbar. [A] `--background` Modus ist gängige Praxis; GPU-lose Ausführung möglich; Python-Version an Blender-Release gebunden (Instabilität bei Versionssprüngen).
- **VRM-Addon-for-Blender [R]:** https://github.com/saturday06/VRM-Addon-for-Blender – Lizenzen "MIT, GPL-3.0 found" (Repo enthält beide Dateien; welcher Teil unter welcher gilt, nicht ermittelt); aktuell v4.10.2 (2026-10-11); Blender 2.93 bis 5.2 (4.2+ und 2.93–4.1 getrennt); bietet Python-API für Automatisierung; PyPI-Paket `bpy-vrm-format` vorhanden; 4.10.0 enthält "experimental support spring bone limits". Ob alle 1.0-Extensions (node_constraint, MToon vollständig) unterstützt werden, ging aus der Seite nicht hervor.
- Konsequenz [A]: Blender-Pfad ist gut für **Referenz-/Fallback-Export, Retopo/Weight-Painting-Automation, Bake von Shape Keys, Batch-Tests**. Als Pflichtpfad ungeeignet (Größe ~400 MB+, Versions-Python-Kopplung, GPL).

---

## 4. Morph-/Deformationsverfahren (speziesübergreifend) und Kleidungs-Fitting

| Verfahren | Idee | Quelle | OSS-Machbarkeit | Risiko |
|---|---|---|---|---|
| Blendshape/Delta-Morphs | Basismesh + additive Vertex-Deltas, Gewichte linear | VRM nutzt morph targets (Spec s. o.) | Sehr hoch, trivial, GPU-tauglich | Niedrig; kombinatorische Artefakte bei extremen Mischungen, braucht gleiche Topologie |
| Cage-basiert (Mean-Value-Coordinates) | Hüllgitter, Koordinaten relativ zum Käfig | MVC (Ju et al. 2005) [A], nicht neu recherchiert | Hoch; Eigenbau oder libigl | Mittel; Käfig-Erstellung pro Spezies manuell |
| Bounded Biharmonic Weights | Glatte, lokalisierte Handle-Gewichte (Punkte/Bones/Cages) | Jacobson et al., SIGGRAPH 2011, https://www.alecjacobson.com/weblog/1784.html | Mittel; libigl-Implementierung [A], Optimierung (QP) nötig | Mittel; Rechenzeit, Tuning |
| RBF-Deformation | Landmark-Verschiebung → glatte Interpolation (Thin-Plate/Biharmonisch) | [A] Standardtechnik | Hoch (dichte Löser bei <2000 Landmarken) | Niedrig–mittel; Landmarken-Korrespondenz je Spezies |
| Deformation Transfer | Triangle-Deformation-Gradients übertragen, Marker-Korrespondenz, vorfaktorisiertes LGS | Sumner & Popović 2004, https://people.csail.mit.edu/sumner/research/deftransfer | Mittel–hoch (Sparse-Solver) | Mittel; Marker-Setup, keine Topologiegleichheit nötig |
| Delta Mush | Glättungs-Korrektur nach Skinning, Details erhalten | Mancewicz/Derksen/Wilson SIGGRAPH 2014 (Autodesk Maya-Doku) | Hoch als Preview/Bake; Patent-/Lizenzstatus [A] nicht geprüft | Mittel; **nicht VRM-exportierbar** (nur Bake) |
| Skinning-Weight-Transfer (nearest surface) | Gewichte vom Proxy/Body auf Kleidung | [A] Standard in Blender "Data Transfer" | Hoch | Mittel; Fehler an Falten/Überlappungen, braucht Nachglättung |
| Bone Heat / Auto-Skinning | Heat-Diffusion-Gewichte | Pinocchio-Idee [A]; Blender intern | Mittel (Eigenbau Laplace-Solver) | Mittel |
| Proxy-/Wrap-Fitting (MakeHuman/MPFB2) | Kleidung referenziert Basismesh-Vertices (Barycentrics); folgt Morphs | https://www.cgchannel.com/2025/03/check-out-open-source-blender-character-generation-plugin-mpfb-2 ; Lizenz https://static.makehumancommunity.org/about/license.html | Hoch, wenn Kleidung auf Base-Topologie "gebunden" authored wird | Niedrig bei hauseigenen Assets; hoch bei Fremdkleidung |
| Neuronales Rigging | RigNet: Skelett+Skinning vorhersagen | Xu et al. 2020, arXiv:2005.00559 (Code-Lizenz unklar: Fork nennt "dual-license", Mirror GPL-3.0 – ungeklärt) | Niedrig (Forschungs-Code, Lizenz, Qualität) | Hoch |

[R] MPFB2: Quellcode GPL, mitgelieferte Assets CC0 (MakeHuman-Lizenzseite, abgerufen 2026-10-23); Status der Fitting-Proxies nach Lizenzänderung 2020: ältere Doku führte Targets/Proxies/Basismesh als "Source/AGPL" – für heutige Dateien Header prüfen.
[A] Character Creator / Marvelous Designer: proprietär, keine Algorithmen-Quelle recherchiert; ihre Praxis (Basismesh-gebundene Kleidung + Auto-Skin-Transfer + Simulation) deckt sich mit den Wrap-Ansätzen.

**Empfehlung (A):** 
1. Körperform: Basismesh-pro-Archetype + Delta-Morphs (Shape Keys), optional RBF-Landmark-Layer. 
2. Kleidung: **Basismesh-referenzierte Barycentric-Bindung** + Offset entlang Normalen (Proxy-Fitting), danach Laplace-Glättung/Kollisions-Push (Kleidung außerhalb Körper). Weights per nearest-surface-Transfer + Glättung.
3. Species-übergreifend (Katze/Hund): gemeinsames Topologie-Template mit "Ohr-/Schwanz-Regionen" als Zusatz-Patches; vollständig andere Topologien (Phase 2) über Deformation Transfer oder Cage-MVC.
4. Delta Mush nur als optionaler Bake-/Preview-Schritt.

---

## 5. Direkte 3D-Manipulation → Parameter

- [R] **Direct Manipulation Blendshapes** (Lewis & Anjyo, IEEE CG&A 30(4), 2010): Pin-and-drag, Blendshape-Gewichte per Pseudo-Inversen/linearen Systemen mit Regularisierung gegen unerwünschte Änderungen; Folgearbeit: Cetinaslan, Lewis, Orvalho 2017 "Transposition Based Blendshape Direct Manipulation" (https://scribblethink.org/Work/DirectManipBlendshapes/directmanip.html, https://www.scitepress.org/papers/2017/61319/61319.pdf). Direkt anwendbar auf **Körper-Slider-Morphs**: Handle ziehen → NNLS/Tikhonov-gedämpftes Least-Squares auf Gewichte w, Constraint 0≤w≤1.
- [A] Ansätze nach Aufwand:
  1. **Least-Squares/Tikhonov** auf Morph-Gewichte (Matrix = Delta-Spalten an den Handle-Vertices). Aufwand gering, Echtzeit bei <100 Morphs.
  2. **Constraint-basiert/IK-Stil**: Landmark-Ziele plus Symmetrie-/Proportions-Constraints (Projected Gauss-Newton). Mittel.
  3. **FFD/Handles → danach Morph-Fit**: Freie Deformation per BBW/Cage, dann die Deformation als Morph-Gewichte projizieren (Residuum als "Sculpt-Korrektiv-Morph" speichern). Mittel–hoch.
  4. **Gelernt aus Daten** (PCA/statistisches Körpermodell, z. B. SMPL-artig): benötigt Trainingsdaten und Lizenzen; für Anime/Species-offen [A] nicht realistisch zum Start.
- Risiko: Ungleichmäßige Wirkung, nicht-orthogonale Morphs → instabile Lösungen; Mitigation: Regularisierung + Slider-Rückmeldung (Gewicht-Delta-Anzeige).

---

## 6. Rigging über Anatomien

- [R] **Unreal IK Retargeter**: Chain-Mapping statt Bone-zu-Bone; Chains pro Rig definiert, unterschiedliche Bone-Zahl/Namen/Orientierung ok; optionale IK für Hand-/Fußkontakte; Auto-Mapping per Fuzzy-String-Suche (`auto_map_chains`) – https://dev.epicgames.com/documentation/unreal-engine/ik-rig-animation-retargeting-in-unreal-engine . Mixamo/Rokoko: nicht recherchiert; [A] beide nutzen Standard-Humanoid-Mapping + Rest-Pose-Normalisierung.
- [A] Retargeting-Pipeline fürs Projekt: (1) Rest-Pose beider Rigs in gemeinsame T-Pose normalisieren (VRMA verlangt T-Pose, [R]); (2) **Chain-Mapping** (Humanoid-Pflicht-Chains + optionale Fingerketten); (3) lokale Rotationen übertragen (rest-relativ), Hips-Translation um Beinlängenverhältnis skalieren; (4) optional IK-Pass für Füße/Hände; (5) Extra-Chains (Ohr/Schwanz) **nicht** retargeten, sondern Spring Bones.
- [A] **Automatisches Rig-Fitting bei Morphs**: Joints aus Landmarken ableiten: jedes Gelenk = gewichteter Schwerpunkt/Offset zu Landmark-Vertices (z. B. Knie = Mittelpunkt der Kniescheiben-Ringschnitte) oder als zusätzliche **Delta-Targets für Joint-Positionen** (Joint-Positionen werden mit den Vertex-Morphs mitgeblendet – exakt wie MakeHuman/MPFB-Skelette an Basismesh-Vertices gebunden sind [A]). Danach: Bone-Rolls neu berechnen, Skin-Weights bleiben (gleiche Topologie) oder werden neu übertragen. Robuster als geometrische Auto-Rigger (Pinocchio/RigNet [A] für Fremdmeshes, Qualität fraglich).
- Nicht-humanoide Skelette (Phase 2) [A]: VRM-Export nicht möglich; Alternativen: glTF ohne VRM, eigenes Format. Architektur sollte "Export-Profile" trennen (VRM-Humanoid-Profil vs. generisches glTF).

---

## 7. Risiken & Spike-Vorschläge mit messbaren Kriterien

| # | Risiko | Spike | Messbares Kriterium (bestanden wenn …) |
|---|---|---|---|
| R1 | VRM-1.0-Export-Konformität/Runtime-Kompatibilität | Minimaler Exporter: Humanoid + Meta + Expressions + SpringBone + MToon aus Code; Test in three-vrm 3.5.5, UniVRM 0.131.4, VRM-Addon, godot-vrm | Datei besteht JSON-Schema-Validierung der Spec-Repo-Schemas [A: Schemata im Repo vorhanden, prüfen]; lädt in ≥3 von 4 Runtimes ohne Fehler; Humanoid-Retarget einer VRMA funktioniert |
| R2 | Katzen-/Hunde-Ohren/-Schwanz als Spring-Chains | Demi-Modell mit 2 Ohr-Chains (je 3 Joints) + Schwanz (6 Joints) + Limit-Extension mit Fallback | Keine Mesh-Durchdringung Kopf/Körper in 60-Sek-Idle-Animation in ≥2 Runtimes; ohne Limit-Support gleiche Datei weiterhin ladbar; Frame-Time-Overhead < 0,5 ms/Modell |
| R3 | Morph-Qualität & Echtzeit | Basismesh + 20 Delta-Morphs + RBF-Landmarks, GPU/CPU-Blend | Gewichtsänderung → sichtbares Update ≤ 16 ms bei 50k Vertices; Extrem-Kombination (3 Morphs 1.0) ohne Selbstüberschneidung (Check: 0 invertierte Dreiecke > 0,1 %) |
| R4 | Kleidungs-Auto-Fit | Proxy-Bindung + Normalen-Offset + Glättung auf 5 Kleidungsstücke × 10 Körper-Parameterextreme | ≥ 95 % Kleidungsvertices außerhalb Körper-Mesh (Signed-Distance ≥ 0); max. Durchdringung ≤ 2 mm; Fit-Zeit ≤ 2 s |
| R5 | Skinning-Weight-Transfer | Nearest-Surface + Glättung vs. Blender Data Transfer als Referenz | Mittlere L1-Abweichung der Weights zu Referenz (manuell gemalt) ≤ 0,05; 0 Vertices mit Weight-Summe ≠ 1 (±1e-4); ≤ 4 Influences/Vertex (VRM-Runtime-Limit [A]) |
| R6 | Direct Manipulation | Pin-and-Drag (Tikhonov-NNLS) auf 20 Morphs | Handle-Zielfehler ≤ 2 mm nach 3 Iterationen bei erreichbaren Zielen; Lösungszeit ≤ 10 ms; Gewichte immer in [0,1] |
| R7 | Retargeting/Rig-Fitting | Humanoid→Humanoid mit unterschiedlichen Proportionen, VRMA aus Mixamo-ähnlichem Clip | Fußgleiten ≤ 2 cm/Zyklus mit IK-Pass; Pose-Fehler Gelenke (mittlerer Winkelfehler) ≤ 5° gegen Referenz in Blender |
| R8 | Blender-Prozessgrenze/Lizenz | Worker-Prozess mit CLI, nur Datei-/JSON-I/O; Distribution ohne Blender | Hauptanwendung enthält 0 GPL-Linkage (Dependency-Audit, z. B. SBOM-Scan 0 GPL-Komponenten im Hauptprozess); Blender-Pfad optional abschaltbar, Kern-Export funktioniert ohne Blender; juristische Freigabe dokumentiert |
| R9 | Bibliotheks-Wartung/Lizenz | Dependency-Matrix (Abschnitt 2) automatisieren | 100 % Dependencies mit SPDX-Lizenz erfasst; 0 Dependencies außerhalb {MIT, BSD, Apache-2.0, Zlib, MPL-2.0, Unlicense} im Kernpfad; letzte Release/Commit ≤ 24 Monate (gltf-rs wäre Ausnahme-Kandidat) |
| R10 | Asset-Lizenz-Aggregation in VRM-Meta | Export-Flow mit `thirdPartyLicenses` | Jedes Fremd-Asset im Export hat Lizenzeintrag; Export blockiert, wenn restriktivste `modification`/`commercialUsage`-Kombination verletzt würde |
| R11 | Nicht-humanoid (Phase 2) | Papierspike: Export-Profil-Abstraktion (VRM-Profil vs. generisches glTF) | Architektur-Review: Species-Archetype-Definition enthält keine hartkodierten Humanoid-Annahmen außerhalb des VRM-Export-Moduls |

Offene Recherchelücken: Blender-Release-/LTS-Daten und offizielle Python-API-Doku (Host nicht erreichbar); Release-Daten der meisten C/C++-Libs; UniVRM-/godot-vrm-Lizenz; OpenMesh; BVH-Libs; Mixamo/Rokoko-Details; 1.0-Status von chest/neck; Lizenz der Spezifikationstexte.
