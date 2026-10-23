# AGENTS.md – Einstieg für Entwickler und KI-Agenten

HASE VRM MAKER ist ein eigenständiges Desktop-Character-Creation-Studio (VRM 1.0 als zentrales Exportziel). **Aktuelle Phase: Konzept (Discovery & Architecture). Es gibt keinen Produktcode.**

## Lesereihenfolge bei Übernahme

1. `docs/08-handoffs/CURRENT_STATE.md` – Istzustand, Blocker, Verweise
2. `docs/08-handoffs/NEXT_ACTIONS.md` – was als Nächstes zu tun ist
3. `docs/00-project/owner-decisions.md` – was verbindlich ist
4. `docs/06-planning/open-questions.md` – was der Owner noch freigeben muss
5. `docs/00-project/requirements.md` – Anforderungs-IDs
6. `docs/08-handoffs/WORK_LOG.md` und `DECISION_LOG.md` – Historie

Das Originalmandat liegt unter `docs/00-project/source/projektmandat-2026-10.docx` und ist Quelle der Owner-Entscheidungen.

## Harte Regeln (Owner-Vorgaben, Mandat Abschnitt 9)

- Keine Implementierung des Produkts ohne gesonderten Auftrag. Spikes nur nach Freigabe.
- Keine Git-Commits, Pushes, Releases oder Veröffentlichungen ohne ausdrückliche Owner-Freigabe.
- Keine kostenpflichtigen Dienste/Ressourcen ohne Freigabe.
- Keine ungeprüfte Übernahme fremder Assets oder lizenzrechtlich problematischer Komponenten. Das Verzeichnis `assets/` hat ungeklärte Herkunft (siehe OQ-03) und ist nur interne Testdatenbasis.
- Keine eigenmächtige Reduzierung des Zielumfangs. Priorität ≠ Umfang.
- Keine stillschweigenden Architekturentscheidungen bei Zielkonflikten: Alternativen + Empfehlung dem Owner vorlegen. Endgültige Architekturfreigabe nur durch den Product Owner.

## Arbeitsprinzipien (OD-024)

- Open Source, nicht kommerziell ausgerichtet. KI-generierter Code/Assets sind zulässig.
- **Wiederverwenden vor Neuerfinden:** zuerst vorhandene Open-Source-Lösungen suchen, kombinieren, weiterentwickeln.
- Lizenzprüfung nur im Umfang der tatsächlichen Verwendung eines Fremdprojekts; Ergebnis ins Provenienzregister.
- Keine ungefragten Grundsatzdiskussionen über KI-Urheberrechte.

## Statuslabel (in allen Dokumenten konsequent verwenden)

| Label | Bedeutung |
|---|---|
| OWNER DECIDED | Vom Product Owner verbindlich festgelegt (nur in `owner-decisions.md` maßgeblich) |
| PROPOSED | Empfehlung, nicht freigegeben |
| ASSUMED | Arbeitsannahme, noch zu validieren |
| RESEARCHED | Durch zitierte Quelle/Untersuchung belegt (Quelle, Datum, Version angeben) |
| BLOCKED | Ohne weitere Information nicht entscheidbar |

## Dokumentationsregeln

- **Eine maßgebliche Quelle pro Thema**, andere Dokumente verweisen per ID/Link. Maßgebliche Quellen sind jeweils im Dokumentkopf benannt.
- IDs: `REQ-*` Anforderungen, `OD-*` Owner-Entscheidungen, `OQ-*` offene Fragen, `R-*` Risiken, `SP-*` Spikes, `F-*` Komponenten, `ADR-NNNN` Entscheidungen. Neue Artefakte verweisen auf REQ-IDs.
- Wesentliche Entscheidungen als ADR (`docs/05-decisions/`, Vorlage `ADR-TEMPLATE.md`): Kontext, Problem, Alternativen, Bewertung, Entscheidung/Empfehlung, Begründung, Konsequenzen, Status, Datum.
- Ersetztes wird als *ersetzt* markiert (mit Verweis), nie kommentarlos überschrieben.
- Dokumentation **parallel** zur Arbeit aktualisieren. Am Ende jeder Sitzung: `CURRENT_STATE.md`, `WORK_LOG.md`, `NEXT_ACTIONS.md` aktualisieren.
- Externe Aussagen belegen (URL, Abrufdatum, Version). Nicht verifizierbares als ASSUMED kennzeichnen. Keine erfundenen Quellen.
- Sprache der Dokumentation: Deutsch (siehe OQ-10).

## Verzeichnisstruktur

Siehe `README.md`. Abweichungen vom Mandatsvorschlag sind in `docs/08-handoffs/DECISION_LOG.md` (D-002) begründet.
