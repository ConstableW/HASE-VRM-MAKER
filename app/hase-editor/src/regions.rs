//! Navigationsstruktur des Character Studios: Hauptbereiche, anatomische Regionen (Parameter-Zuordnung, Kamerafokus).
use crate::camera::Camera;
use hase_core::params::ParamDef;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Main { Charakter, Gesicht, Koerper, Haare, Stil, Kleidung, Animation }

impl Main {
    pub const ALL: [Main; 7] = [Main::Charakter, Main::Gesicht, Main::Koerper, Main::Haare, Main::Stil, Main::Kleidung, Main::Animation];
    pub fn label(self) -> &'static str { match self { Main::Charakter => "Charakter", Main::Gesicht => "Gesicht", Main::Koerper => "Körper", Main::Haare => "Haare", Main::Stil => "Stil", Main::Kleidung => "Kleidung", Main::Animation => "Animation" } }
}

const fn cam(yaw: f32, pitch: f32, dist: f32, y: f32) -> Camera { Camera { yaw, pitch, dist, target: [0.0, y, 0.0], fov: 0.62 } }

pub const CAM_BODY: Camera = cam(0.0, 0.04, 3.4, 0.88);
pub const CAM_HEAD: Camera = cam(0.0, 0.02, 0.95, 1.55);

pub struct Region { pub label: &'static str, pub main: Main, pub groups: &'static [&'static str], pub ids: &'static [&'static str], pub cam: Camera }

macro_rules! r { ($l:expr, $m:expr, [$($g:expr),*], [$($i:expr),*], $c:expr) => { Region { label: $l, main: $m, groups: &[$($g),*], ids: &[$($i),*], cam: $c } }; }

/// Reihenfolge = Zuordnungspriorität (erste passende Region beansprucht den Parameter).
pub const REGIONS: &[Region] = &[
    r!("Gesamtkopf", Main::Gesicht, ["head"], ["asym-top"], CAM_HEAD),
    r!("Stirn & Schläfen", Main::Gesicht, ["forehead"], ["asym-temple"], cam(0.0, 0.02, 0.6, 1.62)),
    r!("Augen", Main::Gesicht, ["eyes"], ["asym-eye"], cam(0.0, 0.0, 0.55, 1.60)),
    r!("Brauen", Main::Gesicht, ["eyebrows"], ["asym-brown"], cam(0.0, 0.0, 0.45, 1.62)),
    r!("Nase", Main::Gesicht, ["nose"], ["asym-nose"], cam(0.5, 0.0, 0.45, 1.57)),
    r!("Wangen", Main::Gesicht, ["cheek"], ["asym-cheek"], cam(0.4, 0.0, 0.55, 1.57)),
    r!("Mund & Lippen", Main::Gesicht, ["mouth"], ["asym-mouth"], cam(0.0, 0.0, 0.42, 1.52)),
    r!("Kiefer & Kinn", Main::Gesicht, ["chin"], ["asym-jaw"], cam(0.6, 0.0, 0.55, 1.50)),
    r!("Ohren", Main::Gesicht, ["ears"], ["asym-ear"], cam(1.45, 0.0, 0.5, 1.60)),
    r!("Gesamtkörper", Main::Koerper, [], [], CAM_BODY),
    r!("Brust", Main::Koerper, ["breast"], ["pectoral", "measure-bust", "underbust", "frontchest", "asymm-breast"], cam(0.35, 0.05, 1.3, 1.25)),
    r!("Hals", Main::Koerper, ["neck"], [], cam(0.3, 0.03, 0.9, 1.45)),
    r!("Schultern & Torso", Main::Koerper, ["torso"], ["asymm-trunk"], cam(0.35, 0.05, 1.8, 1.2)),
    r!("Bauch", Main::Koerper, ["stomach"], [], cam(0.35, 0.05, 1.3, 1.05)),
    r!("Becken & Hüfte", Main::Koerper, ["hip", "pelvis"], [], cam(0.35, 0.05, 1.5, 0.95)),
    r!("Gesäß", Main::Koerper, ["buttocks"], [], cam(3.14, 0.05, 1.5, 0.95)),
    r!("Arme", Main::Koerper, ["arms"], [], cam(0.5, 0.05, 2.4, 1.2)),
    r!("Hände", Main::Koerper, ["hands"], [], cam(0.0, 0.1, 1.2, 0.85)),
    r!("Beine", Main::Koerper, ["legs"], [], cam(0.4, 0.04, 2.6, 0.5)),
    r!("Füße", Main::Koerper, ["feet"], [], cam(0.6, 0.2, 1.1, 0.1)),
];

pub fn regions_of(main: Main) -> impl Iterator<Item = (usize, &'static Region)> { REGIONS.iter().enumerate().filter(move |(_, r)| r.main == main) }

/// Region (Index in REGIONS), die den Parameter beansprucht; Makros und Unzugeordnetes → None.
pub fn region_of(d: &ParamDef) -> Option<usize> {
    if d.group == "Makro" { return None; }
    REGIONS.iter().position(|r| r.label == d.group || r.groups.contains(&d.group.as_str()) || r.ids.iter().any(|s| d.id.contains(s)))
}

/// Unterabschnitt innerhalb einer Region: 0 Anatomisches Modell (HASE), 1 Form, 2 Proportionen, 3 Position & Winkel, 4 Asymmetrie.
pub fn tier(d: &ParamDef) -> usize {
    let id = d.id.as_str();
    if id.starts_with("hase:") { 0 }
    else if id.contains("asym") { 4 }
    else if ["trans", "angle", "rot", "push", "point-down"].iter().any(|k| id.contains(k)) { 3 }
    else if ["scale", "measure", "height", "width", "dist", "length"].iter().any(|k| id.contains(k)) { 2 }
    else { 1 }
}
pub const TIER_LABELS: [&str; 5] = ["Anatomisches Modell", "Form", "Proportionen", "Position & Winkel", "Asymmetrie (links/rechts getrennt)"];

/// Kleidungs-Unterkategorie aus dem Asset-Namen (Heuristik, bis Package-Metadaten vorliegen).
pub fn clothes_kind(name: &str) -> usize {
    let n = name.to_lowercase();
    if ["boot", "shoe", "flats", "sandal", "sneaker", "heel"].iter().any(|k| n.contains(k)) { 3 }
    else if ["hat", "fedora", "cap", "glove", "scarf", "glasses", "belt", "tie"].iter().any(|k| n.contains(k)) { 4 }
    else if ["suit", "dress", "outfit", "overall"].iter().any(|k| n.contains(k)) { 0 }
    else if ["pants", "jeans", "shorts", "skirt", "trouser"].iter().any(|k| n.contains(k)) { 2 }
    else { 1 }
}
pub const CLOTHES_KINDS: [&str; 5] = ["Outfits", "Oberteile", "Unterteile", "Schuhe", "Accessoires"];
