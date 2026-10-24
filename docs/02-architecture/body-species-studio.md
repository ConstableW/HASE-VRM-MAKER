# Körperbasis- und Speziesstudio

Maßgebliche Quelle für das Authoring von Körperbasen/Spezies (OD-027, REQ-STUDIO-*). Baut auf `species-system.md` und `package-system.md` auf. Status: Teile **implementiert** (siehe Tabelle), Rest PROPOSED.

## Ressourcen und Referenzen
Body Package ← Species Package ← Character Project; Asset Packages und Outfit Presets hängen am Charakter. Ein Charakter speichert nur seine Parameterwerte plus Referenzen (`species id + version`, `body id + version`, Asset-IDs); keine Duplikate der Geometrie. Eigene Ressourcen liegen unter `user-content/packages/<id>/hase-package.json` (selbes Format wie mitgelieferte Beispiele), laden und installieren folgen dem Package-System.

## Prüfstufen (ehrlich getrennt, `hase-core/src/validate.rs`)
Importiert → Geometrisch verwendbar (Indizes gültig, keine degenerierten Dreiecke) → Rig-kompatibel (Skelett + Gewichte vorhanden) → Morph-kompatibel (Targets/Parameter) → Als Körperbasis vollständig konfiguriert (Links/Rechts-Paare, alle Parameter einer Region zugeordnet, VRM-Humanoid-Fähigkeit) → Als Speziespaket validiert (nur bekannte Parameter, Grenzen konsistent, Defaults in Grenzen, keine Stub-Komponenten). Ein OBJ-Import ohne Rig/Morphs wird als nicht editierbar ausgewiesen, mit Aufbereitungsweg (Blender → glTF mit Rig/Shape Keys).

## Versionsbindung und Migration (implementiert, Kern)
- Jedes Charakterprojekt speichert `body` und `species` mit `id`, `version` und **Inhalts-Hash** (SHA-256 des Manifests). Der Charakter speichert alle Parameterwerte selbst; eine Änderung der Spezies verändert sein Aussehen daher nie.
- Beim Laden werden Version/Hash geprüft (Körperbasis, Spezies; fehlende Spezies; unbekannte Parameter-IDs). Bei Abweichung erscheint ein Dialog (laden mit alter Bindung / laden und neu binden / abbrechen) – nie stilles Umschalten. Im Charakterstudio zeigt „Spezies & Presets“ Abweichungen und bietet „An installierte Version binden“ (Werte bleiben unverändert).
- SemVer-Regeln (PROPOSED): Patch = keine Wertänderung; Minor = Ergänzungen; Major = entfernte/umgedeutete Parameter oder geänderte Topologie → Migration nötig.
- Parameter-IDs: kanonisches Schema `mh:<kategorie-name>`; ältere Kurzformen werden über `Catalog::canonical` (Suffix-Regeln + Alias-Tabelle) aufgelöst. Beispiel-Packages sind migriert, Lock neu erzeugt.
- **Grenzen sind wirksam:** Spezies-`limits` begrenzen Regler und Setzvorgänge (Presets, Spezies-Defaults); Werte außerhalb (nach Spezieswechsel/Laden) werden gemeldet und können per Knopf in die Grenzen gebracht werden.
- Offen: Migrations-Maps für Major-Änderungen, Side-by-side mehrerer Ressourcenversionen, Daten-Hash der Körperbasis (aktuell nur Manifest).

## Umsetzungsstand (2026-10-24)
| Funktion | Stand |
|---|---|
| Speziesbibliothek, Anwenden, Neu-aus-Charakter, Duplizieren, Werte/Grenzen/Presets, Version, Speichern/Laden als Paket | implementiert |
| Prüfstufen Körperbasis, Spezies-Validierung | implementiert |
| Import-Analyse OBJ **und glTF/GLB/VRM** (Geometrie, Skelett mit Zuordnung auf die 15 VRM-Pflicht-Bones, Morph-Targets, Materialien) | implementiert (an Avatar A/B getestet) |
| Importiertes Modell als **Körperbasis-Entwurf** speichern (Paket, Status draft) | implementiert; Nutzung im Editor noch nicht |
| Aufbereitung in der Oberfläche: Regionen/Links-Rechts/Morph-Kategorien zuordnen, Bone-Mapping bearbeiten, Skinning prüfen; allgemeiner Body-Loader (glTF-Morphs → sparse Targets); Entwurf → vollständige Körperbasis → Spezies → Charakter | **offen (nächster großer Schritt des Studios)** |
| Nicht-humanoide Anatomien, mehrere aktive Körperbasen | offen |
