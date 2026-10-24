# ADR-0004: Body Archetype, Species und Topologie-Familien

- **Status:** PROPOSED (Owner-Freigabe offen)
- **Datum:** 2026-10-24
- **Betrifft:** REQ-ARCH-001…007, REQ-PKG-002/-003, REQ-CLO-011, R-01, R-02, OQ-20/-21
- **Evidenz:** `../02-architecture/species-system.md`, Testcandidate (Rig aus gemorphtem Körper, Proxy-Fits), `examples/species/`, Selbsttests

## Kontext
Drei Ebenen (Body/Species/Character) sollen nicht verdrahtet sein; Spezies sollen ohne Kernänderung entstehen; spätere Körperfamilien dürfen nicht ausgeschlossen sein.

## Alternativen
| Option | Pro | Contra |
|---|---|---|
| A. Klassenvererbung (Spezies erbt von Körper) | einfach | starr, Mischformen schwer, Kernänderung bei neuen Merkmalen |
| B. Ein universeller Mega-Body mit allen Merkmalen | wenig Packages | Topologie-Explosion, nicht erweiterbar auf andere Anatomien |
| **C. Body Package + Species-Overlay + Assets über Schnittstellenvertrag, Topologie-Familien T0/T1/T2 (gewählt, PROPOSED)** | Komposition, unabhängige Updates, Erweiterbarkeit | Vertragspflege, Kompatibilitätsprüfung nötig |

## Entscheidung (Empfehlung)
Option C. Eigener Body Archetype nur nach der Entscheidungsregel in `species-system.md` §3. Parameter sind namespaced und vom Body definiert; Spezies liefern Defaults/Limits/Traits/Komponenten; Mischformen über Traits, Komponenten und Morphsets.

## Begründung
Im Testcandidate erzeugen Rig (aus Gelenk-Würfeln des gemorphten Körpers), Skin-Weights und Kleidungs-Fits aus **einer** Basis konsistente Ergebnisse – T0 trägt Mensch/Elf/Demi ohne Zusatzarbeit. Zusätzliche Anatomie (Ohren, Schwanz) lässt sich als Bodyparts mit Spring-Chains ergänzen, ohne die Topologie zu ändern.

## Konsequenzen
- Body-Packages sind das zentrale Stabilitätsversprechen (Schnittstellenversion). 
- Offene Messungen: T1-Erweiterung (Schnauzen-Patch) und T2-Korrespondenz (SP-01/-02). 
- Nicht-humanoide Archetypes erfordern Export-Profile jenseits VRM (ADR-0003).
- Reversibel, solange kein Community-Package auf Schnittstelle 1.0 gebaut wurde.
