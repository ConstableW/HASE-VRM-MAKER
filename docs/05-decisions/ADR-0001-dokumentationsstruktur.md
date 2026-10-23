# ADR-0001: Dokumentationsstruktur und Statuskonventionen

- **Status:** PROPOSED
- **Datum:** 2026-10-23
- **Betrifft:** REQ-DOC-001…005, OD-015

## Kontext
Das Mandat (7.1) liefert eine Ausgangsstruktur, die „mit Begründung verbessert“ werden darf. Übergabefähigkeit und eine maßgebliche Quelle pro Thema sind verpflichtend.

## Problemstellung
Wie vermeidet die Struktur Parallel-Dokumentation und macht Zuständigkeiten pro Thema eindeutig?

## Alternativen
1. Mandatsstruktur unverändert.
2. Mandatsstruktur plus wenige Ergänzungen (gewählt).
3. Komplett neue Struktur (Wiki o. ä.) – verworfen: unnötiger Bruch zum Mandat.

## Entscheidung bzw. Empfehlung
Variante 2. Ergänzungen gegenüber dem Mandat:

| Ergänzung | Begründung |
|---|---|
| `docs/00-project/owner-decisions.md` | Einzige maßgebliche Quelle für OWNER DECIDED; verhindert, dass Owner-Beschlüsse in Prosa untergehen |
| `docs/00-project/source/` | Originalmandat als unveränderliche Quelle |
| `docs/02-architecture/investigations.md` | Verbindet Untersuchungsbereiche mit Ergebnissen/ADRs |
| `docs/05-decisions/README.md`, `ADR-TEMPLATE.md` | Register und einheitliches Format |
| `docs/06-planning/functional-decomposition.md` | Komponentenliste mit REQ-Abdeckung |
| `AGENTS.md` im Root | Einstieg und harte Regeln für Agenten |

Statuslabel: OWNER DECIDED / PROPOSED / ASSUMED / RESEARCHED / BLOCKED, ergänzt um Kennzeichen [R]/[A] in Rechercheberichten. ID-Schemata siehe `AGENTS.md`.

## Konsequenzen
Neue Dokumente müssen im Kopf ihre maßgebliche Rolle nennen. Reversibel; rein organisatorisch.
