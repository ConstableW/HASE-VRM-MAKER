# Kompatibilitätsregeln

**Maßgebliche Quelle für die Regeln, nach denen Packages und Charaktere geprüft werden.** Status: `PROPOSED`. Umsetzung (Prototyp): `tools/validate/validate_packages.py`; Selbsttests belegen die Fehlerfälle.

| ID | Regel | Prüfzeitpunkt | Fehlercode (Prototyp) |
|---|---|---|---|
| C1 | `requires` müssen vorhanden sein und ihre Versionsbereiche erfüllen; `conflicts` dürfen nicht erfüllt sein | Installation/Auflösung | `[requires]`, `[conflicts]` |
| C2 | Spezies und Assets nennen `targets.bodyInterface`; das geladene Body-Package muss dieselbe Interface-ID liefern und den Bereich erfüllen | Installation, Charakter laden | `[interface]` |
| C3 | Ein Asset belegt nur Slots, die der Body definiert; eine Spezies-Komponente belegt den in `components.slot` genannten Slot | Installation | `[slot]` |
| C4 | `requiresCapabilities` ⊆ Body-Capabilities ∪ Spezies-Capabilities | Installation | `[capability]` |
| C5 | Exklusive Slots dürfen im Charakter nur einmal aktiv belegt sein; bei Layer-Slots entscheidet der Layer | Charakter bearbeiten/laden | `[slot-konflikt]` |
| C6 | Parameter in `defaults`, `limits`, `traits`, `presets`, Charakter müssen existieren (Body oder Morphset); `limits` liegen innerhalb des Body-Bereichs | Installation, Laden | `[parameter]`, `[limits]` |
| C7 | Asset mit `targets.species` ist nur mit den genannten Spezies kombinierbar | Charakter | `[species]` |
| C8 | Charakter-`lock` stimmt mit den aufgelösten Package-Digests überein (Reproduzierbarkeit) | Laden/Export | `[lock]` |
| C9 | Export-Profil-Prüfung: Für VRM 1.0 müssen die 15 Pflicht-Humanoid-Bones gemappt sein; Extra-Bones als Spring-Chains/Nodes; Lizenzen aggregierbar (restriktivste gewinnt) | Export | `export-pipeline.md` (K4) |

## Konfliktauflösung (Policy, PROPOSED)
- Zwei Komponenten wollen denselben Slot (z. B. Elf-Ohren vs. Katzenohren): Der Nutzer-Asset im Charakter gewinnt vor der Spezies-Komponente; die Spezies-Komponente wird *überschrieben*, nicht stillschweigend entfernt (Hinweis im UI).
- Capability fehlt: Asset bleibt installiert, ist aber für diesen Charakter gesperrt (mit Grund), statt zu scheitern.
- Interface-MAJOR-Sprung: Charakter lädt im „Reparaturmodus“ mit Migrationsangebot (Parameter-Umbenennungen laut `migrations`).

## Fehlermeldungen
Jede Regel liefert Regel-ID, Package, Ursache und Lösungsvorschlag (REQ-EXP-005). Der Prototyp liefert Regel-ID und Ursache.
