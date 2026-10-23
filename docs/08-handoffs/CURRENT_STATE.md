# CURRENT_STATE

*Zuletzt aktualisiert: 2026-10-23 (Arbeitslauf 1)*

## Aktuelle Projektphase
Discovery & Architecture (Konzeptphase). **Kein Produktcode.** Owner-Abnahme nicht erfolgt.

## Zuletzt abgeschlossene Arbeiten (Arbeitslauf 1)
- Mandat konsolidiert: Vision, Scope, Randbedingungen, Glossar (`docs/00-project/`).
- Anforderungsmatrix mit IDs und Prioritäten (`requirements.md`, 146 Anforderungen).
- Owner-Entscheidungen OD-001…018 (`owner-decisions.md`).
- Funktionale Zerlegung F-01…F-24, Risikomatrix R-01…R-16, Untersuchungsübersicht AI-01…17, offene Fragen OQ, Konzeptplan K0…K8 und Spikes SP-01…09 (`docs/06-planning/`, `docs/02-architecture/investigations.md`).
- Drei Recherchestränge (Technologie/Engine/UI; Formate/Bibliotheken/Geometrie; Referenzprodukte/Packages/Recht) → `docs/04-research/{technology-evaluation,feasibility,references}.md`.
- Erster Architekturentwurf `docs/02-architecture/overview.md`, ADR-0001 (PROPOSED), ADR-Register.

## Verifizierter Istzustand
- Repo enthält: `LICENSE` (Apache-2.0), Stub-README (jetzt ersetzt), `assets/animation-BVH` (118 BVH) und `assets/animation-vrma` (118 VRMA) aus dem SillyTavern-Pack `test157t/VRM-Assets-Pack-For-Silly-Tavern`.
- **Recherche ergab:** dieses Pack hat keine Lizenzdatei; Animationen stammen laut README von Mixamo (Weitergabe als Pack laut Adobe-FAQ nicht zulässig) → nicht redistributierbar, interne Testdaten (OQ-03, R-11).
- **Git:** Freigabe durch Owner erteilt (OD-019). Stand ist auf Branch `claude/gifted-ptolemy-tdh9vc` committet und gepusht (kein PR).
- Recherchequalität: Teile stützen sich auf Suchzusammenfassungen; Primärquellen einiger Anbieter (VRoid, Blender, Khronos, SPDX u. a.) waren nicht abrufbar. [R]/[A]-Markierungen in den Berichten beachten. Die Berichte wurden von Subagenten erstellt und von Claude nur gelesen/integriert, nicht quellenweise nachverifiziert.

## Aktive Aufgaben
Keine laufenden Hintergrundaufgaben.

## Offene Entscheidungen
Siehe `docs/06-planning/open-questions.md`. Dringlichste: OQ-11 (Commit/Push), OQ-02 (Produktlizenz), OQ-05 (Basis-Content, kostenpflichtig), OQ-01 (Stack, nach weiterer Konzeptarbeit), OQ-04 (Spikes).

## Wichtigste Erkenntnisse aus der Recherche (RESEARCHED/PROPOSED, siehe Berichte)
1. VRM 1.0 verlangt 15 Humanoid-Pflicht-Bones und genau ein Humanoid-Mapping; Demi-Formen sind über Extra-Nodes/Spring-Chains konform, echte Nicht-Humanoide nicht → Export-Profile (P5).
2. VRMA ist rein humanoid; Ohren/Schwanz nicht animierbar → Spring Bones.
3. `import bpy` im Hauptprozess = GPL; Blender nur als optionaler externer Worker.
4. Permissive Lib-Landschaft genügt (cgltf/fastgltf, ufbx, meshoptimizer, ozz, Basis Universal); FBX SDK und CGAL-GPL vermeiden; gltf-rs seit 2024-05 ohne Update.
5. Kein belegter Rust-VRM-Exporter; Godot-VRM-Export ohne Spring Bones; UniVRM hat den reifsten Export, aber Unity-Lizenzrisiko.
6. Realistische Verfahren: Delta-Morphs, RBF-Landmarks, Basismesh-gebundene Kleidung, nearest-surface Weight-Transfer, Pin-and-Drag per regularisiertem Least-Squares.

## Bekannte Risiken und Blocker
`docs/06-planning/risks.md`. Top: R-03 Kleidungsfitting, R-01 speziesübergreifende Geometrie, R-05 Nichtdestruktivität vs. Expert-Edit, R-13 Content-Beschaffung. Blocker: OQ-02 blockiert die finale Lizenzbewertung.

## Nicht erledigt von der Konzeptphase (Abschluss siehe Mandat 12)
Feature-Katalog, User Journeys, UX-Konzept/Screens, Teststrategie, Spezifikationen (Formate/Schemas), Beispielpakete, Species-/Package-System-Detail, Geometrie/Rigging/Rendering/Asset-/Export-Detail, ADR-0002…0008. Plan: `docs/06-planning/roadmap.md` (K1 teilweise, K2–K8 offen).

## Nächste Schritte
`NEXT_ACTIONS.md`.
