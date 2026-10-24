//! Projektzustand (Editor-Format v0.1) mit Snapshot-Undo/Redo.
use crate::params::Catalog;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct AssetSel { pub id: String, pub tint: [f32; 3] }

/// Ein Auge: Iris-Farben (innen/außen), Pupillengröße (Faktor), eigene Farben statt Textur.
#[derive(Clone, Debug, PartialEq)]
pub struct EyeSide { pub inner: [f32; 3], pub outer: [f32; 3], pub pupil: f32, pub custom: bool }
impl Default for EyeSide { fn default() -> Self { Self { inner: [0.45, 0.28, 0.12], outer: [0.20, 0.12, 0.05], pupil: 1.0, custom: false } } }

#[derive(Clone, Debug, PartialEq)]
pub struct EyeStyle { pub linked: bool, pub left: EyeSide, pub right: EyeSide, pub reflection: f32, pub limbal: f32 }
impl Default for EyeStyle { fn default() -> Self { Self { linked: true, left: EyeSide::default(), right: EyeSide::default(), reflection: 0.5, limbal: 0.5 } } }

#[derive(Clone, Debug, PartialEq)]
pub struct HairStyle { pub root: [f32; 3], pub tip: [f32; 3], pub highlight: [f32; 3], pub highlight_amt: f32, pub gloss: f32, pub length: f32, pub vol_top: f32, pub vol_front: f32, pub vol_side: f32, pub vol_back: f32 }
impl Default for HairStyle { fn default() -> Self { Self { root: [0.30, 0.19, 0.11], tip: [0.55, 0.36, 0.22], highlight: [0.80, 0.62, 0.38], highlight_amt: 0.0, gloss: 0.3, length: 0.0, vol_top: 0.0, vol_front: 0.0, vol_side: 0.0, vol_back: 0.0 } } }
impl HairStyle { pub fn shape_zero(&self) -> bool { self.length == 0.0 && self.vol_top == 0.0 && self.vol_front == 0.0 && self.vol_side == 0.0 && self.vol_back == 0.0 } }

#[derive(Clone, Debug, PartialEq)]
pub struct Layer { pub color: [f32; 3], pub amount: f32 }

/// Lokales Hautdetail (Muttermal, Tattoo-Motiv, Narbe) in UV-Koordinaten.
#[derive(Clone, Debug, PartialEq)]
pub struct Decal { pub shape: u8, pub u: f32, pub v: f32, pub size: f32, pub rot: f32, pub color: [f32; 3], pub opacity: f32 }
pub const DECAL_SHAPES: [&str; 6] = ["Muttermal", "Stern", "Herz", "Ring", "Kreuz", "Narbe"];

#[derive(Clone, Debug, PartialEq)]
pub struct SkinStyle { pub gloss: f32, pub lips: Layer, pub shadow: Layer, pub blush: Layer, pub freckles: Layer, pub freckle_size: f32, pub freckle_seed: u32, pub decals: Vec<Decal> }
impl Default for SkinStyle { fn default() -> Self { Self {
    gloss: 0.3, lips: Layer { color: [0.72, 0.25, 0.30], amount: 0.0 }, shadow: Layer { color: [0.45, 0.30, 0.45], amount: 0.0 }, blush: Layer { color: [0.85, 0.40, 0.40], amount: 0.0 },
    freckles: Layer { color: [0.45, 0.27, 0.15], amount: 0.0 }, freckle_size: 1.0, freckle_seed: 1, decals: vec![] } } }
impl SkinStyle { pub fn has_details(&self) -> bool { self.lips.amount > 0.0 || self.shadow.amount > 0.0 || self.blush.amount > 0.0 || self.freckles.amount > 0.0 || !self.decals.is_empty() } }

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Style { pub eyes: EyeStyle, pub hair: HairStyle, pub skin: SkinStyle }

fn lj(l: &Layer) -> Value { json!({"color": jv3(&l.color), "amount": l.amount}) }
fn pl(v: &Value, d: &Layer) -> Layer { Layer { color: pv3(&v["color"], d.color), amount: pf(&v["amount"], d.amount) } }
fn jv3(a: &[f32; 3]) -> Value { json!([a[0], a[1], a[2]]) }
fn pv3(v: &Value, d: [f32; 3]) -> [f32; 3] { v.as_array().and_then(|a| Some([a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32, a.get(2)?.as_f64()? as f32])).unwrap_or(d) }
fn pf(v: &Value, d: f32) -> f32 { v.as_f64().map(|x| x as f32).unwrap_or(d) }

impl Style {
    pub fn to_json(&self) -> Value {
        let side = |e: &EyeSide| json!({"inner": jv3(&e.inner), "outer": jv3(&e.outer), "pupil": e.pupil, "custom": e.custom});
        json!({
            "eyes": {"linked": self.eyes.linked, "left": side(&self.eyes.left), "right": side(&self.eyes.right), "reflection": self.eyes.reflection, "limbal": self.eyes.limbal},
            "hair": {"root": jv3(&self.hair.root), "tip": jv3(&self.hair.tip), "highlight": jv3(&self.hair.highlight), "highlightAmount": self.hair.highlight_amt, "gloss": self.hair.gloss, "length": self.hair.length, "volTop": self.hair.vol_top, "volFront": self.hair.vol_front, "volSide": self.hair.vol_side, "volBack": self.hair.vol_back},
            "skin": {"gloss": self.skin.gloss, "lips": lj(&self.skin.lips), "shadow": lj(&self.skin.shadow), "blush": lj(&self.skin.blush), "freckles": lj(&self.skin.freckles), "freckleSize": self.skin.freckle_size, "freckleSeed": self.skin.freckle_seed, "decals": self.skin.decals.iter().map(|d| json!({"shape": d.shape, "u": d.u, "v": d.v, "size": d.size, "rot": d.rot, "color": jv3(&d.color), "opacity": d.opacity})).collect::<Vec<_>>()}
        })
    }
    pub fn from_json(v: &Value) -> Style {
        let d = Style::default();
        let side = |v: &Value, d: &EyeSide| EyeSide { inner: pv3(&v["inner"], d.inner), outer: pv3(&v["outer"], d.outer), pupil: pf(&v["pupil"], d.pupil), custom: v["custom"].as_bool().unwrap_or(d.custom) };
        Style {
            eyes: EyeStyle { linked: v["eyes"]["linked"].as_bool().unwrap_or(true), left: side(&v["eyes"]["left"], &d.eyes.left), right: side(&v["eyes"]["right"], &d.eyes.right), reflection: pf(&v["eyes"]["reflection"], d.eyes.reflection), limbal: pf(&v["eyes"]["limbal"], d.eyes.limbal) },
            hair: HairStyle { root: pv3(&v["hair"]["root"], d.hair.root), tip: pv3(&v["hair"]["tip"], d.hair.tip), highlight: pv3(&v["hair"]["highlight"], d.hair.highlight), highlight_amt: pf(&v["hair"]["highlightAmount"], d.hair.highlight_amt), gloss: pf(&v["hair"]["gloss"], d.hair.gloss), length: pf(&v["hair"]["length"], 0.0), vol_top: pf(&v["hair"]["volTop"], 0.0), vol_front: pf(&v["hair"]["volFront"], 0.0), vol_side: pf(&v["hair"]["volSide"], 0.0), vol_back: pf(&v["hair"]["volBack"], 0.0) },
            skin: SkinStyle { gloss: pf(&v["skin"]["gloss"], d.skin.gloss), lips: pl(&v["skin"]["lips"], &d.skin.lips), shadow: pl(&v["skin"]["shadow"], &d.skin.shadow), blush: pl(&v["skin"]["blush"], &d.skin.blush), freckles: pl(&v["skin"]["freckles"], &d.skin.freckles), freckle_size: pf(&v["skin"]["freckleSize"], 1.0), freckle_seed: v["skin"]["freckleSeed"].as_u64().unwrap_or(1) as u32,
                decals: v["skin"]["decals"].as_array().map(|a| a.iter().map(|x| Decal { shape: x["shape"].as_u64().unwrap_or(0) as u8, u: pf(&x["u"], 0.5), v: pf(&x["v"], 0.5), size: pf(&x["size"], 0.02), rot: pf(&x["rot"], 0.0), color: pv3(&x["color"], [0.2, 0.12, 0.1]), opacity: pf(&x["opacity"], 0.8) }).collect()).unwrap_or_default() },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PkgRef { pub id: String, pub version: String, pub digest: String }

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterState {
    pub body_ref: PkgRef,
    pub species_ref: PkgRef,
    pub style: Style,
    pub species: String,
    pub values: Vec<f32>, // aligned zu Catalog.defs
    pub assets: Vec<AssetSel>,
    pub skin: String,
    pub eyes: String,
}

pub struct History { undo: Vec<CharacterState>, redo: Vec<CharacterState>, last_key: Option<(usize, Instant)> }

impl History {
    pub fn new() -> Self { Self { undo: vec![], redo: vec![], last_key: None } }
    /// Merkt den Zustand *vor* einer Änderung. `key` (z. B. Parameterindex) fasst schnelle Folgeänderungen (Slider-Drag) zusammen.
    pub fn record(&mut self, before: &CharacterState, key: Option<usize>) {
        let now = Instant::now();
        if let (Some(k), Some((lk, t))) = (key, self.last_key) { if k == lk && now.duration_since(t) < Duration::from_millis(600) { self.last_key = Some((k, now)); return; } }
        self.undo.push(before.clone());
        if self.undo.len() > 300 { self.undo.remove(0); }
        self.redo.clear();
        self.last_key = key.map(|k| (k, now));
    }
    pub fn undo(&mut self, cur: &CharacterState) -> Option<CharacterState> { let s = self.undo.pop()?; self.redo.push(cur.clone()); self.last_key = None; Some(s) }
    pub fn redo(&mut self, cur: &CharacterState) -> Option<CharacterState> { let s = self.redo.pop()?; self.undo.push(cur.clone()); self.last_key = None; Some(s) }
    pub fn can_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo.is_empty() }
}

impl CharacterState {
    pub fn to_json(&self, cat: &Catalog, body_id: &str) -> Value {
        let params: BTreeMap<&str, f32> = cat.defs.iter().zip(&self.values).filter(|(d, v)| (**v - d.default).abs() > 1e-6).map(|(d, v)| (d.id.as_str(), *v)).collect();
        json!({
            "formatVersion": "0.1-editor", "note": "Editor-Zwischenformat, wird auf docs/03-specifications/project-format.md abgebildet",
            "body": { "id": body_id, "range": "^1.0", "version": self.body_ref.version, "digest": self.body_ref.digest }, "species": { "id": self.species, "range": "^1.0", "version": self.species_ref.version, "digest": self.species_ref.digest },
            "parameters": params,
            "assets": self.assets.iter().map(|a| json!({ "dev-asset": a.id, "tint": a.tint })).collect::<Vec<_>>(),
            "skin": self.skin, "eyes": self.eyes, "style": self.style.to_json()
        })
    }
    /// Parameter-IDs einer Projektdatei, die der Katalog nicht kennt (gehen beim Laden verloren → Migrationshinweis).
    pub fn unknown_params(v: &Value, cat: &Catalog) -> Vec<String> {
        v["parameters"].as_object().map(|o| o.keys().filter(|k| cat.canonical(k).is_none()).cloned().collect()).unwrap_or_default()
    }
    pub fn from_json(v: &Value, cat: &Catalog) -> Option<CharacterState> {
        let mut values = cat.defaults();
        for (k, x) in v["parameters"].as_object()? { if let (Some(i), Some(f)) = (cat.canonical(k), x.as_f64()) { values[i] = f as f32; } }
        let assets = v["assets"].as_array()?.iter().filter_map(|a| { let t = a["tint"].as_array()?; Some(AssetSel { id: a["dev-asset"].as_str()?.into(), tint: [t.first()?.as_f64()? as f32, t.get(1)?.as_f64()? as f32, t.get(2)?.as_f64()? as f32] }) }).collect();
        let r = |k: &str| PkgRef { id: v[k]["id"].as_str().unwrap_or("").into(), version: v[k]["version"].as_str().unwrap_or("").into(), digest: v[k]["digest"].as_str().unwrap_or("").into() };
        Some(CharacterState { style: Style::from_json(&v["style"]), body_ref: r("body"), species_ref: r("species"), species: v["species"]["id"].as_str()?.into(), values, assets, skin: v["skin"].as_str().unwrap_or("").into(), eyes: v["eyes"].as_str().unwrap_or("").into() })
    }
}
