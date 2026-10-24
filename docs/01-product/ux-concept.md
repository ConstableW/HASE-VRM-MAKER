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

## Designsystem (Schritt A, umgesetzt 2026-10-24)
Tokens zentral in `app/hase-editor/src/theme.rs` (Hintergrund #080D15, Panel #101824, Sekundär #172231, Hover #233449, Aktiv #17477E, Akzent #58A6FF, Text #E9EFF7/#9BAABB, Linie #2B394A, Warnung #E5AA52, Fehler #E36C72). Deutsche Anzeigenamen für Parameter per Wortzuordnung (`label_de`), technische IDs nur in „Erweitert“. Shell: Kopfzeile (3 Arbeitsbereiche, Laden/Speichern, Bibliothek deaktiviert), linke Bereichsliste, Viewport, rechter Eigenschafteneditor, untere Presetleiste (Kacheln ohne Vorschaubilder). Navigationsstruktur und Regionen: `regions.rs`.
Bekannte Einschränkungen: keine Icons/Thumbnails, Panels nicht einklappbar, Lichter/Hintergrund-Presets fehlen, nur auf Linux/Lavapipe headless geprüft (nicht auf Windows).

## Körperbasis- und Speziesstudio (OD-027)
Vierter, übergeordneter Arbeitsbereich „Körper & Spezies“ (Authoring) neben den drei Charakterphasen. Links Speziesbibliothek (neu aus Charakter, duplizieren), rechts Reiter Übersicht · Körperbasis (Prüfstufen, OBJ-Importanalyse) · Werte & Grenzen · Presets · Kompatibilität · Paket (Version, Speichern). Architektur, Prüfstufen, Versionsstrategie und Stand: `../02-architecture/body-species-studio.md`.

Navigation (präzisiert): kreative Phasen „1 Charakter · 2 Outfit · 3 Finalisieren“ links; rechts getrennt unter „Werkzeuge“ der Authoring-Bereich „Körperbasen & Spezies“, daneben Bibliothek, Speichern, Laden.

## Umsetzungsstand Studios (2026-10-24, nach Windows-Verifikation der Kette)
Umgesetzt (Details und Grenzen: `../02-architecture/character-studios.md`): Augenstudio (Form-Presets mit Vorschaubildern, Iris innen/außen je Auge, Pupille, Reflexion/Limbus), anatomisches Brust-/Gesäßmodell, Haarstudio (Auswahl-Kacheln, Form, Farbe/Verlauf/Highlights), Haut (Ton, Glanz), Make-up, Sommersprossen, Muttermale/Tattoos/Narben per Klick, Charakterbibliothek mit Vorlagen/Favoriten/Varianten/Snapshots/Vergleich, Preset-Leiste mit Vorschaubildern, Viewport-Werkzeuge (Bereichs-Pick, Fokus), Licht-/Hintergrund-Presets.
Offen: Outfit-Studio-Vertiefung, Finalisieren (Posen, Animation, VRM-Export), Pupillenformen/Irismuster, Haar-Scheitel/Strähnen/Physik, Panel-Einklappen, Icons, Direktmanipulation per Griffe.

Finalisieren (2026-10-24): „Posen & Animation“ (VRMA-Clips mit Skinning-Vorschau) und „Export (VRM 1.0)“ (Profile, Metadaten, Vorabinformationen, Bericht mit Verlustliste, Hintergrund-Schreiben) sind umgesetzt; Details und Grenzen in `../02-architecture/character-studios.md`.
