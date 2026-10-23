# DECISION_LOG

Chronologisches Register. Maßgeblich für Owner-Entscheidungen ist `../00-project/owner-decisions.md`, für Architektur die ADRs. Hier stehen Routineentscheidungen des Analyseauftrags (dokumentiert, reversibel; Mandat 9).

| ID | Datum | Entscheidung | Art | Begründung | Reversibel |
|---|---|---|---|---|---|
| D-001 | 2026-10-23 | Dokumentation auf Deutsch | Routine (ASSUMED, OQ-10) | Mandat ist deutsch | ja |
| D-002 | 2026-10-23 | Mandatsstruktur um owner-decisions.md, investigations.md, functional-decomposition.md, ADR-Register/-Vorlage, AGENTS.md, source/ erweitert | Routine (PROPOSED → ADR-0001) | Eine maßgebliche Quelle pro Thema | ja |
| D-003 | 2026-10-23 | Priorität P0–P3 bedeutet Reihenfolge, nicht Umfang | Routine | Mandat 1.2 / REQ-PROD-005 | ja |
| D-004 | 2026-10-23 | Es wird nichts committet oder gepusht, obwohl die Sitzung eine Entwicklungsbranch vorgibt | Befolgung Mandat 9 | Ausdrückliche Owner-Vorgabe hat Vorrang; Freigabe als OQ-11 offen | ja |
| D-005 | 2026-10-23 | `assets/` als nicht redistributierbare interne Testdaten behandelt | Routine (PROPOSED, OQ-03) | Mixamo-Herkunft, keine Lizenz | ja |
| D-006 | 2026-10-23 | Rechercheberichte unverändert übernommen, Empfehlungen als PROPOSED geführt | Routine | Evidenz bleibt nachvollziehbar | ja |
| D-007 | 2026-10-23 | Konzeptdokumente committet und auf Branch `claude/gifted-ptolemy-tdh9vc` gepusht | Owner-Freigabe OD-019 | Owner: „Git nach eigenem Ermessen“ | ja |
| D-008 | 2026-10-23 | Produktlizenz Apache-2.0 (Code) + CC0/CC-BY (Eigen-Assets) | Owner delegiert → ADR-0009/OD-020 | Passt zum permissiven Stack und offenem Package-Ökosystem | ja |
