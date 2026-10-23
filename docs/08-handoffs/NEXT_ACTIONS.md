# NEXT_ACTIONS

*Stand: 2026-10-23.* Status: offen / in Arbeit / erledigt / wartet auf Owner.

| # | Aktion | Ziel | Voraussetzung | Erwartetes Ergebnis | Akzeptanzkriterien | Status | Agent |
|---|---|---|---|---|---|---|---|
| 1 | Owner-Review der Erstergebnisse inkl. ADR-0009 (Lizenz, delegiert entschieden) | Ergebnisse sichern, Lizenzbewertung entsperren | – | Freigabe/Entscheidung | Entscheidung in `owner-decisions.md` bzw. `DECISION_LOG.md` | wartet auf Owner | – |
| 2 | Species-/Package-System ausarbeiten (`species-system.md`, `package-system.md`) inkl. Kriterien „Topologie-Familie vs. Archetype“, Kompatibilitäts- und Versionsregeln | Kern von REQ-ARCH/PKG | Recherche `references.md` B, `feasibility.md` §4 | Zwei Architekturdokumente + ADR-0004/-0005 (PROPOSED) | Jede REQ-ARCH-*/PKG-* referenziert; Beispiele für Mensch, Elf, Katzen-Demi, Hunde-Demi; Konfliktfälle beschrieben | offen | – |
| 3 | Spezifikationen: Projektformat, Package-Manifest, Species-/Asset-Schema, Kompatibilitätsregeln + Beispielpakete in `examples/species/` | REQ-PKG-005/-008 | Aktion 2 | `03-specifications/*`, Beispiel-Manifeste | Beispiele validieren gegen die Schemas (Prüfung auf Papierebene bzw. JSON-Schema-Dateien) | offen | – |
| 4 | Geometrie/Morphing, Rigging/Animation, Fitting-Konzept | R-01/-03/-05/-06 | Aktion 2 | `geometry-morphing.md`, `rigging-animation.md`, ADR-0004/-0006 | Delta-Layer-Modell, Invalidierungsregeln, Fitting-Pipeline und Qualitätsmetriken definiert | offen | – |
| 5 | Feature-Katalog mit Abhängigkeiten/Akzeptanzkriterien; User Journeys; UX-Konzept und Screen-Entwürfe | REQ-UX-*, Mandat 3/6 | – | `01-product/*` | Jede REQ-FACE/BODY/HAIR/CLO/MAT/ADV-ID hat Feature-Eintrag und Akzeptanzkriterium | offen | – |
| 6 | Qualitätsattribute, Teststrategie, Validierungsstrategie | REQ-QA-* | – | `07-quality/*` | Messbare Attribute, Golden-/Visual-Regression-Konzept | offen | – |
| 7 | Render-, Asset-, Export-Pipeline-Konzepte | AI-03, AI-06, AI-17 | Aktion 4 | `rendering.md`, `asset-pipeline.md`, `export-pipeline.md` | VRM-Profil, Report, Determinismus beschrieben | offen | – |
| 8 | Primärquellen nachverifizieren (VRM-Spec, VRoid-Terms, Mixamo-Terms, Blender-FAQ, Lizenzen UniVRM/godot-vrm/Bevy/Godot) | Recherchelücken schließen | Netzzugang zu den Hosts | Ergänzte `references.md`/`feasibility.md` | Alle [A] in Entscheidungsrelevanten Punkten ersetzt oder bestätigt | offen | – |
| 9 | ADR-0002 (Stack) als Entscheidungsvorlage an den Owner | OQ-01 | Aktionen 2–4, OQ-02 | ADR mit Alternativen/Empfehlung | Matrix mit belegten Kriterien; Spike-Plan SP-07/-08 bestätigt | offen | – |
| 10 | Spike-Spezifikationen finalisieren und zur Freigabe vorlegen | OQ-04 | Aktion 9 | Spike-Briefs | Messbare Kriterien, Zeitbox, Hardware | offen | – |
| 11 | Konsistenzreview (Traceability REQ ↔ F ↔ ADR ↔ Test), Abnahmepaket | Mandat 12 | Aktionen 2–10 | Abnahmedokument | Alle Abschlusskriterien aus Mandat 12 geprüft | offen | – |

Hinweis: Aktionen mit Spike-Code, Commits, Käufen sind **nicht** ohne Owner-Freigabe auszuführen.
| 12 | Testcandidate weiter vorbefüllen: Rig-Mapping MakeHuman → VRM-Humanoid, Dezimierung/LOD, Expression-Blendshapes (Blink, Viseme), prozedurale Augen/Haare/Kleidung als Eigen-Assets, eigene Idle-Animation (VRMA), Test-VRM-Export | OD-023, OQ-27 | Aktion nur mit Dev-Tools (`bpy` im Scratch-venv) | Lauffähiger Test-Avatar als VRM | VRM lädt fehlerfrei in Referenz-Loader (soweit im Container testbar) | offen | – |
| 13 | Owner: Netzwerk-Allowlist erweitern für `static.makehumancommunity.org` (+Spiegel), `quaternius.itch.io`/`quaternius.com` (optional `download.blender.org`) | Haare/Augen/Kleidung/Animationen aus CC0-Packs | Owner-Aktion in Umgebungseinstellungen | Zugriff auf Packs | Hosts erreichbar | wartet auf Owner | – |
