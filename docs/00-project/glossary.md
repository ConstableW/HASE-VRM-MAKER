# Glossar

**Maßgebliche Quelle für Begriffe.** Begriffe bitte hier ergänzen statt in anderen Dokumenten neu zu definieren.

| Begriff | Bedeutung |
|---|---|
| Body Archetype / Body Family | Grundlegende Anatomie: Topologie, Skeleton, Rig-Anforderungen, Morph-Kompatibilität, Geometrie (Mandat 2.1) |
| Species | Speziesabhängige Merkmale, Basiswerte, Presets, Komponenten, Variationen, Kompatibilitätsregeln |
| Character | Individuelle Ausprägung: Morphs, Materialien, Kleidung, Animationen u. a. |
| Package | Versionierte, validierbare, installierbare Einheit; Typen Body, Species, Asset |
| Morph Target / Blendshape | Gespeicherte Vertexverschiebung (Delta) gegenüber einer Basis, mit Gewicht (0…1 oder signiert) skalierbar |
| Delta-Layer | Nachträgliche Geometrieänderung (z. B. Sculpt), als Differenz zur parametrisch erzeugten Basis gespeichert |
| Topologie | Konnektivität/Vertexanordnung eines Meshes; bestimmt, ob Morphs/Weights direkt übertragbar sind |
| Skinning / Weights | Zuordnung von Vertices zu Bones mit Gewichten |
| Retargeting | Übertragung einer Animation auf ein Skeleton mit anderen Proportionen/Struktur |
| Fitting | Automatische Anpassung von Kleidung/Haar/Accessoires an die Körperform |
| Spring Bones | Physikalisch simulierte Zusatzbones (Haare, Ohren, Schwänze, Stoff) |
| VRM 1.0 | glTF-basiertes Avatar-Format mit Humanoid-Mapping, Expressions, Spring Bones, MToon |
| VRMA | VRM-Animationsformat (`VRMC_vrm_animation`) |
| MToon | Toon-Shader-Material für VRM (`VRMC_materials_mtoon`) |
| Designer-Modus | Geführte, parametrische Oberfläche für normale Nutzer |
| Expert-Modus | Werkzeuge für Mesh-, Sculpt-, Weight-, UV-, Physik-Bearbeitung auf demselben Projektzustand |
| Projektzustand | Single Source of Truth des Characters im nativen Format (REQ-ARCH-008/-009) |
| Character-Intermediate | Evaluierter, ableitbarer Zwischenzustand (Mesh, Rig, Materialien) als Eingabe für Rendering und Export |
| Spike | Zeitlich begrenzter Machbarkeitsversuch mit messbaren Kriterien; nur nach Freigabe |
| ADR | Architecture Decision Record |
| Demi | Menschlich-anthropomorphe Mischform (z. B. Katzen-/Hunde-Demi) |
