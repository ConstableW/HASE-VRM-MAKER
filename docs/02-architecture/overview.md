# Architekturüberblick (Erstentwurf)

**Maßgebliche Quelle für die Zielarchitektur auf Systemebene.** Status: durchgehend `PROPOSED`, technologieneutral. Technologiewahl: ADR-0002 (offen). Komponenten: `../06-planning/functional-decomposition.md`. Evidenz: `../04-research/feasibility.md`, `technology-evaluation.md`.

## Leitprinzipien (PROPOSED)

| # | Prinzip | Begründung / Beleg | REQ |
|---|---|---|---|
| P1 | **Parameter sind Wahrheit, Mesh ist abgeleitet.** Der Projektzustand speichert Archetype-/Species-Referenzen, Parameter, Asset-Slots und Delta-Layer; das evaluierte Mesh ist reproduzierbar daraus berechenbar | Voraussetzung für Nichtdestruktivität, Undo, Determinismus | ARCH-008/-009, EXP-007 |
| P2 | **Daten vor Code.** Body-/Species-/Asset-Packages sind deklarativ (Manifeste + Binärdaten). Ausführbarer Code nur sandboxed und optional | Sicherheit (R-09); Kernänderungsfreiheit | PKG-006, -013 |
| P3 | **Komposition über Verträge.** Body-Packages veröffentlichen eine Schnittstelle (Topologie-ID, Morph-Namensraum, Landmarken, Skeleton-Profil, Slots); Species und Assets deklarieren, gegen welche Schnittstelle sie gebaut sind | Entkoppelt Ebenen; unabhängige Updates (`references.md` B2.3) | ARCH-001/-003, PKG-005/-012 |
| P4 | **Zwei Kompatibilitätsstufen der Topologie.** (a) gleiche Topologie-Familie → direkte Delta-Morphs, Weights, Basismesh-gebundene Kleidung; (b) abweichende Topologie → Korrespondenz über Landmarken/Deformation Transfer (später). Kriterienkatalog in `species-system.md` (K2) | Delta-Morphs brauchen identische Topologie (`feasibility.md` §4) | ARCH-006, CLO-011 |
| P5 | **Export-Profile statt Humanoid-Annahmen im Core.** VRM 1.0 verlangt 15 Humanoid-Bones und ein Mapping [R]; nur das VRM-Profil kennt diese Regel. Nicht-humanoide Archetypes exportieren über ein generisches glTF-Profil | Nicht-humanoide Archetypes bleiben möglich | ARCH-002/-007, EXP-001 |
| P6 | **Extra-Anatomie als Spring-Chains/Extra-Nodes.** Ohren, Schwänze: keine Humanoid-Bones, lineare Spring-Chains; nicht retargeten | VRM-Spec, VRMA rein humanoid [R] | ARCH-005, RIG-007, ANI-007 |
| P7 | **Ein Command-Modell für beide Modi.** Designer- und Expert-Werkzeuge erzeugen dieselben undo-fähigen Befehle auf demselben Zustand | OD-010 | ARCH-008, UX-009 |
| P8 | **Headless-fähiger Core.** Evaluation, Validierung, Export laufen ohne UI (CLI/CI, Package-SDK, Golden-Tests) | Reproduzierbarkeit, Tests | QA-001, EXP-007 |
| P9 | **Validierung als Querschnitt.** Jede Regel hat ID, Schweregrad, Erklärung, Fix-Vorschlag; Export erzeugt einen Report inkl. verlustbehafteter Schritte | „Kreative Freiheit + Fehlererkennung“ | BODY-010, EXP-005/-006 |
| P10 | **Provenienz und Lizenz durchgängig.** Jede Datei in einem Package trägt Herkunft/SPDX; Export aggregiert in VRM-`meta`/`thirdPartyLicenses` | VRM-Meta-Pflichtfelder [R] | PKG-011, LEG-002 |

## Verarbeitungskette (PROPOSED)

```
Packages ──resolve/validate──► Registry (Archetypes, Species, Assets)
Projektzustand (Parameter, Slots, Delta-Layer, Overrides)
  └► Morph-Evaluation ► Joint-Fitting/Skinning ► Asset-Fitting ► Materialauflösung
        └► Character-Intermediate ─► Viewport (Preview, Simulation live)
                                  └► Export-Profil (VRM1 | glTF | FBX) ► Validierung ► Report
```

Hinweise: Körper-Morphs werden beim Export **eingebacken**; nur Gesicht/Ausdruck bleiben als VRM-Expressions (`feasibility.md` §1.4). Delta Mush u. ä. nur als Bake.

## Stack-Tendenz (PROPOSED, nicht entschieden – ADR-0002)

Die Recherche (`technology-evaluation.md`, Bewertungsmatrix §9–10) favorisiert als Hypothese einen **Rust-Core (wgpu) mit egui-UI und wasmtime für Package-Code**, wegen durchgehend permissiver Lizenzen, eingebauter Accessibility (AccessKit) und Windows-Backends. Größte Lücken: **kein belegter Rust-VRM-Exporter**, nachlaufende Docking-Crates, viel Eigenbau (MToon, Sculpting). Alternativen: **Godot 4 + godot-vrm** (schnell, aber SpringBone-Export fehlt, Add-on-Pflege unklar) und **Tauri + three.js/three-vrm** (Exporter ungeklärt, Sculpting-Performance). Unity/Unreal/Qt: aus Lizenz-/Kostengründen nicht empfohlen. Die Tendenz ist an OQ-02 (Produktlizenz) gekoppelt und braucht SP-08, SP-07 vor einer Entscheidung. Unbelegt: Aktualität einiger Versionsangaben (siehe Lücken im Bericht).

Blender: ausschließlich externer, optionaler Worker-Prozess (kein `import bpy` im Hauptprozess wegen GPL) – `feasibility.md` §3.

## Offene Architekturfragen

Siehe `investigations.md` und `../06-planning/open-questions.md` (OQ-20…25).
