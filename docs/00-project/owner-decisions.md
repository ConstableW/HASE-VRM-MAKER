# Owner-Entscheidungen (OWNER DECIDED)

**Maßgebliche Quelle für alle verbindlichen Owner-Entscheidungen.** Herkunft: Projektmandat (`source/projektmandat-2026-10.docx`), Product Owner, Konzeptphase „Discovery & Architecture“.
Nur was hier steht, gilt als OWNER DECIDED. Offene Punkte stehen in `../06-planning/open-questions.md`.

| ID | Entscheidung | Mandat | Anforderungen |
|---|---|---|---|
| OD-001 | HASE VRM MAKER ist ein eigenständiges Desktop-Produkt. „HASE“ ist nur Produktname; keine verpflichtende Abhängigkeit zu Home Assistant, HASEC oder anderen HASE-Produkten | 1 | REQ-PROD-001 |
| OD-002 | Zielbild: Character Creation Studio – MMORPG-artige intuitive Erstellung plus DCC-artige Bearbeitungstiefe; kein VRoid-Klon, keine vereinfachte Blender-Oberfläche | 1.1 | REQ-PROD-003, -007 |
| OD-003 | Referenzprodukte sind nur funktionale/gestalterische Referenz, keine Vorlage für geschützte Assets, UIs oder Implementierungen | 1.1 | REQ-PROD-006 |
| OD-004 | Der vollständige Zielumfang bleibt erhalten. Inkrementelle Umsetzung ist erlaubt, eigenmächtige Reduzierung nicht | 1.2 | REQ-PROD-005 |
| OD-005 | Drei Ebenen Body Archetype / Species / Character, nicht fest verdrahtet | 2.1 | REQ-ARCH-001 |
| OD-006 | Erste Körperfamilie ist humanoid; Core darf nicht darauf beschränkt sein; spätere Körperfamilien mit anderen Skeletten/Gliedmaßen dürfen nicht ausgeschlossen sein | 2.1 | REQ-ARCH-002, -004, -007 |
| OD-007 | Komposition und Wiederverwendung sind gegenüber starrer Klassenvererbung zu bevorzugen | 2.2 | REQ-ARCH-003 |
| OD-008 | Spezies sind installierbare Packages; Standardspezies nutzen dieselbe Infrastruktur wie Community-Packages. Pakettypen: Body, Species, Asset | 2.2 | REQ-PKG-001 bis -004 |
| OD-009 | Neue Spezies darf keine Kernänderung erfordern; HASE Package SDK und visueller Package Builder sind zu konzipieren | 2.2 | REQ-PKG-006, -008, -009 |
| OD-010 | Designer- und Expert-Modus verwenden denselben Projektzustand | 3.6 | REQ-ARCH-008 |
| OD-011 | VRM 1.0 ist zentrales Zielformat, aber nicht das interne Datenmodell; das native Projektformat erhält alle editierbaren Quelldaten | 4.3 | REQ-EXP-001, REQ-ARCH-009 |
| OD-012 | Windows ist primäres Plattformziel | 5 | REQ-PROD-002 |
| OD-013 | Technologieentscheidungen werden evidenzbasiert aus Alternativvergleichen getroffen, nicht nach Präferenz; Ergebnis ist eine begründete Entscheidung | 5 | – |
| OD-014 | Unklare Machbarkeit → gezielte Spikes mit messbaren Kriterien; Implementierung nur nach gesonderter Freigabe | 5 | REQ-GOV-002 |
| OD-015 | Dokumentationspflicht, Statuslabel, ADRs, IDs, Übergabedateien gemäß Mandat 7 und 8 | 7, 8 | REQ-DOC-* |
| OD-016 | Verbindliche Grenzen: keine Produktimplementierung, keine eigenmächtige Visionsreduzierung, keine stillschweigenden Architekturentscheidungen bei Zielkonflikten, keine Commits/Pushes/Releases/kostenpflichtigen Ressourcen ohne Freigabe, keine ungeprüften Fremdassets | 9 | REQ-GOV-001, REQ-LEG-002 |
| OD-017 | Vor wesentlichen Architekturfestlegungen werden dem Owner Alternativen und begründete Empfehlung vorgelegt; Routineentscheidungen im Analyseauftrag dürfen selbständig getroffen werden, sofern dokumentiert und reversibel | 9 | – |
| OD-018 | Endgültige Architekturfreigabe ausschließlich durch den Product Owner | 12 | REQ-GOV-003 |
| OD-019 | Git-Operationen (Commit, Push auf die Entwicklungsbranch) nach Ermessen des Agenten freigegeben (Owner-Nachricht 2026-10-23: „Git nach eigenem Ermessen“). Gilt nicht für Releases, Veröffentlichungen, Pull Requests oder kostenpflichtige Ressourcen | Sitzung 2026-10-23 | REQ-GOV-001 |
| OD-020 | Lizenzmodell: Entscheidung an Claude delegiert („best guess“), umgesetzt in ADR-0009 (Apache-2.0 Code, CC0/CC-BY Eigen-Assets, Autor-Wahl für Packages). Durch Owner jederzeit revidierbar | Sitzung 2026-10-23 | REQ-LEG-001, ADR-0009 |
| OD-021 | Basis-Content: frei nutzbare, selbst erstellte und KI-generierte Assets; Community später; keine kostenpflichtige Beschaffung. Ausgestaltung siehe ADR-0010 | Sitzung 2026-10-23 | REQ-PKG-011, REQ-LEG-002, ADR-0010 |
