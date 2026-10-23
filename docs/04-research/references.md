> **Status:** Rechercheergebnis (delegierter Analyseagent, 2026-10-23), von Claude integriert, **nicht** vom Owner geprüft. Kennzeichnung im Text: [R] = RESEARCHED, [A] = ASSUMED. Abgeleitete Empfehlungen sind PROPOSED. Entscheidungen erfolgen ausschließlich per ADR (`../05-decisions/`).
> **Maßgebliche Quelle für:** Referenzanalyse, Package-System-Vorbilder, Rechtshinweise inkl. Herkunft des Animationspakets

# HASE VRM MAKER – Recherche: Referenzen, Package-Systeme, Rechtliches

Stand: 2026-10-23. Kein Rechtsrat. Nichts implementiert, nichts übernommen.

**Kennzeichnung:** `[R]` = RESEARCHED (Quelle in der Aussage, abgerufen 2026-10-23, meist über Websuche-Zusammenfassungen; Primärseiten waren teils nicht erreichbar). `[A]` = ASSUMED (Fachwissen/Schlussfolgerung, nicht in dieser Session belegt). Schwach belegt = `[R-schwach]` (Sekundärquellen, Guides, Foren).

**Einschränkung der Recherche:** Direkter Abruf von vroid.com, developer.vroid.com, docs.blender.org, openusd.org, spdx.org, docs.unity3d.com, registry.khronos.org und wasi.dev scheiterte (DNS/Proxy). Aussagen dazu stützen sich auf Suchergebnis-Zusammenfassungen. Vor Entscheidungen Primärtexte lesen.

---

## A) Referenzanalyse

### A1. Tabelle je Produkt

| Produkt | Lernen | Vermeiden | Schluss für HASE |
|---|---|---|---|
| **VRoid Studio** | Stilisierte Anime-Avatare, Export als VRM 1.0, ab v2.0 Dress-up-System mit XWear-Format, Auto-Fitting + manuelle Knochenanpassung [R: VRoid-Notices v2.0.0/2.0.2]. Lizenz wird pro Export gesetzt (kommerziell Firma/Einzelperson, Weitergabe veränderter Modelle, politische/antisoziale Darstellung) [R-schwach: Blender-Artists-Thread, Booth-Listings; VRM-Lizenzfelder siehe VRM-Spec]. | Keine Offenheit für Dritt-Erweiterung im Studio selbst [A]. Kleidung als „Textur-Malen" statt echter Parametrik [A]. | VRM-Export mit vollständigen Lizenz-Metadaten als Pflicht; Kleidung als eigenes, austauschbares Asset (ähnlich XWear) mit Fit-Schritt. |
| **MakeHuman / MPFB2** | Parametrische Basis über Targets (Morph-Deltas), Proxies für Kleidung/Haare, Rigify-Unterstützung in MPFB [R: MPFB-Release-Notes, Blender-Extensions-Seite]. Lizenz: Code AGPL (MakeHuman) bzw. GPL-3.0-or-later (MPFB-Add-on); Assets (Basismesh, Targets, Proxies) seit 2020 CC0, Ausgabe ebenfalls CC0 [R: makehumancommunity.org License/FAQ 2020, Extensions-Seite]. Vor 2020 galt für Targets AGPL – historisch [R]. | Code nicht übernehmen (Copyleft). Alte Assets (vor 2020) und Drittbeiträge nicht ungeprüft nutzen [R/A]. Kleidung bricht nach Slider-Änderung (FAQ-Eintrag existiert) [R-schwach: nur FAQ-Titel gesehen]. | Das Targets-Prinzip (additive Deltas, kombinierbar, austauschbar) nachbauen; Assets nur mit geprüfter CC0-Herkunft je Datei. Proxy-Fitting als Konzept, nicht als Code. |
| **Reallusion Character Creator 4** | Morph-Slider-Authoring (Head/Body), SkinGen, großes Content-Ökosystem [R-schwach: Reallusion-Manual-Titel, Download-Seite]. Export-Lizenz pro Content-Pack nötig; Content darf nicht zum öffentlichen Download weitergegeben werden [R-schwach: älterer Forum-Post, aktuelle EULA nicht geprüft]. | Lizenzfragmentierung pro Pack, die Nutzer überrascht (Export blockiert) [R]. | Lizenz-/Export-Rechte pro Paket sichtbar und vorab prüfbar machen („Export-Check" vor Auswahl). |
| **Unreal MetaHuman** | Hohe Qualität, Mesh-to-MetaHuman-Idee (eigenes Mesh → Rig) [R-schwach]. Ab UE 5.6 gilt die Unreal-Engine-EULA; Nutzung mit jeder Engine/DCC laut offizieller Lizenzseite, Umsatzgrenze 1 Mio. USD [R: metahuman.com/license, unrealengine.com/eula/mhc]. Auto-Rigging/Textursynthese laufen teils auf Epic-Servern [R-schwach: Medium-Blog]. | Cloud-Abhängigkeit; Lizenz an Epic-EULA gebunden; Realismus-Fokus passt nicht zu VRM-Stil [A]. | Offline-first bleiben; nur Konzept „Preset-Mischung + Regler" lernen. Kein Asset/Code übernehmen. |
| **Daz Studio / Genesis 9** | Figur-Konzept: Basisfigur + Morphs; Kleidung folgt per „Auto-Follow/Fit To" [R-schwach: Store-Listings erwähnen Auto-Follow und „Dedicated Fit"-Morph-Packs pro Kleidungsstück]. Zusatz-Morphs pro Kleidungsstück als Korrektur [R]. Lizenzdetails nicht gefunden [R: Lücke]. | Pro-Kleidung-Korrektur-Morphs führen zu Paketexplosion und Abhängigkeitsketten [A]. | Fit-Daten (Korrekturmorphs) als optionaler, versionierter „Fit-Layer" entkoppeln; Basisfit automatisch, Korrekturen optional. |
| **Blender (DCC)** | Modifier ändern Darstellung ohne die Geometrie zu zerstören; Reihenfolge im Stack beeinflusst Ergebnis; „Apply" backt [R: Blender-Manual Modifiers Intro]. Geometry Nodes ermöglichen eigene Modifier [R]. Shape Keys = gespeicherte Vertex-Zustände mit Blend-Werten [A, nicht in Session belegt]. | Destruktives „Apply" als Default; Stack-Reihenfolge-Fallen [A]. | Kern-Datenmodell: Basis + geordnete, nichtdestruktive Schichten (Morphs, Proxies, Texturen); „Bake" nur beim Export. |
| **AION 2** | Start mit Körpertyp (2 Typen), Presets in Kategorien (Face, Hair, Style, Body Shape), >200 Einstellungen, rotierbare Echtzeit-Vorschau, 8 Speicherslots + 8 Download-Slots; spätere Änderung per Ticket [R-schwach: Guides allthings.how, exitlag, timesaver.gg; Slider-Namen nicht offiziell]. Symmetrie/Spiegeln: nicht gefunden [R: Lücke]. | – | Preset → Feintuning, Slots/Teilen als Teil der UX. |
| **Guild Wars 2** | Hybrid: Presets wählen, dann einzelne Slider (z. B. Nasenbreite); Gruppen: Körper (Größe, Statur), Kopf (Frisur, Gesicht, Ohren), Detail für Augen/Nase/Mund/Wangen/Kiefer/Kinn/Kopfform [R: wiki.guildwars2.com Physical appearance]. Nachträgliche Änderung über Kits [R]. | Aussehen bei Erstellung „fest" – Frust bei späterer Änderung [A]. | Gesichtsgruppen als Navigations-Gliederung; Änderungen jederzeit erlauben (Konzeptprojekt = Studio, nicht Spiel). |

Hinweis: Stand von VRoid-Details (z. B. exakte Lizenztexte) ist nicht aus den Primärseiten verifiziert.

### A2. VRM-Format-Bezug
- VRM 1.0 (Spezifikation September 2022) ist in Extensions aufgeteilt: `VRMC_vrm` (Humanoid, Meta, Expressions), `VRMC_springBone`, `VRMC_materials_mtoon`; Lizenz-Flags wurden granularer, VRM Public License 1.0 [R: vrm.dev/vrm1, moguravr]. Exakte Feldnamen aus dem Repo vrm-c/vrm-specification prüfen [R: Lücke].
- Schluss: HASE exportiert VRM 1.0; Lizenzmetadaten des Charakters werden aus den Lizenzen aller enthaltenen Pakete abgeleitet (siehe B/C).

### A3. VRoid-SDK / Nutzungsbedingungen
- VRoid SDK/Hub-API: ermöglicht Apps, auf VRoid-Hub-Modelle zuzugreifen [R-schwach: MoguraVR]. Offizielle Bedingungen konnte ich nicht abrufen (developer.vroid.com nicht erreichbar) [R: Lücke]. Für Unity-Workflows wird UniVRM genutzt, nicht ein VRoid-SDK [R-schwach: Community-Template].
- Exporte: Frühe Aussage „frei nutzbar" stammt aus 2019 vor formalen Bedingungen [R-schwach]; heute pro-Modell-Lizenzeinstellung. Dritt-Sample-Items tragen die Lizenz ihrer Ersteller [R-schwach].
- Grenzen: Studio ist geschlossen und nur eingeschränkt erweiterbar [A].

### A4. UX-Muster (überwiegend [A]; durch GW2/AION-Beobachtung gestützt)
- **Direkte Manipulation:** Punkte/Handles am Modell ziehen statt nur Slider; zusätzlich Slider als Präzisionsweg. Mapping Handle → Morph-Gewichte muss definiert sein (z. B. Gruppen-Influenz-Radius) [A].
- **Wizard vs. frei:** GW2/AION führen Presets → Detail (de-facto geführt) [R-schwach]. Empfehlung: zwei Modi auf demselben Datenmodell – „Geführt" (Preset-Auswahl, wenige Regler) und „Frei/Pro" (alle Regler, Layer-Stack) [A].
- **Undo/Verlauf:** Befehlsbasierte Historie (Command Pattern) mit Zeitstrahl und benannten Snapshots (analog AION-Speicherslots [R-schwach]); Slider-Drags zu einem Eintrag zusammenfassen [A].
- **Symmetrie:** Standard an, pro Gruppe abschaltbar; in AION-Quellen nicht belegt [R: Lücke], daher [A].
- **Vorschau:** rotierbar, Beleuchtungs-Presets, Vergleichsansicht Vorher/Nachher [A].

---

## B) Package-System-Vorbilder

### B1. Recherche-Befunde
| Thema | Befund | Beleg |
|---|---|---|
| **PubGrub** | Von Natalie Weizenbaum für Dart/pub entwickelt (2018); SAT-inspiriert, Backtracking mit Konfliktableitung, erzeugt verständliche Fehlerketten („Because X depends on Y…"). Implementiert u. a. in Rust-Crate `pubgrub` (genutzt von uv), Bundler, Poetry. | [R: dart.googlesource.com pub/doc/solver.md, docs.rs pubgrub] |
| **Factorio-Manifest** | `info.json` mit `dependencies`: Präfixe `!` (inkompatibel, ohne Version), `?` (optional, bei Versionsverstoß wird Mod deaktiviert), Operatoren `< <= = >= >`; Standard `["base"]` | [R: lua-api.factorio.com mod-structure, v2.0.76] |
| **glTF-Extensions** | `extensionsUsed` vs. `extensionsRequired`; Viewer ohne Support für eine required-Extension sollen das Modell ablehnen, used-only dürfen ignoriert werden (Fallback) | [R-schwach: Khronos-Extensions-README via Mirrors, Unreal-Doku; Spec-Text selbst nicht abgerufen] |
| **OpenUSD-Komposition** | Layers + Composition Arcs mit Stärkenordnung LIVERPS (Local, Inherits, Variants, (Relocates), References, Payloads, Specializes); stärkere „Opinions" überschreiben schwächere, ohne Originaldaten zu ändern → nichtdestruktiv | [R: NVIDIA Learn OpenUSD, Foundry Katana] |
| **Sigstore/TUF** | Sigstore keyless: Fulcio (kurzlebige Zertifikate an OIDC-Identität), Rekor (Transparenzlog); Vertrauenswurzel via TUF verteilt. TUF-Rollen: root (offline), targets (Dateien+Hashes), snapshot, timestamp (online, kurzlebig) | [R: docs.sigstore.dev overview; Rollenbeschreibung aus Drittblogs = R-schwach] |
| **WASM-Sandbox** | WASI ist capability-basiert: Modul erhält nur vom Host gewährte Rechte; Component Model + WIT als Schnittstellenverträge; Isolation ersetzt kein Ressourcenlimit-Design | [R-schwach: Blogs, wasmtime-Doku-Mirror] |
| **Unity Package Manager / Godot AssetLib / Cargo / npm / NuGet / SPDX / Ed25519 / Lockfiles** | Primärquellen nicht abrufbar bzw. nicht gesucht. Bekannt aus Fachwissen: UPM `package.json` mit name/version(SemVer)/dependencies; Cargo/npm Lockfiles mit Prüfsummen; SPDX-Lizenzausdrücke (z. B. `CC0-1.0 OR MIT`, `LicenseRef-…`); Ed25519 als kompakte Signatur | [A] |
| **SemVer** | MAJOR.MINOR.PATCH, Inkompatibel = MAJOR | [A] |

### B2. Designprinzipien für das HASE-Package-System (abgeleitet, [A] soweit nicht anders markiert)

1. **Paket = unveränderlicher, inhaltsadressierter Container.** Jedes Paket hat Manifest + Dateien; Dateihashes (SHA-256) im Manifest, ein Gesamt-Digest identifiziert die Version. Lockfile pro Projekt/Charakter (Cargo/npm-Prinzip) fixiert exakt aufgelöste Versionen + Hashes.
2. **Typisierte Pakete:** `body` (Basismesh + Morph-Targets + Rig), `species` (Delta-/Constraint-Schicht über einem Body: Ohren, Hörner, Proportionsgrenzen), `asset` (Kleidung, Haar, Accessoires), `animation`, `preset`. Jeder Typ hat ein festes Schema; Erweiterungen laufen als benannte Extensions (glTF-Modell) mit `used` vs. `required`: fehlende optionale Extension → Degradierung, fehlende required → Paket abgelehnt.
3. **Kompatibilität über Verträge statt Versionsnummern allein.** Body-Pakete veröffentlichen eine „Body-Schnittstelle" (Skelett-/Vertex-Topologie-ID, Morph-Namensraum, Anker-Punkte). Assets deklarieren `targets: body-interface@^1`. Bruch der Schnittstelle = MAJOR. Das ermöglicht unabhängige Updates von Body und Asset (Factorio-/Cargo-artige Bereichs-Constraints mit `!`-Inkompatibilitäten und `?`-optionalen Abhängigkeiten).
4. **Komposition als geordnete Layer, nicht als Überschreiben.** Charakter = Basis + Stack von Layern (Morph-Gewichte, Species-Delta, Asset-Fits, Overrides). Stärkenordnung explizit (USD-LIVERPS-Idee, vereinfacht), nichtdestruktiv; „Bake" nur beim VRM-Export (passt zu Blender-Modifier-Konzept [R]).
5. **Deterministische Auflösung mit erklärbaren Fehlern.** Resolver nach PubGrub-Prinzip (Konfliktableitung, lesbare Fehlerketten [R]); SemVer-Bereiche; Lockfile erzwingt Reproduzierbarkeit.
6. **Integrität vor Vertrauen, Signaturen optional-zwingend nach Quelle.** Hashes immer; Signatur (Ed25519) des Manifests durch Autor; Index/Registry mit TUF-artigen Rollen (offline root, targets, kurzlebige timestamp) gegen Rollback/Freeze-Angriffe [R-Konzept]. Sigstore-keyless als spätere Option für Identitätsbindung [R].
7. **Code nur in Sandbox, Daten bevorzugt.** Die meisten Pakete sind reine Daten (deklarativ). Skripte/Procedural-Logik als WASM-Komponenten mit capability-basierten Rechten (kein Dateisystem/Netz standardmäßig), Speicher-/Zeit-/Fuel-Limits [R-schwach/A]. Keine nativen Plugins von Community-Quellen.
8. **Lizenz als maschinenlesbares Pflichtfeld.** SPDX-Ausdruck pro Paket und pro Datei-Gruppe (`LicenseRef-` für Eigenes), Herkunftsangabe (Autor, Quelle-URL, Datum) und Weitergabe-Flags (redistribution, modification, commercial). Beim Export wird die effektive VRM-Lizenz als Schnittmenge (restriktivste Bedingung) berechnet; unklare Lizenz → Paket nicht verteilbar markiert (siehe C).

Zusatz: Updates unabhängig; Channels (stable/beta); Migration-Skripte deklarativ (Morph-Umbenennungs-Maps) statt Code.

---

## C) Rechtliches (kein Rechtsrat)

### C1. Referenzprodukte als Inspiration
- Funktionale Ideen (Presets, Slider, Layer) sind generell nicht urheberrechtlich geschützt; Ausdrucksformen (Assets, UI-Grafik, Code, Markenbilder, Namen) schon [A]. Daher: kein Asset-, Code-, Icon- oder UI-Layout-Klon; keine Nutzung der Marken „AION", „Guild Wars", „VRoid", „MetaHuman" in Produktname/Marketing [A]. Patente/Designrechte nicht geprüft [A].
- Beim Studium von AGPL/GPL-Code (MakeHuman/MPFB) keinen Code übernehmen; Copyleft würde auf HASE durchschlagen [R: AGPL/GPL-Status; Wirkung A].

### C2. Lizenzstatus relevanter Quellen
| Quelle | Status | Beleg |
|---|---|---|
| **MakeHuman-Assets** (Basismesh, Targets, Proxies) | Seit Sept. 2020 CC0; Code AGPL; MPFB-Add-on GPL-3.0-or-later; vor 2020 und Drittbeiträge abweichend möglich (Targets damals AGPL) | [R: static.makehumancommunity.org about/license, FAQ „What changed regarding the license in 2020"; Extensions-Seite] |
| **VRoid-Sample-Assets** | Nicht belegt. Von VRoid mitgelieferte Presets/Texturen: Nutzungsbedingungen der VRoid-Seite nicht abrufbar; Dritt-Items tragen eigene Lizenz | [R: Lücke; R-schwach: Booth-Listings] |
| **Mixamo-Animationen** | Adobe-FAQ: lizenzfrei für persönliche, kommerzielle, non-profit Projekte (Filme, Spiele etc.). Rohdateien (Charaktere/Animationen) nicht als eigenständiges Produkt oder Asset-Pack weitergeben; Einbettung in fertiges Produkt ok; ML-Training ausgeschlossen (laut älterem Community-Post). | [R: helpx.adobe.com Mixamo FAQ; community.adobe.com FAQ-Post 2022, möglicherweise veraltet] |

Folgerung: Mixamo-Animationen als **öffentlich downloadbare Bibliothek/Paket zu verteilen** ist nach diesen Quellen nicht zulässig; als Teil einer fertigen Anwendung eingebettet eher ja, aber im Zweifel Adobe-Terms prüfen/anfragen [R + A].

### C3. Repo „VRM-Assets-Pack-For-Silly-Tavern" (test157t)
Geprüft: README und Dateiliste auf github.com/test157t/VRM-Assets-Pack-For-Silly-Tavern (Abruf 2026-10-23) [R].
- **Keine LICENSE-Datei** und keine Lizenzaussage im README sichtbar. Root: `animation/`, `animation_nitral-fork/`, `desktop theme/`, `examples/`, `model/`, `README.md` (Liste evtl. unvollständig).
- **Herkunft Animationen:** README nennt „Animations: https://www.mixamo.com/#/" und verweist auf Konvertierung Mixamo-FBX → BVH mit ButzYung's SystemAnimatorOnline. Keine Aufschlüsselung, welche Dateien aus Mixamo stammen und welche anderweitig (z. B. `animation_nitral-fork`).
- **Modelle:** „6 Example VRM models, 'Do with as you will' -Nitral" – informell, keine formale Lizenz, keine Prüfung der Drittrechte (Modell-Urheber ggf. andere).
- **Theme/Extension:** ST-Discord-inspired Theme von nyxkrage-Projekt, Extension von SillyTavern – eigene Lizenzen [R].
- **Bewertung:** Ohne Lizenz gelten per Default alle Rechte vorbehalten für eigene Anteile des Repo-Inhabers [A]. Die Mixamo-Herkunft führt zu den Mixamo-Beschränkungen (keine Weitergabe der Rohdateien als Pack) [R+A]. Die 118 BVH/VRMA-Dateien dürfen deshalb **nicht ohne weitere Klärung** im HASE-Distribution-Paket oder öffentlichem Repo weitergegeben werden. VRMA-Dateien sind ggf. abgeleitete Konvertierungen derselben Mixamo-Daten [A].
- **Empfehlung:** (1) Nur intern für Entwicklung/Tests nutzen, nicht committen/ausliefern. (2) Maintainer fragen und Lizenzangabe erbeten. (3) Für Auslieferung eigene oder klar lizenzierte (CC0/CC-BY) Animationen verwenden oder Nutzer ihre Mixamo-Dateien selbst importieren lassen (Import-Workflow statt Mitlieferung) [A]. (4) Provenienz-Manifest je Animation (Quelle, Lizenz, Hash) führen – deckt sich mit Prinzip B2.8.

### C4. Weitere Hinweise [A]
- VRM-Exportlizenz: HASE-Charaktere dürfen die VRM-Metadaten nur so setzen, wie die Lizenzen der enthaltenen Pakete erlauben.
- CC-BY-Assets: Attribution automatisch in Credits/Meta ausgeben.
- Community-Pakete: Uploader-Erklärung zu Rechten, Takedown-Prozess, Haftungsausschluss in Nutzungsbedingungen (juristisch prüfen lassen).
- AI-Trainings-/Datenschutz-Fragen bei Cloud-Funktionen: offline-first vermeiden das größtenteils.

---

## Lücken / nächste Schritte
1. Primärtexte lesen: VRoid-Nutzungsbedingungen, VRM-Spec-Repo (Lizenzfelder), Mixamo-Terms aktuell, glTF-Spec-Abschnitt Extensions, MakeHuman LICENSE im Repo.
2. Symmetrie-Werkzeuge in AION 2 offiziell prüfen.
3. UPM/Godot AssetLib/NuGet/Cargo/npm sowie SPDX-Primärquellen nachziehen (hier nur [A]).
4. Juristische Prüfung der Package-Lizenzschnittmengen-Logik und des Animationspakets.

## Quellen (abgerufen/gefunden 2026-10-23)
- https://github.com/test157t/VRM-Assets-Pack-For-Silly-Tavern
- https://static.makehumancommunity.org/about/license.html
- https://static.makehumancommunity.org/oldsite/faq/what_changed_regarding_the_license_in_2020.html
- https://static.makehumancommunity.org/oldsite/documentation/legal.html
- https://extensions.blender.org/add-ons/mpfb/
- https://static.makehumancommunity.org/mpfb/releases/release_20a1.html
- https://helpx.adobe.com/lv/creative-cloud/faq/mixamo-faq.html
- https://community.adobe.com/t5/mixamo-discussions/mixamo-faq-licensing-royalties-ownership-eula-and-tos/m-p/13234775
- https://vroid.com/en/studio/notice/3rgHw1j8oCVkdWM0LKHBH6 (v2.0.0)
- https://vroid.com/en/studio/notice/6ZBxtNd4Gu9scHJpYl2K1L (v2.0.2)
- https://www.metahuman.com/license ; https://www.unrealengine.com/eula/mhc
- https://wiki.guildwars2.com/wiki/Physical_appearance
- https://allthings.how/aion-2-character-customization-guide-every-face-hair-and-body-slider/ ; https://www.exitlag.com/blog/aion-2-character-creation/
- https://dart.googlesource.com/pub.git/+/08d71024/doc/solver.md ; https://docs.rs/pubgrub/
- https://docs.sigstore.dev/signing/overview/
- https://lua-api.factorio.com/latest/auxiliary/mod-structure.html
- https://docs.nvidia.com/learn-openusd/latest/creating-composition-arcs/strength-ordering/what-is-liverps.html
- https://docs.blender.org/manual/en/dev/modeling/modifiers/introduction.html
- https://vrm.dev/vrm1/
- https://www.moguravr.com/?p=89565 (VRoid SDK, schwach)
