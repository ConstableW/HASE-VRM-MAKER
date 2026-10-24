# Anforderungsmatrix

**Maßgebliche Quelle für alle Anforderungs-IDs.** Andere Dokumente verweisen per ID.
Quelle der Anforderungen: Projektmandat (`source/projektmandat-2026-10.docx`, im Folgenden „Mandat“), Abschnitt in Spalte *Mandat*.

## Lesehinweise

**Status** (Definition siehe `AGENTS.md`): `OD` = OWNER DECIDED (direkt aus dem Mandat), `PR` = PROPOSED (abgeleitet, noch nicht freigegeben), `AS` = ASSUMED.

**Priorität = Umsetzungsreihenfolge, NICHT Umfang.** Alle Anforderungen gehören zum Zielumfang (siehe REQ-PROD-005). Eine niedrige Priorität verschiebt, streicht aber nichts.

| Prio | Bedeutung |
|---|---|
| P0 | Fundament: ohne diese Anforderung ist die Architektur nicht tragfähig (Datenmodell, Packages, Projektzustand, Export-Kern) |
| P1 | Kern-Produkt: erste nutzbare Version eines humanoiden Creators |
| P2 | Ausbau: vollständige Studio-Funktionen |
| P3 | Erweiterung: Expert-Funktionen, Simulation, Ökosystem |

Diese Priorisierung ist selbst `PR` bis zur Owner-Abnahme.

## Produkt

| ID | Anforderung | Status | Prio | Mandat |
|---|---|---|---|---|
| REQ-PROD-001 | Eigenständiges Desktop-Produkt; HASE ist nur Produktname, keine verpflichtende technische Abhängigkeit zu Home Assistant, HASEC oder anderen HASE-Produkten | OD | P0 | 1 |
| REQ-PROD-002 | Windows ist primäre Zielplattform; weitere Plattformen sind nicht auszuschließen | OD | P0 | 5 (13) |
| REQ-PROD-003 | Nutzererlebnis: erstellen → gestalten → bei Bedarf detailliert nachbearbeiten → technisch prüfen → unmittelbar verwenden | OD | P1 | 1.1 |
| REQ-PROD-004 | Technische Komplexität wird möglichst von der Software übernommen, nicht vom Nutzer | OD | P1 | 1.1, 13 |
| REQ-PROD-005 | Vollständiger Zielumfang bleibt erhalten; inkrementelle Umsetzung erlaubt, eigenmächtige Reduzierung nicht | OD | P0 | 1.2, 9 |
| REQ-PROD-006 | Referenzprodukte (AION 2, GW2, VRoid Studio, Blender) sind funktionale/gestalterische Referenz; keine Übernahme geschützter Assets, UIs oder proprietärer Implementierungen | OD | P0 | 1.1 |
| REQ-PROD-007 | Produkt ist weder VRoid-Klon noch vereinfachte Blender-Oberfläche | OD | P1 | 1.1 |
| REQ-PROD-008 | Zielgruppen definieren und priorisieren (Hobby-Creator, VTuber/Avatar-Nutzer, Spiele-/Asset-Creator, Package-Autoren) | PR | P1 | 10.A |

## Architektur, Spezies, Packages

| ID | Anforderung | Status | Prio | Mandat |
|---|---|---|---|---|
| REQ-ARCH-001 | Drei getrennte Ebenen: Body Archetype/Body Family, Species, Character; nicht fest verdrahtet | OD | P0 | 2.1 |
| REQ-ARCH-002 | Core-Architektur darf nicht auf den humanoiden Archetype beschränkt sein | OD | P0 | 2.1 |
| REQ-ARCH-003 | Komposition und Wiederverwendung vor starrer Klassenvererbung | OD | P0 | 2.2 |
| REQ-ARCH-004 | Erste vollständig implementierte Körperfamilie: humanoid (Mensch, Elf, Katzen-Demi, Hunde-Demi, weitere anthropomorphe Mischformen) | OD | P1 | 2.1 |
| REQ-ARCH-005 | Mischformen möglich (z. B. menschliches Gesicht + Tierohren; Schnauze, Pfoten, abweichende Beinproportionen) | OD | P1 | 2.1 |
| REQ-ARCH-006 | Untersuchen, wann gemeinsame Topologie genügt und wann eigener Body Archetype nötig ist | OD | P0 | 2.1 |
| REQ-ARCH-007 | Spätere Körperfamilien mit zusätzlichen Gliedmaßen, abweichenden Skeletten, nichtmenschlicher Anatomie dürfen nicht grundsätzlich ausgeschlossen sein | OD | P0 | 2.1 |
| REQ-ARCH-008 | Projektzustand ist Single Source of Truth; Designer- und Expert-Modus nutzen denselben Zustand | OD | P0 | 3.6 |
| REQ-ARCH-009 | Native Projektdatei erhält sämtliche editierbaren Quelldaten (verlustfrei, versioniert, migrierbar) | OD | P0 | 4.3 |
| REQ-ARCH-010 | Erweiterbarkeit über definierte Schnittstellen (Plugins/Packages), ohne Kernänderung | PR | P1 | 5 (7) |

| ID | Anforderung | Status | Prio | Mandat |
|---|---|---|---|---|
| REQ-PKG-001 | Spezies sind eigenständige installierbare Packages; mitgelieferte Standardspezies nutzen dieselbe Package-Infrastruktur wie Community-Packages | OD | P0 | 2.2 |
| REQ-PKG-002 | Body Packages: Körperbasen, Topologien, Skeletons, Rig-Profile, Morph-Systeme | OD | P0 | 2.2 |
| REQ-PKG-003 | Species Packages: Speziesdefinition, Standardwerte, Merkmale, Presets, Abhängigkeiten | OD | P0 | 2.2 |
| REQ-PKG-004 | Asset Packages: Kleidung, Haare, Augen, Materialien, Körperkomponenten, Accessoires, Animationen | OD | P0 | 2.2 |
| REQ-PKG-005 | Packages sind versioniert, validierbar, referenzierbar und möglichst unabhängig aktualisierbar | OD | P0 | 2.2 |
| REQ-PKG-006 | Neue Spezies erfordert keine Änderung am Anwendungskern | OD | P0 | 2.2 |
| REQ-PKG-007 | Dokumentierte Import- und Package-Pipeline für externe Werkzeuge (Procreate, Blender, weitere 2D/3D-Tools) | OD | P1 | 2.2 |
| REQ-PKG-008 | HASE Package SDK (konzipieren) | OD | P1 | 2.2 |
| REQ-PKG-009 | Visueller Package Builder (konzipieren) | OD | P2 | 2.2 |
| REQ-PKG-010 | Package-Integrität und -Herkunft prüfbar (Hashes, optional Signatur) | PR | P1 | 11 (Package-Versionierung) |
| REQ-PKG-011 | Lizenz- und Herkunftsmetadaten (SPDX) pro Package/Asset, Anzeige im UI und im Export | PR | P1 | 9 (Lizenz) |
| REQ-PKG-012 | Kompatibilitätsregeln deklarativ (Body ↔ Species ↔ Asset), Prüfung vor Installation und Nutzung | PR | P0 | 2.1, 3.4 |
| REQ-PKG-013 | Sichere Behandlung von Community-Packages (keine unkontrollierte Codeausführung) | PR | P0 | 5 (7) |

## Gesichtseditor (Mandat 3.1)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-FACE-001 | Austauschbare Basisgesichter | OD | P1 |
| REQ-FACE-002 | Schädel- und Gesichtsproportionen | OD | P1 |
| REQ-FACE-003 | Stirn, Brauen, Augenhöhlen | OD | P1 |
| REQ-FACE-004 | Augen, Lider, Iris, Pupillen | OD | P1 |
| REQ-FACE-005 | Nase, Nasenrücken, Nasenflügel | OD | P1 |
| REQ-FACE-006 | Wangen, Jochbein, Gesichtsfülle | OD | P1 |
| REQ-FACE-007 | Mund, Lippen, Kiefer | OD | P1 |
| REQ-FACE-008 | Kinn, Ohren | OD | P1 |
| REQ-FACE-009 | Speziesabhängige Schnauzen und Fangzähne | OD | P2 |
| REQ-FACE-010 | Symmetrische und asymmetrische Bearbeitung | OD | P1 |
| REQ-FACE-011 | Presets, Morph Targets, Detailkorrekturen | OD | P1 |
| REQ-FACE-012 | Direkte Manipulation am 3D-Modell, abgebildet auf konsistente parametrische Werte (Untersuchung, siehe R-04) | OD | P2 |

## Körpereditor (Mandat 3.2)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-BODY-001 | Körpergröße und Proportionen | OD | P1 |
| REQ-BODY-002 | Körperbau, Muskulatur, Fettverteilung | OD | P1 |
| REQ-BODY-003 | Schulter-, Brust-, Taillen-, Beckenformen | OD | P1 |
| REQ-BODY-004 | Arme, Beine, Hände, Füße | OD | P1 |
| REQ-BODY-005 | Hals und Rumpf | OD | P1 |
| REQ-BODY-006 | Anatomische Besonderheiten, speziesabhängige Körpermerkmale (Schwanz, Pfoten, Digitigrade-Beine u. ä.) | OD | P2 |
| REQ-BODY-007 | Symmetrische und asymmetrische Veränderungen | OD | P1 |
| REQ-BODY-008 | Automatische Anpassung von Skeleton und Skinning an Morphs | OD | P1 |
| REQ-BODY-009 | Korrekturmechanismen bei extremen Morph-Kombinationen | OD | P2 |
| REQ-BODY-010 | Technische Fehlererkennung bei gleichzeitig kreativer Freiheit (Warnen statt Verbieten) | OD | P1 |

## Haarstudio (Mandat 3.3)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-HAIR-001 | Modulare Frisuren | OD | P1 |
| REQ-HAIR-002 | Haarsegmente und Strähnen | OD | P2 |
| REQ-HAIR-003 | Haaransatz, Länge, Volumen, Krümmung | OD | P2 |
| REQ-HAIR-004 | Farben, Verläufe, Materialeigenschaften | OD | P1 |
| REQ-HAIR-005 | Stilisiertes und realistischeres Haar | OD | P2 |
| REQ-HAIR-006 | Physik und Kollisionen | OD | P2 |
| REQ-HAIR-007 | Import externer Haar-Assets | OD | P2 |
| REQ-HAIR-008 | Speziesabhängige Anpassung | OD | P2 |

## Kleidungseditor (Mandat 3.4)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-CLO-001 | Erweiterbare Outfit-Bibliothek | OD | P1 |
| REQ-CLO-002 | Mehrschichtige Kleidung | OD | P1 |
| REQ-CLO-003 | Kombinierbare Einzelteile | OD | P1 |
| REQ-CLO-004 | Automatische Anpassung an Körper-Morphs | OD | P1 |
| REQ-CLO-005 | Skinning und Weight Transfer | OD | P1 |
| REQ-CLO-006 | Kollisionsprüfung (Durchdringung) | OD | P1 |
| REQ-CLO-007 | Materialien, Muster, Farben | OD | P1 |
| REQ-CLO-008 | Eigene Kleidungspakete | OD | P1 |
| REQ-CLO-009 | Accessoires und Befestigungspunkte | OD | P1 |
| REQ-CLO-010 | Optionale Cloth-Simulation | OD | P3 |
| REQ-CLO-011 | Wiederverwendbarkeit von Kleidung über Proportionen und kompatible Spezies hinweg (Untersuchung, siehe R-03) | OD | P0 |

## Materialien und Texturen (Mandat 3.5)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-MAT-001 | Materialien für Haut und Fell | OD | P1 |
| REQ-MAT-002 | Materialien für Augen und Haare | OD | P1 |
| REQ-MAT-003 | Stoffe, Leder, Metall | OD | P2 |
| REQ-MAT-004 | Transparenz und Oberflächenstruktur | OD | P1 |
| REQ-MAT-005 | Farb- und Muster-Layer | OD | P1 |
| REQ-MAT-006 | Materialzonen | OD | P1 |
| REQ-MAT-007 | Texturimport | OD | P1 |
| REQ-MAT-008 | Shader-Kompatibilität (insbesondere MToon/VRM) | OD | P0 |
| REQ-MAT-009 | Baking und Exportoptimierung | OD | P2 |

## Advanced Editing (Mandat 3.6)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-ADV-001 | Expert-Modus als eigener Bearbeitungsmodus | OD | P2 |
| REQ-ADV-002 | Mesh-Bearbeitung | OD | P3 |
| REQ-ADV-003 | Sculpting | OD | P3 |
| REQ-ADV-004 | Morph- und Blendshape-Editor | OD | P2 |
| REQ-ADV-005 | UV-Bearbeitung | OD | P3 |
| REQ-ADV-006 | Material-Editor | OD | P2 |
| REQ-ADV-007 | Rigging und Weight Painting | OD | P2 |
| REQ-ADV-008 | Physics- und Constraint-Bearbeitung | OD | P2 |
| REQ-ADV-009 | Animation und Posing | OD | P2 |
| REQ-ADV-010 | Nichtdestruktive, parametrische Bearbeitung bleibt erhalten; Umgang mit inkompatiblen Geometrieänderungen ist definiert (Untersuchung, siehe R-05) | OD | P0 |

## Rigging, Animation, Export (Mandat 4)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-RIG-001 | Unterschiedliche Skeleton-Profile und zusätzliche Bones | OD | P0 |
| REQ-RIG-002 | Automatische Rig-Anpassung | OD | P1 |
| REQ-RIG-003 | Bone-Mapping | OD | P0 |
| REQ-RIG-004 | Skinning | OD | P1 |
| REQ-RIG-005 | Corrective Deformations | OD | P2 |
| REQ-RIG-006 | Retargeting | OD | P1 |
| REQ-RIG-007 | Speziesabhängige Bones | OD | P1 |
| REQ-RIG-008 | Zusätzliche Körperkomponenten | OD | P2 |
| REQ-RIG-009 | Rig-Validierung | OD | P1 |
| REQ-ANI-001 | Animationen direkt im Editor abspielen und testen | OD | P1 |
| REQ-ANI-002 | Idle, Gehen, Laufen, Springen | OD | P1 |
| REQ-ANI-003 | Sitzen und grundlegende Interaktionen | OD | P2 |
| REQ-ANI-004 | Gesichtsausdrücke (Expressions) | OD | P1 |
| REQ-ANI-005 | Augenbewegungen und Blicksteuerung | OD | P1 |
| REQ-ANI-006 | Lippenbewegungen und Viseme | OD | P2 |
| REQ-ANI-007 | Spring Bones | OD | P1 |
| REQ-ANI-008 | Animation-Retargeting | OD | P1 |
| REQ-ANI-009 | Import von Animationen | OD | P1 |
| REQ-ANI-010 | VRMA-Unterstützung | OD | P1 |
| REQ-ANI-011 | Pose-Editor und Posenbibliothek | OD | P2 |
| REQ-EXP-001 | VRM 1.0 als zentrales Exportziel, nicht als internes Datenmodell | OD | P0 |
| REQ-EXP-002 | Eignung weiterer Formate prüfen (glTF/GLB, FBX) | OD | P2 |
| REQ-EXP-003 | Export-/Validierungspipeline: VRM-Kompatibilität, Humanoid-Bone-Mapping, Expressions, Materialien, Spring Bones | OD | P0 |
| REQ-EXP-004 | Polygon- und Texturbudgets, LOD-Strategien | OD | P2 |
| REQ-EXP-005 | Fehlerdiagnostik mit verständlichen, handlungsleitenden Meldungen | OD | P0 |
| REQ-EXP-006 | Verlustbehaftete Konvertierungen sind sichtbar und dokumentiert (Export-Report) | OD | P0 |
| REQ-EXP-007 | Reproduzierbare (deterministische) Exportergebnisse | OD | P0 |

## UX (Mandat 6)

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-UX-001 | Bedienung für normale Nutzer intuitiv trotz komplexer Engine | OD | P1 |
| REQ-UX-002 | Character-Creation-Wizard | OD | P1 |
| REQ-UX-003 | Freie Bearbeitungsoberfläche | OD | P1 |
| REQ-UX-004 | Kategorie- und Preset-Navigation | OD | P1 |
| REQ-UX-005 | Direkte Manipulation | OD | P2 |
| REQ-UX-006 | Kontextabhängige Werkzeuge | OD | P1 |
| REQ-UX-007 | Suchbare Asset-Bibliotheken | OD | P1 |
| REQ-UX-008 | Vergleich von Charaktervarianten | OD | P2 |
| REQ-UX-009 | Undo/Redo und Änderungsverlauf | OD | P0 |
| REQ-UX-010 | Kamera- und Lichtpresets | OD | P1 |
| REQ-UX-011 | Mehrere Vorschauansichten | OD | P2 |
| REQ-UX-012 | Animationstest in der Vorschau | OD | P1 |
| REQ-UX-013 | Designer-/Expert-Modus | OD | P2 |
| REQ-UX-014 | Barrierefreiheit und alternative Eingabegeräte | OD | P1 |
| REQ-UX-015 | Projektverwaltung und Wiederherstellung (Autosave, Crash-Recovery) | OD | P1 |

## Qualität, Dokumentation, Recht, Prozess

| ID | Anforderung | Status | Prio | Mandat |
|---|---|---|---|---|
| REQ-QA-001 | Automatisierte Tests und visuelle Regressionstests | OD | P0 | 5 (11) |
| REQ-QA-002 | Performance-, Speicher- und Skalierbarkeitsziele quantifizieren | OD | P0 | 5 (12) |
| REQ-QA-003 | Reproduzierbare Nachweise für Validierung/Export | OD | P1 | 10.H |
| REQ-DOC-001 | Dokumentation ist integraler Bestandteil, parallel zur Arbeit aktualisiert | OD | P0 | 7 |
| REQ-DOC-002 | Statuslabel OWNER DECIDED / PROPOSED / ASSUMED / RESEARCHED / BLOCKED konsequent | OD | P0 | 7.2 |
| REQ-DOC-003 | ADRs für wesentliche Architekturentscheidungen | OD | P0 | 7.2 |
| REQ-DOC-004 | Übergabefähigkeit (CURRENT_STATE, WORK_LOG, NEXT_ACTIONS) | OD | P0 | 8 |
| REQ-DOC-005 | Eine maßgebliche Quelle pro Thema, keine widersprüchliche Parallel-Dokumentation; Ersetztes wird als ersetzt markiert | OD | P0 | 8.5 |
| REQ-LEG-001 | Alle Abhängigkeiten auf Lizenz und Distributionsfähigkeit prüfen | OD | P0 | 5 (14) |
| REQ-LEG-002 | Keine ungeprüfte Übernahme fremder Assets oder lizenzrechtlich problematischer Komponenten | OD | P0 | 9 |
| REQ-GOV-001 | Keine Implementierung, Commits, Pushes, Releases, kostenpflichtigen Ressourcen ohne Owner-Freigabe | OD | P0 | 9 |
| REQ-GOV-002 | Technische Spikes nur nach gesonderter Freigabe | OD | P0 | 5 |
| REQ-GOV-003 | Architekturfreigabe ausschließlich durch den Product Owner | OD | P0 | 12 |

## Performance (OD-025)

Maßgebliche Zielwerte und Messregeln: `../07-quality/performance-targets.md`.

| ID | Anforderung | Status | Prio |
|---|---|---|---|
| REQ-PERF-001 | Interaktive Frame-Time statisch (p95 ≤ 16,6 ms T-Ref) | OD | P0 |
| REQ-PERF-002 | Kontinuierliche Parameteränderung ohne Frame-Einbruch (64 aktive Targets) | OD | P0 |
| REQ-PERF-003 | Eingabe→Pixel ≤ 2 Frames | OD | P0 |
| REQ-PERF-004 | Alle Targets gleichzeitig aktiv als Stresstest | PR | P1 |
| REQ-PERF-005 | UI-Frame ≤ 2 ms CPU bei ≥ 600 Parametern (virtualisiert) | OD | P0 |
| REQ-PERF-006 | Keine UI-Blockade > 100 ms; lange Operationen abbrechbar | OD | P0 |
| REQ-PERF-007 | Inkrementelles Fitting nur betroffener Komponenten | OD | P0 |
| REQ-PERF-008 | Keine unnötigen Neuberechnungen (messbar) | OD | P0 |
| REQ-PERF-009 | Projekt-Öffnen ≤ 3 s | PR | P1 |
| REQ-PERF-010 | Große Bibliotheken/Package-Mengen skalieren (Index, Virtualisierung) | OD | P0 |
| REQ-PERF-011 | CPU-Speicher kontrolliert (≤ 1,5 GB / 50 Assets) | OD | P0 |
| REQ-PERF-012 | GPU-Speicher kontrolliert (≤ 1 GB T-Ref) | OD | P0 |
| REQ-PERF-013 | Morph-Daten sparse/komprimiert (≤ 100 MB / 609 Targets) | PR | P0 |
| REQ-PERF-014 | Export/Validierung im Hintergrund, UI unbeeinträchtigt | OD | P0 |
| REQ-PERF-015 | Asynchrones Laden, Platzhalter, Caching | OD | P0 |
| REQ-PERF-016 | Effizientes Skinning/Posing/Animation | OD | P0 |
| REQ-PERF-017 | Skalierung linear in sichtbaren Assets, unabhängig von Bibliotheksgröße | OD | P0 |
| REQ-PERF-018 | Profiling von Anfang an eingebaut | OD | P0 |
| REQ-PERF-019 | Performance-Regressionstests in CI | PR | P1 |
| REQ-PERF-020 | Performance ohne Einschränkung der Spezies-/Package-Erweiterbarkeit | OD | P0 |


## Körperbasis- und Speziesstudio (OD-027)

| ID | Anforderung | Quelle | Prio |
|---|---|---|---|
| REQ-STUDIO-001 | Eigener Arbeitsbereich zum Erstellen/Verwalten von Körperbasen und Spezies im Designsystem | OD | P0 |
| REQ-STUDIO-002 | Neue Körperbasis aus Vorlage erstellen, duplizieren, weiterentwickeln | OD | P0 |
| REQ-STUDIO-003 | Externes Modell importieren (Mesh, Materialien, Texturen, optional Rig) mit sichtbarer Voraussetzungsprüfung | OD | P0 |
| REQ-STUDIO-004 | Anatomische Regionen, Links/Rechts-Zuordnung, Symmetrieachsen, Morph-Import/-Kategorisierung, Korrektur-Morphs definierbar | OD | P0 |
| REQ-STUDIO-005 | Skelett, Bone-Mapping, Skinning prüfen; Körperbasis als Package speichern | OD | P0 |
| REQ-STUDIO-006 | Prüfstufen getrennt ausweisen: importiert / geometrisch verwendbar / rig-kompatibel / morph-kompatibel / konfiguriert / als Speziespaket validiert | OD | P0 |
| REQ-STUDIO-007 | Spezies aus einer oder mehreren kompatiblen Körperbasen plus Merkmalen (Ohren, Haut, Augen, Morphs, Defaults, Grenzen, kompatible Assets, Rig-Erweiterungen) zusammenstellen; nicht auf Humanoide beschränkt | OD | P0 |
| REQ-STUDIO-008 | Visueller Spezieseditor (Bibliothek, Körperbasis-Auswahl, Viewport, Anatomie-/Rig-Ansicht, Parameterzuordnung, Merkmalsbibliothek, Kompatibilität, Paketinfo/Version, Validierung) | OD | P0 |
| REQ-STUDIO-009 | Körperbasen/Spezies sind versionierte, duplizierbare, referenzierbare, als Paket exportierbare Ressourcen mit deklarierten Abhängigkeiten | OD | P0 |
| REQ-STUDIO-010 | Versions-/Migrationsstrategie: Änderungen zerstören bestehende Charaktere nicht unbemerkt | OD | P0 |
| REQ-STUDIO-011 | Charakter aus Spezies-Vorlage, Charakter als Vorlage/Variante speichern; Umwandlung Charakter → Spezies nur bewusst | OD | P0 |
| REQ-STUDIO-012 | Normale Abläufe ohne manuelle JSON/Manifest-Bearbeitung; technische Details im Expertenbereich | OD | P0 |
