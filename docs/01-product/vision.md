# Produktvision und Umsetzungslinie

Maßgeblich: OD-028 (`../00-project/owner-decisions.md`). Dieses Dokument übersetzt die Vision in eine Umsetzungslinie; es ist **kein** weiteres Konzeptpapier, sondern die Reihenfolge der Arbeit. Status der Architektur-Aussagen: **PROPOSED** (Freigabe durch den Product Owner steht aus, OD-018).

## Leitgedanke
Ein Charakter besteht aus **Körperbasis** (Geometrie + Rig + Parameterkatalog), **Spezies** (Merkmale, Grenzen, Komponenten), **Stilschichten** (Augen, Haut-/Haarmaterial, Proportionen) und **Teilen** (Haare, Ohren/Schwänze, Brille, Kleidung). Der Editor gestaltet immer diese Schichten – nie ein fertiges Modell. Ausgabe ist eine Abbildung dieser Schichten auf ein Zielformat.

## Körperbasis-Arten (PROPOSED)
| Art | Parameter wirken über | Stand |
|---|---|---|
| `morph` (MakeHuman-artig) | vorberechnete Morph-Targets, inkrementelle Verformung | umgesetzt (realistisch; Basis „hm08“) |
| `parametric` (Anime/VTuber-Basis) | Landmarken + weiche Bereichsoperationen + Knochenskalierung auf einem Basismesh (z. B. VRoid-artiges VRM als **Basisgeometrie**, nicht als Fertigmodell) | Kern umgesetzt (`hase-core/src/avatar.rs`): Gesicht (Augen links/rechts getrennt, Brauen, Nase, Mund, Kinn/Kiefer, Ohren, Wangen), Körper (Größe, Proportionen, Brust, Taille, Hüfte, Gesäß, Gliedmaßen), Farben je Materialgruppe |
| `authored` | im Körperbasisstudio erstellte/geänderte Basis | Studio vorhanden (Import → Prüfung → Spezies) |

Beide Arten durchlaufen denselben Weg: Körperbasis wählen → Charakter gestalten → Outfit → Finalisieren/Export. Eigene Basen stammen aus Dateien des Owners und bleiben lokal (Lizenzklarheit, OD-024); das Repository liefert keine fremden Basismeshes aus.

## Stil und Überführung (langfristig)
Stil ist eine Eigenschaft der Schichten, nicht des Programms: Augenmodell (flächig/anime vs. sphärisch/realistisch), Haarmodell (Karten/Strähnen), Material (Toon/MToon vs. PBR), Proportionsprofil. Eine Überführung realistisch → Anime ist eine Abbildung dieser Schichten (Proportionsprofil auf Parameter, Augen-/Haar-/Materialtausch), keine Bildfilterung. Erste Stufe: Stilvorlagen, die Parameter, Augen, Material und Haare gemeinsam setzen.

## Reihenfolge (nach Nutzen für ein benutzbares Gesamtprodukt)
1. Erschaffen auf Anime-Basis in der bestehenden Oberfläche (Gesicht/Körper/Farben, Augen getrennt) – **begonnen**.
2. Charakter dauerhaft speichern (Basis-Referenz + Parameter + Farben + Teile), Vorlagen/Varianten wie bei MakeHuman-Charakteren.
3. Haare erstellen (Karten-/Strähnengenerator mit Farbe/Verlauf/Länge/Volumen; Spring-Bone-Export).
4. Spezies-Merkmale auf Anime-Basis (Tierohren, Schweif, spitze Ohren) als Teile mit Ankerknochen.
5. Outfitstudio für beide Basisarten; Zubehör (Brille …).
6. Ausgabeformate: VRM zuerst; glTF/GLB (PBR) und weitere über eine Format-Schicht.
7. Stilvorlagen/Überführung.
