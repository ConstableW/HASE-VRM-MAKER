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

## Bekannte Einschränkungen (offen)
Keine Pupillenformen, Irismuster, Augenweiß-/Augen-Make-up; keine Outfit-Studio-Funktionen über das Anlegen hinaus (Layer, Material, Clipping); Finalisieren (Posen/Animation/Export) nicht umgesetzt; Panels nicht einklappbar; keine Icons; Direktmanipulation per Griffe fehlt. GUI wurde für diesen Stand nur headless unter Linux geprüft.
