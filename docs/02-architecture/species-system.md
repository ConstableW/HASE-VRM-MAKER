# Species-System und Body Archetypes

**Maßgebliche Quelle für die Ebenen Body Archetype / Species / Character und die Topologie-Entscheidung.** Status: `PROPOSED` (ADR-0004), Beleg durch lauffähige Beispiele (`examples/species/`, `tools/validate/validate_packages.py`) und den Testcandidate (`content/testcandidate/`). Anforderungen: REQ-ARCH-001…007, REQ-PKG-002/-003, REQ-CLO-011, OD-005…009.

## 1. Drei Ebenen, nicht verdrahtet

| Ebene | Was sie festlegt | Verkörpert als | Beispiel (Testcandidate) |
|---|---|---|---|
| **Body Archetype / Body Family** | Topologie, Skeleton, Rig-Profil, Skinning-Gewichte, Morph-Parameter, Slots, Fit-Methoden, Export-Fähigkeiten | **Body Package** (`type: body`) mit einer *Schnittstelle* (`provides.interface` + Version + Capabilities) | `org.hase.body.humanoid-mh08` (19.158-Vertex-Quad-Mesh, 53-Bone-Rig, 609 Targets) |
| **Species** | Parameter-Startwerte und -Grenzen, Trait-Bündel, automatische Komponenten (Ohren, Schwanz), Presets, zusätzliche Capabilities | **Species Package** (`type: species`), *zeigt* auf eine Body-Schnittstelle (`targets.bodyInterface`) | `human`, `elf`, `cat-demi`, `dog-demi` |
| **Character** | Individuelle Werte: Parameter, Asset-Instanzen, Materialüberschreibungen, Delta-Layer, Expressions, Lizenz-/VRM-Meta | **Projektdatei** `character.hase.json` mit Referenzen + Lockfile | `examples/species/character.example.hase.json` |

Keine Ebene kennt die andere zur Compile-Zeit. Sie sind Daten, die zur Laufzeit über Referenzen und Schnittstellenverträge verbunden werden (REQ-ARCH-001, -003).

## 2. Komposition statt Vererbung

- Eine Spezies ist **keine Unterklasse** eines Körpers. Sie ist ein *Overlay* über die Parameter-/Slot-Welt des Body-Packages.
- Wiederverwendung läuft über **Traits** (benannte Parameterbündel, z. B. „Katzenaugen“) und **Komponenten** (Assets, die eine Spezies automatisch anbringt). Eine Spezies, die „wie X, aber …“ ist, nutzt X‘s Traits und Komponenten, statt X zu erben (Dependencies statt `extends`).
- Neue Spezies = neues Package. **Keine Kernänderung** nötig (REQ-PKG-006), solange sie auf eine vorhandene Body-Schnittstelle zielt.

## 3. Wann reicht eine gemeinsame Topologie, wann braucht es einen eigenen Archetype? (PROPOSED)

Der Kern unterscheidet drei **Kompatibilitätsstufen** zwischen zwei Körpern, gemessen an der Topologie-Familie (`body.topology.family`):

| Stufe | Definition | Übertragbar ohne Zusatzarbeit | Beispiel |
|---|---|---|---|
| **T0 – identisch** | gleiche Vertex-Anzahl/-Reihenfolge/UVs (gleiche Familie) | Morph-Deltas, Skin-Weights, Basismesh-gebundene Kleidung/Haare, Expressions | Mensch ↔ Elf ↔ Katzen-/Hunde-Demi (alle `mh08`) |
| **T1 – erweitert** | Basis-Indices unverändert, zusätzliche Vertices/Patches angehängt (z. B. Schnauze, Hörner) | alle T0-Daten für die gemeinsame Teilmenge; Zusatz-Parameter über `morphset`-Assets | Hunde-Demi mit Schnauzen-Patch |
| **T2 – disjunkt** | andere Topologie und/oder anderes Skeleton | nichts direkt; braucht Korrespondenz (Landmarken, Deformation Transfer, Cage) | Vierbeiner, Wesen mit zusätzlichen Gliedmaßen |

**Entscheidungsregel – eigener Body Archetype ist nötig, wenn mindestens eines zutrifft:**
1. Vertex-Anzahl/-Reihenfolge der Basis lässt sich nicht als T0/T1 ausdrücken (T2).
2. Das Skeleton unterscheidet sich in der **Topologie** (andere Anzahl/Hierarchie von Hauptbones), nicht nur in Proportionen oder zusätzlichen Spring-Bones.
3. Gliedmaßenanzahl oder grundsätzliche Anatomie ändert sich (VRM-Humanoid-Mapping nicht mehr vollständig, siehe `export-pipeline.md`, ADR-0003).
4. Die Fit-/Weight-Transfer-Qualität liegt in Messung (SP-01/-02) unter dem Zielwert.

**Nicht** nötig sind eigene Archetypes für: abweichende Beinproportionen (digitigrade durch Morphs, weil Gelenke aus Vertex-Würfeln des gemorphten Körpers abgeleitet werden – im Testcandidate bereits so umgesetzt), Tierohren/-schwänze (Bodyparts mit Spring-Chains), Körpergröße, Muskulatur, Alter, Geschlecht.

## 4. Schnittstellenvertrag Body ↔ Rest

Ein Body Package veröffentlicht:
- `provides.interface` + `version` (SemVer der *Schnittstelle*, getrennt von der Package-Version),
- `capabilities` (z. B. `skeleton:ue-mannequin-53`, `humanoid:vrm1-complete`, `fit:mhclo`, `spring-extras`),
- Parameter (`mh:…`, Namespace + Name; Typ `signed-pair`, `unit` oder `macro`; Symmetrie-Metadaten),
- Slots (`hair`, `clothing.top`, `head.ears`, …) mit Exklusivität, Layer und Anker-Bone,
- Skeleton-Profil und VRM-Humanoid-Mapping,
- Fit-Methoden (`mhclo-barycentric`, `weight-transfer`, `rigid-attach`).

Spezies und Assets deklarieren `targets.bodyInterface: {id, range}`. **MAJOR-Änderung der Schnittstelle** (Parameter entfernt/umbenannt ohne Migration, Topologie- oder Skeleton-Wechsel) bricht alle Abhängigen kontrolliert – der Resolver meldet das, bevor etwas geladen wird (`compatibility-rules.md`).

## 5. Mischformen (REQ-ARCH-005)

- *Menschliches Gesicht + Tierohren:* Spezies `cat-demi` = Menschen-Parameter + Komponente `cat-ears` (Slot `head.ears`, zwei Spring-Chains).
- *Stärker anthropomorph (Schnauze, Pfoten):* zusätzliche Morph-Targets als `morphset`-Asset auf T1-Basis (`muzzle-dog`); Pfoten/Beinproportionen über Body-Parameter und Traits; Hände/Füße als `bodypart`, falls die Basis-Topologie nicht reicht.
- *Mehrere Spezies-Merkmale kombinieren:* über Traits mehrerer Spezies-Packages (`uses`/Dependencies, ADR-0004 offene Detailfrage) – keine Mehrfachvererbung.

## 6. Grenzen und offene Punkte

- **Nicht-Humanoide** sind nur über ein anderes Body Package mit anderem Export-Profil möglich; VRM 1.0 deckt sie nicht ab (`../04-research/feasibility.md` §1.2).
- Die Beispiele `cat-demi`, `dog-demi`, `pointed-ears` und `muzzle-dog` sind **Stubs** (Manifeste ohne Meshes/Targets). Sie belegen das Datenmodell, nicht die Geometrie. Die Qualität von T1-Erweiterungen (Schnauzen-Patch) ist ungeprüft (SP-01).
- Detailfragen (Trait-Import aus fremden Spezies, Konfliktauflösung zweier Ohren-Komponenten) stehen in `../06-planning/open-questions.md` (OQ-20, OQ-30).
