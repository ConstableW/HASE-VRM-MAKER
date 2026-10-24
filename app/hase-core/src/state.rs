//! Projektzustand (Editor-Format v0.1) mit Snapshot-Undo/Redo.
use crate::params::Catalog;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct AssetSel { pub id: String, pub tint: [f32; 3] }

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PkgRef { pub id: String, pub version: String, pub digest: String }

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterState {
    pub body_ref: PkgRef,
    pub species_ref: PkgRef,
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
            "skin": self.skin, "eyes": self.eyes
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
        Some(CharacterState { body_ref: r("body"), species_ref: r("species"), species: v["species"]["id"].as_str()?.into(), values, assets, skin: v["skin"].as_str().unwrap_or("").into(), eyes: v["eyes"].as_str().unwrap_or("").into() })
    }
}
