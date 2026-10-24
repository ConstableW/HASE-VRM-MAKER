# Körperbasis- und Speziesstudio

Maßgebliche Quelle für das Authoring von Körperbasen/Spezies (OD-027, REQ-STUDIO-*). Baut auf `species-system.md` und `package-system.md` auf. Status: Teile **implementiert** (siehe Tabelle), Rest PROPOSED.

## Ressourcen und Referenzen
Body Package ← Species Package ← Character Project; Asset Packages und Outfit Presets hängen am Charakter. Ein Charakter speichert nur seine Parameterwerte plus Referenzen (`species id + version`, `body id + version`, Asset-IDs); keine Duplikate der Geometrie. Eigene Ressourcen liegen unter `user-content/packages/<id>/hase-package.json` (selbes Format wie mitgelieferte Beispiele), laden und installieren folgen dem Package-System.

## Prüfstufen (ehrlich getrennt, `hase-core/src/validate.rs`)
Importiert → Geometrisch verwendbar (Indizes gültig, keine degenerierten Dreiecke) → Rig-kompatibel (Skelett + Gewichte vorhanden) → Morph-kompatibel (Targets/Parameter) → Als Körperbasis vollständig konfiguriert (Links/Rechts-Paare, alle Parameter einer Region zugeordnet, VRM-Humanoid-Fähigkeit) → Als Speziespaket validiert (nur bekannte Parameter, Grenzen konsistent, Defaults in Grenzen, keine Stub-Komponenten). Ein OBJ-Import ohne Rig/Morphs wird als nicht editierbar ausgewiesen, mit Aufbereitungsweg (Blender → glTF mit Rig/Shape Keys).

## Versions- und Migrationsstrategie (PROPOSED)
- SemVer je Ressource. Patch: keine Wertänderung. Minor: Ergänzungen (Presets, Merkmale, neue optionale Parameter) – bestehende Charaktere bleiben gültig. Major: entfernte/umgedeutete Parameter oder geänderte Topologie – Charaktere brauchen eine Migration.
- Das Projekt merkt sich die verwendete Ressourcenversion (Lock). Beim Laden mit abweichender Version: Kompatibilitätsprüfung; bei Major-Änderung Migrationsbericht (Parameter-Mapping, Verlustliste), Charakter bleibt unverändert, bis der Benutzer migriert. Alte Ressourcenversionen bleiben installierbar (Side-by-side).
- Noch nicht implementiert: Lock im Charakterprojekt, Versionsvergleich beim Laden, Migrations-Maps.

## Umsetzungsstand (2026-10-24)
| Funktion | Stand |
|---|---|
| Speziesbibliothek, Auswahl, Anwenden im Viewport | implementiert |
| Neue Spezies aus aktuellem Charakter (bewusst), Duplizieren | implementiert, als Paket gespeichert und neu geladen (Test) |
| Standardwerte, Grenzen, Presets bearbeiten, Version anheben, speichern | implementiert (Grenzen werden noch nicht in den Reglern erzwungen) |
| Prüfstufen für aktive Körperbasis, Spezies-Validierung | implementiert |
| OBJ-Importanalyse | implementiert (nur Analyse) |
| Körperbasis importieren/übernehmen (glTF mit Rig/Morphs), Regionen-/Symmetrie-Editor, Bone-Mapping-Prüfung, Package-Export als Archiv | offen |
| Mehrere Körperbasen gleichzeitig, nicht-humanoide Anatomien im Editor | offen (Architektur sieht sie vor: Topologie-Familien) |
| Beispiel-Packages mit älteren Parameter-IDs angleichen | offen |
