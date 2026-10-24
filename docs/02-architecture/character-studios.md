# Charakterstudios: Render-Materialien, Thumbnails, Anatomie, Haut, Haar, Bibliothek

Maßgebliche Quelle für die Umsetzung der Studios im Editor (Stand 2026-10-24). Bezug: OD-026, `../01-product/ux-concept.md`, OD-025 (Performance). Bilder: `../07-quality/screens/` (headless unter Linux/Lavapipe gerendert; Windows-Test durch den Owner bestanden für den Stand davor).

## Render-Materialsystem (`render.rs`, `shader.wgsl`)
Material je Mesh mit Art (Standard, Auge, Haar, Haut) und 6 Parametervektoren. Ein Shader, keine zusätzlichen Pässe.
- **Auge:** Iris-/Pupillensteuerung im Texturraum, je Auge getrennt (links = Welt-x > 0): Iris-Farbe innen/außen (Verlauf, Texturstruktur bleibt), Pupillengröße (Dilatation durch Radialumbildung der Textur), Limbusring, Hornhaut-Reflex. Belegt: `screens/augen-iris-pupille.png`.
- **Haar:** Verlauf Kopfhaut→Spitze über Welt-Höhe, Strähnen-Highlights (Hash je Strähne), Textur als Helligkeitsdetail, Glanz.
- **Haut:** Glanz; Hautdetails über austauschbare Textur (`MeshUpdate::Texture`).
- **Szene:** Beleuchtungs-/Hintergrund-Presets Studio/Outdoor/Nacht, Hintergrundverlauf, Raster schaltbar.

## Thumbnail-Dienst (`thumbs.rs`, `offscreen.rs`)
Ein Worker-Thread mit **eigenem wgpu-Gerät und eigener Session** rendert Vorschaubilder (Presets, Frisuren, Augenformen, Bibliothekseinträge); UI-Thread wird nie blockiert. Cache auf Platte (`.hase-cache/thumbs`, Schlüssel = SHA-256 aus Zustand + Körperbasis-Fingerprint + Ansicht + Renderversion) und als egui-Textur. Kamera relativ zur Körpergröße des dargestellten Zustands. Aufträge nur für sichtbare Kacheln (virtualisierte Raster), höchstens 24 gleichzeitig.

## Anatomisches Modell (`hase-core/src/anatomy.rs`)
Prozedural erzeugte Zusatz-Targets, verankert an gemessenen Landmarken des Basismeshes (Brustwarzen bzw. Schwerpunkt des Gesäß-Targets), mit glattem Falloff (C1) und Verformung entlang Hautnormale/Vorwärtsrichtung. **Brust:** Volumen, Projektion, obere/untere Fülle, Breite, Ansatzhöhe, Hängen – links/rechts getrennt gekoppelt, signiert (negative Richtung gedämpft). **Gesäß:** Volumen, Projektion, Straffung, Breite. Sie sind zusätzliche Parameter (`hase:*`), keine Skalierung; Bilder: `screens/brust-anatomie.png`, `screens/gesaess-anatomie.png`. Grenzen: Kopplung an Gewicht/Alter und Kleidungsanpassung sind nicht modelliert; bei extremen Körper-Morphs kann die Verankerung (Basismesh-Landmarke) abweichen; Muskeldefinition und Hüfte nutzen weiter die MakeHuman-Parameter. Gemessen: Generierung beim Laden im Millisekundenbereich.

## Haut und Details (`skin.rs`)
Masken (Lippen, Lidfalten, Wangen, Nase) entstehen **aus den Morph-Fußabdrücken** (Verformungsstärke der passenden Targets, in UV-Raum rasterisiert, Kurve nach der Interpolation), folgen also der Gesichtstopologie. Komposition auf CPU in 1024²: Lippenfarbe, Lidschatten, Wangenrouge, Sommersprossen (zwei versetzte Raster, Häufungen), lokale Details (Muttermal, Stern, Herz, Ring, Kreuz, Narbe) per Klick am Modell (Strahl → UV). Gemessen: Masken 1,9 ms, Komposition 12 ms (Container-CPU), nur bei Änderung. Bild: `screens/haut-makeup.png`. Grenzen: Tattoos sind einfache Vektor-Motive (keine importierbaren Bilder), Masken nur für die MakeHuman-Topologie, Lippenmaske etwas größer als die Lippen.

## Haar (`session.rs: shape_hair`)
Länge (relativ zur Kopfhaut) und Volumen je Region (Oberkopf, Vorderseite, Seiten, Hinterkopf) entlang der Haarnormalen auf der angepassten Frisur; Farbe: Ansatz/Spitze/Highlight/Glanz. Bild: `screens/haar-form-farbe.png`. Nicht umgesetzt: Scheitel, Strähnenbearbeitung, Locken/Wellen, Haarphysik, Kollision mit Schultern.

## Charakterbibliothek (`library.rs`, `library_ui.rs`)
Ablage `user-content/characters/<id>/` (character.hase.json, meta.json, thumb.png, snapshots/). Neu/Speichern/Öffnen/Duplizieren (Variante mit Verweis)/Umbenennen/Löschen, Vorlagen, Favoriten, Snapshots, Vergleich aktuell ↔ Eintrag mit Parameterunterschieden. Öffnen eines Charakters einer anderen Körperbasis fragt nach Wechsel (Versionsbindung, siehe `body-species-studio.md`). Untere Leiste: Presets der Spezies/Merkmale, eigene Charaktere, Favoriten mit Vorschaubildern.

## Viewport
Werkzeuge Rotieren/Bewegen/Zoomen/Fokus/Bereich: „Bereich“ wählt per Klick die Region mit dem größten **Anteil an ihrer Gesamtverformung** an der Stelle (Spezifität) und öffnet sie mit Kamerafokus. Kompakte FPS-Anzeige, F3 für Details.

## Finalisieren: VRM-1.0-Export (`hase-core/src/export.rs`, `session.rs: build_export`, `export_ui.rs`)
Natives Schreiben von GLB mit VRMC_vrm: Skelett (53 Bones aus dem MPFB2-Rig, Gelenkpositionen aus dem aktuell gemorphten Mesh), Skin-Gewichte (bis 4 Einflüsse; Proxys per Barycentric-Bindung, negative Anteile begrenzt, exakt normiert), Körper + Proxys als Meshes mit MToon-Materialien, Humanoid-Mapping (alle 15 Pflicht-Bones + Finger/Zehen), Expressions (blink, blinkLeft/Right, aa/ih/ou/ee/oh, happy/sad/relaxed/surprised) als Morph-Targets auf Körper, Brauen, Wimpern. Profile: Basischarakter ohne Outfit / mit Outfit (verdeckte Körperflächen entfallen). Augentextur wird mit den individuellen Irisfarben/Pupillen **eingebacken**, Hautdetails sind in die Textur eingerechnet. Zusammenstellung im UI-Thread (≈ 75–170 ms), Schreiben/PNG-Kodierung und Rücklesen im Hintergrund-Thread (≈ 140–260 ms). Nachweis (2026-10-24): VRM-Schema 0 Fehler (`tools/validate/validate_vrm.py`), glTF-Validator 0 Fehler (nur Warnung MULTIPLE_EXTENSIONS, wie beim Blender-Add-on), Import in Blender (VRM-Add-on) und Tanz-VRMA verformt den exportierten Charakter korrekt (`screens/vrm-export-blender-import.png`). Einschränkungen: Haar-Farbverlauf nur als Mittelfarbe, keine Spring Bones/Haarphysik, Augen-Reflexion nicht exportiert, Mundöffnung begrenzt (Basismesh ohne Mundraum), nicht für importierte Körperbasen (Rig/Gewichte fehlen), Dateigröße 15–26 MB (PNG-Texturen 2048²), kein eigenes Zielprofil-System (nur VRM 1.0 generisch).

## Posen/Animation (`hase-core/src/pose.rs`)
VRMA-Clips (Quaternius UAL1/UAL2, CC0) werden geladen und auf das Rig retargetet: Welt-Rotation relativ zur Quell-Ruhepose je humanoidem Bone (die Dateien tragen die Ruhe-Rotationen der Quellknoten, Hüfte ≈ 105° um X), Hüftbewegung skaliert mit der Hüfthöhe des Charakters. CPU-Skinning von Körper und allen Proxys (Haare, Kleidung, Augen, Brauen) in der Vorschau: **1,4 ms pro Frame** bei 42.700 Vertices (Container-CPU, Dance_Loop, gemessen). Bedienung: Clipliste mit Suche, Abspielen/Pause, Schleife, Tempo, Zeitregler, Ruhepose. Bilder: `screens/pose-vorschau.png`, `screens/finalisieren-animation.png`. Grenzen: Quell-A-Pose und MakeHuman-A-Pose weichen leicht ab (Arme gehen durch kurze Ärmel), keine Kollisionskorrektur der Kleidung, nur Referenz-Körperbasis, keine Blend-/Mischung mehrerer Clips, Expressions-Vorschau fehlt.

## Bekannte Einschränkungen (offen)
Keine Pupillenformen, Irismuster, Augenweiß-/Augen-Make-up; keine Outfit-Studio-Funktionen über das Anlegen hinaus (Layer, Material, Clipping); Panels nicht einklappbar; keine Icons; Direktmanipulation per Griffe fehlt. GUI wurde für diesen Stand nur headless unter Linux geprüft.

## Korrekturen nach Owner-Test (2026-10-24)

- **Pose-Retargeting:** VRMA-Quellen haben T-Pose-Ruhelage, unsere Körper A-Pose. Die Welt-Delta-Rotation wird relativ zur Quell-Ruhelage berechnet und für die Armkette um die A→T-Kompensation `C` ergänzt (`pose.rs`). `PoseRig::generic` deckt beliebige Körperbasen ab. RESEARCHED (Sichtprüfung Walk/Dance); Rest-Limit: kurze Ärmel können bei Armhebung durchstoßen.
- **Kleidung:** `fit::push_out` schiebt Kleidungsvertices per `BodyGrid` aus dem Körper (3 mm Abstand, zwei Durchgänge mit Glättung); ca. 14 ms pro Fit. Kostenoptimierung offen.
- **Lippen:** eigene Lippenmaske aus Morph-Footprint (UV 90° gedreht berücksichtigt), kein „Pinselklecks“ mehr.
- **Header:** „Speichern“ legt in die Bibliothek (`quick_save`), „Exportieren“ springt zu Finalisieren › Export.
- **VRM öffnen (statisch):** `vrm_model.rs` lädt fremde VRMs (z. B. Lylia_v2.vrm) als Ansichts-/Pose-/Ausdrucksmodell. Körper, Haare, Kleidung sind gebacken, nicht editierbar; Spring Bones werden nicht simuliert; kein Export/Bibliothek für importierte Modelle. PROPOSED nächster Schritt: Editierbarkeit nur über Morph-Sets/Retopologie.

## Anprobe auf importierten Körpern (2026-10-24, `hase-core/src/dress.rs`)

Ziel: nacktes importiertes VRM (z. B. VRoid-Basis) mit MakeHuman-Kleidung anziehen, posen und als VRM exportieren. PROPOSED, am Test-VRM verifiziert (Blender-Import, Validator 0 Fehler), nicht unter Windows getestet.

1. **Registrierung:** MakeHuman-Basis → Haut des Ziels. (a) Skelett-Vorausrichtung: je humanoidem Bone Drehung+Skalierung entlang der Knochenachse, per MH-Gewichten geblendet (gleicht Proportionen und A-/T-Pose aus). (b) Nicht-rigide ICP-Verfeinerung (10 Iterationen, Normalenprüfung, geglättetes Verschiebungsfeld); Kopf, Hände, Augen bleiben bei (a).
2. **Kleidung:** `.mhclo`-Proxys folgen der abgebildeten Basis; `fit::push_out` (ohne Zweitdurchgang, 4 mm) gegen die Zielhaut; Skin-Gewichte = Gewichte des nächsten Zielhaut-Vertex (rig-unabhängig).
3. **Hautteile:** Standard = Teile mit ≥ 50 % Körperhöhe; in der Teileliste (Modell-Panel) änderbar, Teile lassen sich ausblenden.
4. **Export:** Originalteile, Skelett (Rest-Rotationen in die Welt eingebacken → normalisiertes VRM-1.0-Rig), Humanoid, Ausdrücke bleiben; Kleidung kommt dazu. Verlust: Spring Bones und MToon-Einstellungen des Originals.
5. **Grenzen:** Kopf-/Hand-/Fußkleidung (Hüte, Handschuhe) wird nicht angepasst; im Schritt ohne Skelett (rohe GLB) ist die Anprobe noch nicht möglich; dünne Spalte (Schritt) lassen die aufgemalte Unterwäsche durchscheinen.
