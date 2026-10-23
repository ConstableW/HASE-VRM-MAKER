# ADR-0010: Beschaffung der Basis-Content-Assets

- **Status:** OWNER DECIDED (Owner-Nachricht 2026-10-23: „free to use / selbstgemacht und AI generiert und Community später“); Ausgestaltung unten ist PROPOSED
- **Datum:** 2026-10-23
- **Betrifft:** OQ-05, R-13, R-11, REQ-ARCH-004, REQ-FACE-001, REQ-PKG-011, REQ-LEG-002, C-03

## Entscheidung des Owners (OD-021)
Basis-Content (Body-/Face-Basismeshes, Morph-Sets, Haare, Augen, Beispielkleidung, Materialien, Standardspezies) kommt aus drei Quellen, **ohne kostenpflichtige Beschaffung**:
1. **Frei nutzbare Assets** (CC0 bzw. gleichwertig permissiv, Herkunft je Datei geprüft),
2. **Selbst erstellte Assets**,
3. **KI-generierte Assets**,
und später aus der **Community** (über das Package-System).

## Ausgestaltung (PROPOSED)
| Thema | Vorschlag |
|---|---|
| Zulässige Fremdlizenzen | CC0-1.0, Unlicense, MIT/BSD für Daten; CC-BY nur mit automatischer Attribution im Credits-/VRM-Meta-Export. **Nicht** CC-BY-NC, -ND, -SA, GPL/AGPL-Daten ohne ADR |
| MakeHuman-Daten | Nur CC0-Anteile ab 2020 und nur mit Dateiprüfung (`../04-research/references.md` C2); kein Code |
| Provenienzregister | Pflicht pro Asset: Quelle/URL, Urheber, Lizenz (SPDX), Abrufdatum, Hash, bei KI: Werkzeug+Version, Datum, Prompt/Eingaben, Nachbearbeitung durch Menschen |
| Basismeshes | Selbst erstellt (z. B. in Blender, extern) – tragende Assets mit Topologie-/Morph-Anforderungen sollten **keine** direkte KI-Meshausgabe sein [ASSUMED: KI-Meshes liefern keine kontrollierte Topologie/Edge-Flow für Morphs und Rigging; per Spike prüfen] |
| KI sinnvoll für | Texturen, Konzeptzeichnungen, Muster, Hauttöne, Augen-/Iris-Texturen, Kleidungsdesign-Vorlagen; Meshes nur als Ausgangsform mit Nachbearbeitung |
| KI-Assets | Zulässig (OD-022/OD-024); keine weitere Rechtsdiskussion. Provenienz wird vermerkt. Keine Reproduktion geschützter Assets der Referenzprodukte (REQ-PROD-006) |
| Kosten | Kostenfreie Werkzeuge bevorzugt; kostenpflichtige KI-/Cloud-Dienste weiterhin nur mit Freigabe (C-03) |
| Community | Über Asset-/Species-Packages mit Lizenzpflichtfeld und Prüf-Validator; Community-Inhalte sind nicht Teil der Standardauslieferung ohne Lizenzcheck |
| Standardspezies | Mensch, Elf, Katzen-Demi, Hunde-Demi als eigene Packages auf gemeinsamer humanoider Topologie-Familie (Details K2) |

## Konsequenzen
- OQ-05 geschlossen; R-13 bleibt (Aufwand/Qualität der selbst gebauten Basis), aber Kostenrisiko entfällt.
- Neue offene Punkte: OQ-26 (Rechtsstatus KI-Assets und Werkzeugwahl), OQ-27 (Wer erstellt das Basismesh: Owner extern in Blender oder Agent-gestützt/prozedural? Agenten können Meshes nur skriptgesteuert erzeugen, keine künstlerische Modellierung [ASSUMED]).
- Provenienzregister wird Bestandteil der Asset-Pipeline (`asset-pipeline.md`, ADR-0007).
- Reversibel: Kauf/Lizenzierung bleibt später möglich (Freigabe nötig).
