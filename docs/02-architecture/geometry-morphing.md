# Geometrie, Morphing und Fitting (Ist-Stand der Implementierung)

Maßgebliche Quelle für das implementierte Morph-/Fitting-Verfahren. Status: **RESEARCHED** (gemessen/implementiert in `app/hase-core`), Architekturentscheidung weiter PROPOSED (ADR-0002). Bezug: REQ-PERF-*, R-01, R-03, R-05.

## Morphing (`body.rs`, `params.rs`, `eval.rs`)
- Basis: MakeHuman hm08 `base.obj` (Körper 13.380 Vertices), 610 Targets als sparse Deltas (CSR: Offsets/Indizes/Deltas), Einheit dm → m (×0,1). Binärcache `.hase-cache/morphs.hasebin` (Magic `HASEMRF1`), Parsing parallel (rayon).
- Parameterkatalog aus `target.json`: ~294 Parameter (Signed/Unit/Macro), ID-Schema `mh:<kategorie-name>`; Makros `mh:macro-gender/age/race-*` werden in Target-Gewichte aufgelöst (`target_weights`, `macro_age_weights`).
- Evaluator: gewünschte Gewichte vs. angewendete; nur geänderte Targets werden inkrementell gescattert (Vollberechnung bei >120 Änderungen oder Drift). Messung: 0,14–0,33 ms pro Änderung.
- Symmetrie: links/rechts-Paare werden gespiegelt gesetzt (Toggle). Undo/Redo über Snapshot-History mit Coalescing.
- Bodenabstand: `floor_off` = −min(y) des Körpers, nach jedem Morph neu (Mesh-Ursprung ≈ Becken).

## Fitting (`fit.rs`)
- `.mhclo`-Proxys (Haare, Kleidung, Brauen, Wimpern, Augen): barycentrische Bindung an Basisvertices + Offset skaliert mit aktuellen Körperdimensionen. 0,2–0,85 ms pro Asset.
- Verdeckte Körperflächen werden nicht gerendert (Coverage-Culling im Export-Pfad, siehe Avatar-B-Skripte).

## Offen / geplant
- Delta-Layer für Expert-Modus (R-05), Kollisionskorrektur nach Fit (R-03), Skinning/Animation im Viewport, Export aus dem Editor, Fitting asynchron/inkrementell auf Worker.
- Paket-Parameter-IDs der Beispiel-Packages (`examples/species/`) müssen auf das Katalog-Schema `mh:<kategorie-name>` angeglichen werden.
