//! Package-Laden (Manifeste nach docs/03-specifications): Body und Spezies aus `examples/species/packages`.
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Trait { pub id: String, pub label: String, pub set: BTreeMap<String, f32> }
#[derive(Clone, Debug)]
pub struct Preset { pub id: String, pub label: String, pub params: BTreeMap<String, f32> }
#[derive(Clone, Debug)]
pub struct Component { pub asset: String, pub slot: String, pub stub: bool }

#[derive(Clone, Debug)]
pub struct Species {
    pub id: String,
    pub label: String,
    pub version: String,
    pub defaults: BTreeMap<String, f32>,
    pub limits: BTreeMap<String, [f32; 2]>,
    pub traits: Vec<Trait>,
    pub presets: Vec<Preset>,
    pub components: Vec<Component>,
    pub stub: bool,
}

#[derive(Clone, Debug)]
pub struct BodyPackage { pub id: String, pub version: String, pub label: String, pub data_root: PathBuf, pub interface: String }

pub struct Packages { pub body: BodyPackage, pub species: Vec<Species>, pub issues: Vec<String> }

fn label(v: &Value, fallback: &str) -> String {
    match v { Value::String(s) => s.clone(), Value::Object(o) => o.get("de").or_else(|| o.values().next()).and_then(|x| x.as_str()).unwrap_or(fallback).to_string(), _ => fallback.to_string() }
}
fn fmap(v: &Value) -> BTreeMap<String, f32> { v.as_object().map(|o| o.iter().filter_map(|(k, x)| x.as_f64().map(|f| (k.clone(), f as f32))).collect()).unwrap_or_default() }

impl Packages {
    pub fn load(repo: &Path) -> Result<Packages, String> {
        let dir = repo.join("examples/species/packages");
        let mut body = None; let mut species = Vec::new(); let mut issues = Vec::new();
        let mut manifests: Vec<(String, Value)> = Vec::new();
        for e in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten() {
            let mf = e.path().join("hase-package.json");
            if let Ok(s) = std::fs::read_to_string(&mf) { match serde_json::from_str::<Value>(&s) { Ok(v) => manifests.push((e.file_name().to_string_lossy().into(), v)), Err(err) => issues.push(format!("{}: {err}", mf.display())) } }
        }
        manifests.sort_by(|a, b| a.0.cmp(&b.0));
        let stub_assets: Vec<String> = manifests.iter().filter(|(_, v)| v["dev"]["status"] == "stub").map(|(_, v)| v["id"].as_str().unwrap_or("").to_string()).collect();
        for (_, v) in &manifests {
            match v["type"].as_str() {
                Some("body") => {
                    let root = v["dev"]["fileRoot"].as_str().map(|r| repo.join(r)).unwrap_or_else(|| dir.join(v["id"].as_str().unwrap_or("")).join("data"));
                    body = Some(BodyPackage { id: v["id"].as_str().unwrap_or("").into(), version: v["version"].as_str().unwrap_or("").into(), label: label(&v["name"], "Body"), data_root: root, interface: v["body"]["provides"]["interface"].as_str().unwrap_or("").into() });
                }
                Some("species") => {
                    let s = &v["species"];
                    species.push(Species {
                        id: v["id"].as_str().unwrap_or("").into(), label: label(&v["name"], "Spezies"), version: v["version"].as_str().unwrap_or("").into(),
                        defaults: fmap(&s["defaults"]),
                        limits: s["limits"].as_object().map(|o| o.iter().filter_map(|(k, x)| { let a = x.as_array()?; Some((k.clone(), [a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32])) }).collect()).unwrap_or_default(),
                        traits: s["traits"].as_object().map(|o| o.iter().map(|(k, t)| Trait { id: k.clone(), label: label(&t["label"], k), set: fmap(&t["set"]) }).collect()).unwrap_or_default(),
                        presets: s["presets"].as_array().map(|a| a.iter().map(|p| Preset { id: p["id"].as_str().unwrap_or("").into(), label: label(&p["label"], "Preset"), params: fmap(&p["parameters"]) }).collect()).unwrap_or_default(),
                        components: s["components"].as_array().map(|a| a.iter().map(|c| { let id = c["asset"].as_str().unwrap_or("").to_string(); Component { stub: stub_assets.contains(&id), slot: c["slot"].as_str().unwrap_or("").into(), asset: id } }).collect()).unwrap_or_default(),
                        stub: v["dev"]["status"] == "stub",
                    });
                }
                _ => {}
            }
        }
        let body = body.ok_or("kein Body-Package gefunden")?;
        if species.is_empty() { issues.push("keine Spezies gefunden".into()); }
        Ok(Packages { body, species, issues })
    }
}
