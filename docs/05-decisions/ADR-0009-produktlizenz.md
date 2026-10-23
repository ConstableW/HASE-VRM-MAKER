# ADR-0009: Lizenzmodell des Produkts

- **Status:** OWNER DECIDED (delegiert: Owner-Nachricht 2026-10-23 „lic entscheid selber best guess!“; Entscheidung von Claude nach bestem Wissen, jederzeit durch den Owner revidierbar)
- **Datum:** 2026-10-23
- **Betrifft:** OQ-02, C-06, REQ-LEG-001, REQ-PKG-011, R-11, R-16

## Kontext
Das Repo trägt Apache-2.0. Das Produkt soll ein offenes Package-Ökosystem tragen (REQ-PKG-001…009), unter Windows vertrieben werden und Abhängigkeiten sauber lizenzieren. Die Recherche favorisiert einen permissiv lizenzierten Stack (`../04-research/technology-evaluation.md`). Blender (GPL) soll nur als externer Prozess auftreten (`../04-research/feasibility.md` §3).

## Alternativen
| Option | Pro | Contra |
|---|---|---|
| A. Apache-2.0 für Code (Status quo) | Permissiv, Patentklausel, kompatibel mit MIT/BSD/Apache-Abhängigkeiten, kommerziell nutzbar, niedrige Hürde für Package-Autoren und Beitragende | Kein Copyleft-Schutz gegen proprietäre Forks |
| B. GPL-3.0 / AGPL-3.0 | Starker Copyleft-Schutz, Blender-`bpy` direkt nutzbar | Schließt proprietäre/kommerzielle Package-Autoren und Integrationen ab, kollidiert mit Slint-Royalty-free, erschwert Plugin-Ökosystem |
| C. Proprietär / Source-available | Volle Kontrolle, Monetarisierung | Widerspricht offener Package-Kultur; jede Abhängigkeit muss einzeln kommerziell vertretbar sein; keine Community-Beiträge |
| D. Open Core | Kern offen, Premium-Module geschlossen | Hoher Pflegeaufwand, Abgrenzung unklar, vor Marktkenntnis verfrüht |

## Entscheidung
**Option A mit klarer Trennung nach Artefaktart:**

| Artefakt | Lizenz |
|---|---|
| Anwendungscode, Core, Package SDK/CLI | Apache-2.0 |
| Schemas und Spezifikationen (`03-specifications/`, Manifest-/Species-/Asset-Schemas) | Apache-2.0 (Code/Schemas), Texte CC-BY-4.0 |
| Mitgelieferte Eigen-Assets (Basisbodies, Morphs, Standardspezies, Beispielpakete) | CC0-1.0 bevorzugt, sonst CC-BY-4.0; pro Datei per SPDX deklariert (REQ-PKG-011) |
| Community-Packages | Autor wählt Lizenz (SPDX Pflichtfeld); Editor zeigt und aggregiert sie |
| Dokumentation | CC-BY-4.0 |
| `assets/` (Fremdanimationen) | **nicht** unter Produktlizenz; Herkunft ungeklärt, nicht redistributierbar (OQ-03) |

Regeln daraus:
1. Kern-Abhängigkeiten nur aus {MIT, BSD, Apache-2.0, Zlib, MPL-2.0, Unlicense, CC0}; Ausnahmen per ADR (R9 in `feasibility.md`).
2. Kein GPL/AGPL/LGPL-Code im Hauptprozess; kein `import bpy` im Hauptprozess; Blender nur als separater optionaler Worker, nicht mitgeliefert.
3. Kein Autodesk FBX SDK, keine CGAL-GPL-Pakete.
4. Copyleft-Code (MakeHuman/MPFB) wird nicht übernommen; nur Konzepte und CC0-Assets mit geprüfter Dateiherkunft.
5. Beiträge Dritter nur mit Contributor-Zusicherung (DCO oder CLA, Wahl bei Öffnung des Repos), damit ein späterer Lizenzwechsel möglich bleibt.

## Begründung
Maximale Kompatibilität mit dem empfohlenen permissiven Stack und mit einem offenen Package-Ökosystem, ohne Vertriebs- oder Kommerzialisierungsoptionen zu verbauen. Keine Rechtsberatung; vor erstem Release juristisch prüfen (Patentklausel, Marken „HASE“, Asset-Lizenzen).

## Änderung durch OD-024 (2026-10-23)
Das Projekt ist Open Source und nicht kommerziell ausgerichtet; bestehende Open-Source-Projekte dürfen Grundlage sein. Daher:
- Die Regeln 1–4 oben sind **Standardpräferenz, kein Verbot mehr**. Permissive Lizenzen bleiben erste Wahl, weil sie die Kombination am einfachsten machen.
- Wird ein Fremdprojekt tatsächlich als Grundlage verwendet, wird die Lizenz **zu diesem Zeitpunkt** geprüft und im Provenienzregister vermerkt. Ist es Copyleft (z. B. GPL/AGPL bei MakeHuman/MPFB2, `import bpy`), darf das Produkt bzw. der betroffene Teil unter eine kompatible Copyleft-Lizenz gestellt werden; das entscheidet der Owner dann konkret für den Einzelfall (ADR-Ergänzung). Bis dahin bleibt Apache-2.0 der Default.
- Nicht kommerzielle Ausrichtung entfällt als Argument gegen Slint/Qt-Lizenzbedingungen nicht automatisch; Prüfung erst bei tatsächlicher Wahl (ADR-0002).

## Konsequenzen
- OQ-02 geschlossen; Lizenzbewertung der Abhängigkeiten (AI-14) entsperrt.
- Slint (GPL/Royalty-free-Bedingungen) und Qt (LGPL-Pflichten) sind weiterhin nur mit Zusatzprüfung zulässig.
- Reversibel: Wechsel zu restriktiverer Lizenz betrifft nur künftige Versionen; bereits veröffentlichte Apache-Versionen bleiben Apache. Daher Contributor-Regel vor Öffnung festlegen.
- Marke „HASE VRM MAKER“ wird nicht mitlizenziert (Markenrecht separat, nicht geprüft).
