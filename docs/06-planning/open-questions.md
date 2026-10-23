| OQ-11 |# Offene Fragen und Owner-Entscheidungen
| OQ-11 |
| OQ-11 |**Maßgebliche Quelle für alles Ungeklärte.** Nichts hier ist stillschweigend entschieden. Wird eine Frage entschieden, wandert sie nach `../00-project/owner-decisions.md` (bzw. als ADR nach `../05-decisions/`) und wird hier als *geschlossen* markiert (nicht gelöscht).
| OQ-11 |
| OQ-11 |## A. Echte Owner-Entscheidungen (Freigabe nötig)
| OQ-11 |
| OQ-11 || ID | Frage | Alternativen | Auswirkung | Empfehlung (PROPOSED) | Dringlichkeit |
| OQ-11 ||---|---|---|---|---|---|
| OQ-11 || OQ-01 | **Technologie-Stack** (Sprache, Engine/Framework, Renderer, UI) | siehe `../04-research/technology-evaluation.md` | Langfristig, schwer reversibel | Wird nach Abschluss der Recherche als ADR-0002 mit Alternativen vorgelegt; Walking-Skeleton-Spike SP-08 vor Festlegung | Vor Implementierung |
| OQ-11 || OQ-02 | **Lizenzmodell von HASE VRM MAKER selbst** (Repo trägt aktuell Apache-2.0-`LICENSE`; ist Open Source, Source-available oder proprietär geplant? Kommerzielle Distribution?) | Open Source (permissiv/copyleft), proprietär, Open Core | Bestimmt zulässige Abhängigkeiten (GPL/LGPL/AGPL, Qt, Blender-Bündelung) | Erst klären, dann Abhängigkeiten final bewerten | **Hoch** (blockiert Lizenzbewertung) |
| OQ-11 || OQ-03 | **Vorhandenes Animationspaket** (`assets/`, 118 BVH + 118 VRMA, Quelle: SillyTavern „VRM-Assets-Pack“) – Quellrepo ohne Lizenz, Animationen laut README von Mixamo (Rohdaten-Weitergabe als Pack laut Adobe-FAQ nicht zulässig; RESEARCHED, schwach) | behalten als reine Testdaten / entfernen / durch klar lizenzierte ersetzen | Rechtsrisiko bei Weitergabe (REQ-LEG-002) | Als *nur interne Testdaten, nicht redistributierbar* einstufen, bis Lizenz geklärt (siehe `../04-research/references.md`) | Mittel |
| OQ-11 || OQ-04 | **Freigabe technischer Spikes** SP-01…SP-09 (Prototyp-Code zum Messen) | einzeln / gebündelt / nicht | Ohne Spikes bleiben R-01…R-05 unbelegt | Reihenfolge SP-08 → SP-01 → SP-02 → SP-04 | Nach Konzeptabnahme |
| OQ-11 || OQ-05 | **Beschaffung der Basis-Content-Assets** (humanoider Body/Face-Basismesh, Morph-Set, Haare, Augen, Beispielkleidung): selbst erstellen, beauftragen, lizenzieren, prozedural | – | Kosten, Qualität, Rechte; R-13 | Eigenständig erstellte/beauftragte Basis mit klarer Rechtelage; **kostenpflichtig → Freigabe nötig** | Hoch |
| OQ-11 || OQ-06 | **Zielgruppen und Verwertung** (privat/VTuber/kommerzielle Avatare, Marktplatz?) | – | Beeinflusst UX-Priorität, VRM-Lizenzfelder, Package-Distribution | Primär: Creator, die VRM-Avatare nutzen; Ökosystem später | Mittel |
| OQ-11 || OQ-07 | **Online-Dienste** (Package-Registry, Marketplace, Cloud) im Zielumfang? Mandat verlangt Installation/Referenzierung, nicht Hosting | Offline-first mit lokalen/Datei-Packages; Registry später; sofort | Kosten, Sicherheit, Betrieb | Offline-first, Registry-fähiges Design (REQ-PKG-005/-010) | Niedrig |
| OQ-11 || OQ-08 | **Vertrauensmodell für Community-Packages**: dürfen Packages ausführbaren Code enthalten? | rein deklarativ / WASM-Sandbox / native Plugins | Sicherheit vs. Mächtigkeit (R-09) | Standard rein deklarativ; Code nur sandboxed und mit Berechtigungsfreigabe | Mittel |
| OQ-11 || OQ-09 | **Mindest-Zielhardware** und Performancebudget (z. B. GPU-Klasse, RAM, Zielframerate im Editor) | – | Architektur von Morphing/Rendering | Vorschlag nach Benchmarks; Owner nennt Referenzgeräte | Mittel |
| OQ-11 || OQ-10 | **Sprachen** von UI und Dokumentation (Doku aktuell Deutsch) | DE-only / DE+EN / i18n von Anfang an | i18n-Architektur, Community-Reichweite | UI i18n-fähig, Doku DE (Primär) + EN später | Niedrig |
| OQ-11 || **GESCHLOSSEN 2026-10-23 (siehe OD-019).** Commit-/Push-Freigabe für die Konzeptdokumente** (Mandat verbietet beides ohne ausdrückliche Freigabe) | Freigabe für Branch `claude/gifted-ptolemy-tdh9vc` erteilen / lokal belassen | Übergabefähigkeit über Container-Ende hinaus: Dateien liegen nur im flüchtigen Container, solange nicht committet | **Freigabe erteilen**, sonst gehen Ergebnisse verloren | **Sofort** |
| OQ-11 |
| OQ-11 |## B. Fachliche Klärungen (kein Owner-Beschluss, aber offen)
| OQ-11 |
| OQ-11 || ID | Frage | Weg zur Klärung | Status |
| OQ-11 ||---|---|---|---|
| OQ-11 || OQ-20 | Wann genügt eine gemeinsame Topologie, wann ist ein eigener Archetype nötig? | Kriterienkatalog `../02-architecture/species-system.md`; SP-01 | offen |
| OQ-11 || OQ-21 | Wie werden Kleidung/Haar zwischen Körperfamilien mit disjunkter Topologie wiederverwendet? | SP-02 | offen |
| OQ-11 || OQ-22 | Wie viele und welche Humanoid-Bones sind für Katzen-/Hunde-Demi nötig und wie bildet VRM 1.0 das ab? | `../04-research/feasibility.md`; SP-07 | offen |
| OQ-11 || OQ-23 | Welches interne Austauschformat für das evaluierte Character-Intermediate? (z. B. glTF-basiert mit HASE-Erweiterungen) | ADR-0003 | offen |
| OQ-11 || OQ-24 | Umfang des „Haarstudios“: Karten-/Streifenhaar vs. Strand-basiert | Feature-Katalog | offen |
| OQ-11 || OQ-25 | Zielwerte für Polygon-/Textur-/Bone-Budgets je Zielprofil (VRChat, VTuber-Apps, Spiele-Engines) | Research + OQ-06 | offen |
