# Offene Fragen und Owner-Entscheidungen

**Maßgebliche Quelle für alles Ungeklärte.** Nichts hier ist stillschweigend entschieden. Wird eine Frage entschieden, wandert sie nach `../00-project/owner-decisions.md` (bzw. als ADR nach `../05-decisions/`) und wird hier als *geschlossen* markiert (nicht gelöscht).

## A. Echte Owner-Entscheidungen (Freigabe nötig)

| ID | Frage | Alternativen | Auswirkung | Empfehlung (PROPOSED) | Dringlichkeit |
|---|---|---|---|---|---|
| OQ-01 | **Technologie-Stack** (Sprache, Engine/Framework, Renderer, UI) | siehe `../04-research/technology-evaluation.md` | Langfristig, schwer reversibel | Wird nach Abschluss der Recherche als ADR-0002 mit Alternativen vorgelegt; Walking-Skeleton-Spike SP-08 vor Festlegung | Vor Implementierung |
| OQ-02 | **GESCHLOSSEN 2026-10-23 (ADR-0009, OD-020).** Lizenzmodell des Produkts: Apache-2.0 für Code, CC0/CC-BY für Eigen-Assets, Autor-Wahl für Community-Packages | – | – | – | erledigt |
| OQ-03 | **GESCHLOSSEN 2026-10-23.** Mixamo-abgeleitetes Animationspaket aus `assets/` entfernt, ersetzt durch 84 CC0-VRMA (Quaternius) | – | – | – | erledigt |
| OQ-04 | **Freigabe technischer Spikes** SP-01…SP-09 (Prototyp-Code zum Messen) | einzeln / gebündelt / nicht | Ohne Spikes bleiben R-01…R-05 unbelegt | Reihenfolge SP-08 → SP-01 → SP-02 → SP-04 | Nach Konzeptabnahme | *(Hinweis: Asset-Aufbereitungsskripte unter `tools/assets/` sind per OD-023 freigegeben; Spikes SP-01…09 weiterhin einzeln freizugeben.)*
| OQ-05 | **GESCHLOSSEN 2026-10-23 (ADR-0010, OD-021).** Basis-Content aus frei nutzbaren, selbst erstellten und KI-generierten Assets; Community später; keine kostenpflichtige Beschaffung | – | – | – | erledigt |
| OQ-06 | **Zielgruppen und Verwertung** (privat/VTuber/kommerzielle Avatare, Marktplatz?) | – | Beeinflusst UX-Priorität, VRM-Lizenzfelder, Package-Distribution | Primär: Creator, die VRM-Avatare nutzen; Ökosystem später | Mittel |
| OQ-07 | **Online-Dienste** (Package-Registry, Marketplace, Cloud) im Zielumfang? Mandat verlangt Installation/Referenzierung, nicht Hosting | Offline-first mit lokalen/Datei-Packages; Registry später; sofort | Kosten, Sicherheit, Betrieb | Offline-first, Registry-fähiges Design (REQ-PKG-005/-010) | Niedrig |
| OQ-08 | **Vertrauensmodell für Community-Packages**: dürfen Packages ausführbaren Code enthalten? | rein deklarativ / WASM-Sandbox / native Plugins | Sicherheit vs. Mächtigkeit (R-09) | Standard rein deklarativ; Code nur sandboxed und mit Berechtigungsfreigabe | Mittel |
| OQ-09 | **Mindest-Zielhardware** und Performancebudget (z. B. GPU-Klasse, RAM, Zielframerate im Editor) | – | Architektur von Morphing/Rendering | Vorschlag nach Benchmarks; Owner nennt Referenzgeräte | Mittel |
| OQ-10 | **Sprachen** von UI und Dokumentation (Doku aktuell Deutsch) | DE-only / DE+EN / i18n von Anfang an | i18n-Architektur, Community-Reichweite | UI i18n-fähig, Doku DE (Primär) + EN später | Niedrig |
| OQ-11 | **GESCHLOSSEN 2026-10-23 (siehe OD-019).** Commit-/Push-Freigabe für die Konzeptdokumente** (Mandat verbietet beides ohne ausdrückliche Freigabe) | Freigabe für Branch `claude/gifted-ptolemy-tdh9vc` erteilen / lokal belassen | Übergabefähigkeit über Container-Ende hinaus: Dateien liegen nur im flüchtigen Container, solange nicht committet | **Freigabe erteilen**, sonst gehen Ergebnisse verloren | **Sofort** |

## B. Fachliche Klärungen (kein Owner-Beschluss, aber offen)

| ID | Frage | Weg zur Klärung | Status |
|---|---|---|---|
| OQ-20 | Wann genügt eine gemeinsame Topologie, wann ist ein eigener Archetype nötig? | Kriterienkatalog `../02-architecture/species-system.md`; SP-01 | offen |
| OQ-21 | Wie werden Kleidung/Haar zwischen Körperfamilien mit disjunkter Topologie wiederverwendet? | SP-02 | offen |
| OQ-22 | Wie viele und welche Humanoid-Bones sind für Katzen-/Hunde-Demi nötig und wie bildet VRM 1.0 das ab? | `../04-research/feasibility.md`; SP-07 | offen |
| OQ-23 | Welches interne Austauschformat für das evaluierte Character-Intermediate? (z. B. glTF-basiert mit HASE-Erweiterungen) | ADR-0003 | offen |
| OQ-24 | Umfang des „Haarstudios“: Karten-/Streifenhaar vs. Strand-basiert | Feature-Katalog | offen |
| OQ-25 | Zielwerte für Polygon-/Textur-/Bone-Budgets je Zielprofil (VRChat, VTuber-Apps, Spiele-Engines) | Research + OQ-06 | offen |
| OQ-26 | **GESCHLOSSEN 2026-10-23 (OD-022).** Rechtsstatus KI-Assets: Owner akzeptiert „State of the Art“; Rechte an von Claude generierten Assets liegen laut Owner-Position beim Projekt/Owner. Rechtliche Unsicherheit bleibt dokumentiert (ADR-0010); Provenienzpflicht bleibt | – | erledigt |
| OQ-27 | **ENTSCHIEDEN 2026-10-23 (OD-023).** Basis = MakeHuman/MPFB2-CC0-Daten (Körper, Targets, Rig, Proxys) plus Quaternius-CC0; Aufbereitung per bpy-Skripte (`tools/assets/`). Austausch bis Release Candidate zulässig | – | erledigt |
| OQ-30 | Trait-Import über Spezies-Grenzen (`uses`), Konfliktauflösung zweier Komponenten im selben Slot, Migrations-Map-Format | Spezifikation K3 (Folge), SP-06 | offen |
