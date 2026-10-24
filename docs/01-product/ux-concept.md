# UX-Konzept Character Studio 2.0

Maßgebliche Quelle für die Bedienstruktur. Status: **OWNER DECIDED als gestalterische Richtung** (Mockup und Vorgaben des Owners, 2026-10-24); Detailspezifikation (Komponenten, Typografie, Farbsystem) folgt iterativ mit der Implementierung. Bezug: OD-025, REQ-UX-*.

## Architekturvorgaben (verbindlich)
1. **Charakter ≠ Outfit.** Der Körper ist vollständig und kleidungsunabhängig. Keine eingebackenen Basisoutfits (umgesetzt: Standard-Charakter ist unbekleidet). Kleidung verdeckt nur in der Darstellung, Körpergeometrie bleibt erhalten.
2. **Drei Arbeitsbereiche:** (1) Charakter – Anatomie, Spezies, Gesicht, Körper, Augen, Haare, Haut; (2) Outfit – Kleidung, Passform, Layer, Material, Accessoires; (3) Finalisieren – Animation/Posen, Vorlagen, Varianten, VRM-Export. Wechsel jederzeit, keine Einbahnstraße; Outfit passt sich Körperänderungen an.
3. **Bedientiefen:** Gestalten (Presets/Kacheln) → Verfeinern (Regler je Region: Form, Proportionen, Position, Asymmetrie) → Direkt bearbeiten (Region im Viewport anklicken; geplant). Die vollständige Parameterliste bleibt als „Erweitert“.
4. **Anatomische Navigation:** Bereichsliste mit Kamerafokus je Region; manuelle Kamera bleibt wiederherstellbar.
5. **Pflichtumfang erste vollständige Version:** Export des unbekleideten Basischarakters, eigene editierbare Charaktervorlagen/Varianten, getrennte Presets (Gesicht/Körper/Haare/Stil), persistente Parameter unabhängig vom VRM-Export.
6. **Ehrlichkeit:** Jeder sichtbare Regler hat reale Funktion; nicht Unterstütztes ist als „noch nicht unterstützt“ markiert.
7. **Performance:** Neuberechnung nur bei Änderung (Dirty-Tracking), ruhender Charakter kostet nichts (OD-025).

## Umsetzungsstand (2026-10-24)
Erledigt: Arbeitsbereiche 1–3 (3 als Platzhalter), Bereichsliste Gesicht/Körper/Haare/Stil mit Kamerafokus, Region-Regler in Abschnitten, Erweitert-Ansicht, Outfit nach Kategorien, unbekleideter Standardcharakter.
Offen (Reihenfolge): Augenstudio (Iris/Pupille/Material – braucht eigene Shader/Texturen), anatomisches Brustmodell (neue Morphs/Kombinationen; MakeHuman liefert nur 6 Targets), Haarstudio (Regionen/Verläufe), Material-/Stileditor, Charakterbibliothek/Vorlagen/Snapshots, Preset-Kacheln mit Vorschaubildern, Direktmanipulation, Animation/Export, neutrale Vorschau-Abdeckung für den unbekleideten Körper, deutsche Parameterbezeichnungen.
