//! Design-Tokens und egui-Stil (dunkles Studio-Design). Einzige Stelle für Farben/Abstände.
use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub const BG: Color32 = Color32::from_rgb(0x08, 0x0D, 0x15);
pub const PANEL: Color32 = Color32::from_rgb(0x10, 0x18, 0x24);
pub const PANEL2: Color32 = Color32::from_rgb(0x17, 0x22, 0x31);
pub const HOVER: Color32 = Color32::from_rgb(0x23, 0x34, 0x49);
pub const ACTIVE: Color32 = Color32::from_rgb(0x17, 0x47, 0x7E);
pub const ACCENT: Color32 = Color32::from_rgb(0x58, 0xA6, 0xFF);
pub const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEF, 0xF7);
pub const TEXT2: Color32 = Color32::from_rgb(0x9B, 0xAA, 0xBB);
pub const LINE: Color32 = Color32::from_rgb(0x2B, 0x39, 0x4A);
pub const WARN: Color32 = Color32::from_rgb(0xE5, 0xAA, 0x52);
pub const ERR: Color32 = Color32::from_rgb(0xE3, 0x6C, 0x72);

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL; v.window_fill = PANEL; v.extreme_bg_color = BG; v.faint_bg_color = PANEL2;
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACTIVE; v.selection.stroke = Stroke::new(1.0, ACCENT);
    let r = CornerRadius::same(6);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = PANEL; w.noninteractive.bg_stroke = Stroke::new(1.0, LINE); w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT2); w.noninteractive.corner_radius = r;
    w.inactive.bg_fill = PANEL2; w.inactive.weak_bg_fill = PANEL2; w.inactive.bg_stroke = Stroke::new(1.0, LINE); w.inactive.fg_stroke = Stroke::new(1.0, TEXT); w.inactive.corner_radius = r;
    w.hovered.bg_fill = HOVER; w.hovered.weak_bg_fill = HOVER; w.hovered.bg_stroke = Stroke::new(1.0, ACCENT); w.hovered.fg_stroke = Stroke::new(1.0, TEXT); w.hovered.corner_radius = r;
    w.active.bg_fill = ACTIVE; w.active.weak_bg_fill = ACTIVE; w.active.bg_stroke = Stroke::new(1.0, ACCENT); w.active.fg_stroke = Stroke::new(1.0, TEXT); w.active.corner_radius = r;
    w.open.bg_fill = PANEL2; w.open.weak_bg_fill = PANEL2; w.open.corner_radius = r;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.interact_size.y = 24.0;
        s.spacing.slider_width = 150.0;
        for (_, f) in s.text_styles.iter_mut() { f.size *= 1.05; }
    });
}

/// Deutsche Anzeigenamen für Parameter (Wort-Ersetzung auf den generierten MakeHuman-Namen).
pub fn label_de(l: &str) -> String {
    const MAP: &[(&str, &str)] = &[
        ("decr incr", ""), ("down up", "Höhe"), ("backward forward", "Tiefe"), ("in out", "innen/außen"), ("concave convex", "konkav/konvex"),
        ("square round", "eckig/rund"), ("pointed triangle", "spitz/dreieckig"), ("compress uncompress", "Kompression"), ("l r", "links/rechts"),
        ("asym", "Asymmetrie"), ("eyebrows", "Augenbrauen"), ("brown", "Augenbraue"), ("eyefold", "Lidfalte"), ("eye", "Auge"), ("bag", "Tränensack"),
        ("corner", "Augenwinkel"), ("height", "Höhe"), ("push", "Wölbung"), ("trans", "Position"), ("scale", "Größe"), ("horiz", "Breite"), ("vert", "Höhe"),
        ("depth", "Tiefe"), ("nose", "Nase"), ("nostrils", "Nasenlöcher"), ("width", "Breite"), ("flaring", "Nasenflügel"), ("hump", "Höcker"),
        ("point", "Spitze"), ("curve", "Rücken"), ("base", "Basis"), ("septumangle", "Septumwinkel"), ("volume", "Volumen"), ("greek", "griechisch"),
        ("mouth", "Mund"), ("upperlip", "Oberlippe"), ("lowerlip", "Unterlippe"), ("cupidsbow", "Amorbogen"), ("philtrum", "Philtrum"), ("angles", "Mundwinkel"),
        ("laugh lines", "Lachfalten"), ("dimples", "Grübchen"), ("ext", "Überstand"), ("middle", "Mitte"), ("chin", "Kinn"), ("jaw", "Kiefer"), ("drop", "Absenkung"),
        ("prominent", "Vorsprung"), ("bones", "Knochen"), ("cleft", "Kinngrübchen"), ("triangle", "dreieckig"), ("prognathism", "Prognathie"),
        ("cheek", "Wange"), ("inner", "innen"), ("ear", "Ohr"), ("lobe", "Ohrläppchen"), ("shape", "Form"), ("flap", "Ohrmuschel"), ("wing", "Abstehen"), ("rot", "Drehung"),
        ("forehead", "Stirn"), ("temple", "Schläfe"), ("nubian", "nubisch"), ("head", "Kopf"), ("age", "Alter"), ("angle", "Winkel"), ("fat", "Fülle"),
        ("oval", "oval"), ("round", "rund"), ("rectangular", "rechteckig"), ("square", "quadratisch"), ("diamond", "Raute"), ("triangular", "dreieckig"), ("invertedtriangular", "umgekehrt dreieckig"),
        ("back", "Hinterkopf"), ("neck", "Hals"), ("double", "Doppelkinn"), ("measure", "Maß"), ("circ", "Umfang"), ("dist", "Abstand"), ("length", "Länge"),
        ("torso", "Torso"), ("muscle", "Muskel"), ("pectoral", "Brustmuskel"), ("dorsi", "Rückenmuskel"), ("vshape", "V-Form"), ("underbust", "Unterbrust"),
        ("frontchest", "Brustbreite"), ("bust", "Brust"), ("waist", "Taille"), ("hips", "Hüfte"), ("hip", "Hüfte"), ("nape", "Nacken"), ("napetowaist", "Nacken–Taille"), ("waisttohip", "Taille–Hüfte"),
        ("shoulder", "Schulter"), ("breast", "Brust"), ("nipple", "Brustwarze"), ("stomach", "Bauch"), ("navel", "Nabel"), ("pregnant", "Schwanger"), ("tone", "Spannung"),
        ("pelvis", "Becken"), ("bulge", "Wölbung"), ("buttocks", "Gesäß"), ("upperarm", "Oberarm"), ("lowerarm", "Unterarm"), ("upperleg", "Oberschenkel"), ("lowerleg", "Unterschenkel"),
        ("upperlegs", "Oberschenkel"), ("lowerlegs", "Unterschenkel"), ("calf", "Wade"), ("knee", "Knie"), ("thigh", "Oberschenkel"), ("ankle", "Knöchel"), ("wrist", "Handgelenk"),
        ("leg", "Bein"), ("valgus", "X-Bein"), ("foot", "Fuß"), ("hand", "Hand"), ("fingers", "Finger"), ("distance", "Abstand"), ("diameter", "Dicke"), ("macro", "Makro"),
        ("l", "links"), ("r", "rechts"),
    ];
    let mut out = Vec::new();
    let mut words: Vec<String> = l.split_whitespace().map(|s| s.to_string()).collect();
    // zweiwortige Paare zuerst
    let mut i = 0;
    while i < words.len() {
        if i + 1 < words.len() {
            let pair = format!("{} {}", words[i], words[i + 1]);
            if let Some((_, to)) = MAP.iter().find(|(k, _)| *k == pair) { if !to.is_empty() { out.push(to.to_string()); } i += 2; continue; }
        }
        let w = words[i].to_lowercase();
        if w.chars().all(|c| c.is_ascii_digit()) { out.push(w); }
        else if let Some((_, to)) = MAP.iter().find(|(k, _)| *k == w) { if !to.is_empty() { out.push(to.to_string()); } }
        else { out.push(words[i].clone()); }
        i += 1;
    }
    words.clear();
    let s = out.join(" ");
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => s }
}
