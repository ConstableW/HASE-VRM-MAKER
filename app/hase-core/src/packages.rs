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
    pub user: bool,
    pub description: String,
    pub body_id: String,
    pub dir: Option<PathBuf>,
    pub digest: String,
}

#[derive(Clone, Debug)]
pub struct BodyPackage { pub id: String, pub version: String, pub label: String, pub data_root: PathBuf, pub interface: String, pub capabilities: Vec<String>, pub status: String, pub digest: String, pub user: bool, pub dir: Option<PathBuf> }

pub struct Packages { pub body: BodyPackage, pub bodies: Vec<BodyPackage>, pub species: Vec<Species>, pub issues: Vec<String> }

fn label(v: &Value, fallback: &str) -> String {
    match v { Value::String(s) => s.clone(), Value::Object(o) => o.get("de").or_else(|| o.values().next()).and_then(|x| x.as_str()).unwrap_or(fallback).to_string(), _ => fallback.to_string() }
}
fn fmap(v: &Value) -> BTreeMap<String, f32> { v.as_object().map(|o| o.iter().filter_map(|(k, x)| x.as_f64().map(|f| (k.clone(), f as f32))).collect()).unwrap_or_default() }

impl Packages {
    pub fn load(repo: &Path) -> Result<Packages, String> {
        let dir = repo.join("examples/species/packages");
        let udir = user_dir(repo);
        let mut bodies: Vec<BodyPackage> = Vec::new(); let mut species = Vec::new(); let mut issues = Vec::new();
        let mut manifests: Vec<(String, Value)> = Vec::new();
        let mut entries: Vec<(std::fs::DirEntry, bool)> = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten().map(|e| (e, false)).collect();
        if let Ok(rd) = std::fs::read_dir(&udir) { entries.extend(rd.flatten().map(|e| (e, true))); }
        let mut user_dirs: BTreeMap<String, PathBuf> = BTreeMap::new();
        let mut digests: BTreeMap<String, String> = BTreeMap::new();
        for (e, user) in entries {
            let mf = e.path().join("hase-package.json");
            if let Ok(s) = std::fs::read_to_string(&mf) { match serde_json::from_str::<Value>(&s) { Ok(v) => { digests.insert(v["id"].as_str().unwrap_or("").into(), sha256_hex(s.as_bytes())); if user { user_dirs.insert(v["id"].as_str().unwrap_or("").into(), e.path()); } manifests.push((e.file_name().to_string_lossy().into(), v)) }, Err(err) => issues.push(format!("{}: {err}", mf.display())) } }
        }
        manifests.sort_by(|a, b| a.0.cmp(&b.0));
        let stub_assets: Vec<String> = manifests.iter().filter(|(_, v)| v["dev"]["status"] == "stub").map(|(_, v)| v["id"].as_str().unwrap_or("").to_string()).collect();
        for (_, v) in &manifests {
            match v["type"].as_str() {
                Some("body") => {
                    let root = v["dev"]["fileRoot"].as_str().map(|r| repo.join(r)).unwrap_or_else(|| dir.join(v["id"].as_str().unwrap_or("")).join("data"));
                    let id: String = v["id"].as_str().unwrap_or("").into();
                    let digest = digests.get(&id).cloned().unwrap_or_default();
                    bodies.push(BodyPackage { status: v["dev"]["status"].as_str().unwrap_or("").into(), digest, user: user_dirs.contains_key(&id), dir: user_dirs.get(&id).cloned(), id: v["id"].as_str().unwrap_or("").into(), version: v["version"].as_str().unwrap_or("").into(), label: label(&v["name"], "Body"), data_root: root, interface: v["body"]["provides"]["interface"].as_str().unwrap_or("").into(), capabilities: v["body"]["provides"]["capabilities"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() });
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
                        user: user_dirs.contains_key(v["id"].as_str().unwrap_or("")),
                        description: v["description"].as_str().unwrap_or("").into(),
                        body_id: v["requires"].as_array().and_then(|a| a.iter().filter_map(|r| r["id"].as_str()).find(|i| i.contains(".body."))).unwrap_or("").into(),
                        dir: user_dirs.get(v["id"].as_str().unwrap_or("")).cloned(),
                        digest: digests.get(v["id"].as_str().unwrap_or("")).cloned().unwrap_or_default(),
                    });
                }
                _ => {}
            }
        }
        let body = bodies.iter().find(|b| b.status != "draft").cloned().ok_or("kein Body-Package gefunden")?;
        if species.is_empty() { issues.push("keine Spezies gefunden".into()); }
        Ok(Packages { body, bodies, species, issues })
    }
}

pub fn sha256_hex(b: &[u8]) -> String { use sha2::{Digest, Sha256}; Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect() }
pub fn file_digest(p: &Path) -> String { std::fs::read(p).map(|b| sha256_hex(&b)).unwrap_or_default() }

pub fn user_dir(repo: &Path) -> PathBuf { repo.join("user-content/packages") }

pub fn slug(name: &str) -> String {
    let s: String = name.to_lowercase().chars().map(|c| match c { 'ä' => 'a', 'ö' => 'o', 'ü' => 'u', 'ß' => 's', c if c.is_ascii_alphanumeric() => c, _ => '-' }).collect();
    s.split('-').filter(|x| !x.is_empty()).collect::<Vec<_>>().join("-")
}

fn bump(v: &str, level: usize) -> String {
    let mut p: Vec<u32> = v.split('.').map(|x| x.parse().unwrap_or(0)).collect();
    while p.len() < 3 { p.push(0); }
    match level { 0 => { p[0] += 1; p[1] = 0; p[2] = 0 } 1 => { p[1] += 1; p[2] = 0 } _ => p[2] += 1 }
    format!("{}.{}.{}", p[0], p[1], p[2])
}
/// SemVer-Anhebung (0 = major, 1 = minor, 2 = patch).
pub fn bump_version(v: &str, level: usize) -> String { bump(v, level) }

impl Species {
    /// Schreibt die Spezies als installierbares Package (`hase-package.json`) nach `user-content/packages/<id>/`.
    pub fn write(&self, repo: &Path, body: &BodyPackage) -> Result<PathBuf, String> {
        let dir = self.dir.clone().unwrap_or_else(|| user_dir(repo).join(&self.id));
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let presets: Vec<Value> = self.presets.iter().map(|p| serde_json::json!({"id": p.id, "label": {"de": p.label}, "parameters": p.params})).collect();
        let traits: serde_json::Map<String, Value> = self.traits.iter().map(|t| (t.id.clone(), serde_json::json!({"label": {"de": t.label}, "set": t.set}))).collect();
        let limits: serde_json::Map<String, Value> = self.limits.iter().map(|(k, v)| (k.clone(), serde_json::json!([v[0], v[1]]))).collect();
        let comps: Vec<Value> = self.components.iter().map(|c| serde_json::json!({"asset": c.asset, "slot": c.slot})).collect();
        let mut requires = vec![serde_json::json!({"id": body.id, "range": format!("^{}", body.version.split('.').next().unwrap_or("1")) + ".0"})];
        for c in &self.components { requires.push(serde_json::json!({"id": c.asset, "range": "^1.0"})); }
        let m = serde_json::json!({
            "schemaVersion": "1.0", "id": self.id, "type": "species", "version": self.version,
            "name": {"de": self.label}, "description": self.description, "authors": ["Benutzer"], "tags": ["user"],
            "license": {"spdx": "CC0-1.0", "attribution": null, "redistribution": true},
            "requires": requires, "files": [], "dev": {"status": "user"},
            "species": {
                "targets": {"bodyInterface": {"id": body.interface, "range": "^1.0"}},
                "defaults": self.defaults, "limits": limits, "traits": traits, "components": comps, "capabilities": [], "presets": presets
            }
        });
        let f = dir.join("hase-package.json");
        std::fs::write(&f, serde_json::to_string_pretty(&m).unwrap()).map_err(|e| format!("{}: {e}", f.display()))?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_species_roundtrip() {
        let repo = crate::find_repo_root(&std::env::current_dir().unwrap()).expect("repo root");
        let mut p = Packages::load(&repo).unwrap();
        let mut sp = p.species.iter().find(|s| s.id.ends_with(".human")).unwrap().clone();
        sp.id = "user.species.test-roundtrip".into(); sp.digest.clear(); sp.label = "Test".into(); sp.user = true; sp.dir = None;
        sp.defaults.insert("mh:macro-age".into(), 0.7);
        sp.presets.push(Preset { id: "x".into(), label: "X".into(), params: [("mh:macro-gender".to_string(), 1.0f32)].into_iter().collect() });
        let dir = sp.write(&repo, &p.body).unwrap();
        p = Packages::load(&repo).unwrap();
        let back = p.species.iter().find(|s| s.id == "user.species.test-roundtrip").expect("neu geladen");
        assert!(back.user && back.dir.is_some());
        assert_eq!(back.defaults.get("mh:macro-age"), Some(&0.7));
        assert_eq!(back.presets.len(), sp.presets.len());
        assert_eq!(bump_version("1.2.3", 1), "1.3.0");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// Legt eine Körperbasis als **Entwurf** an (Modelldatei wird kopiert, Prüfstufen im Manifest vermerkt). Nutzbar im Editor erst nach Aufbereitung.
pub fn write_body_draft(repo: &Path, label: &str, model: &Path, stages: &[(String, Option<bool>, String)]) -> Result<PathBuf, String> {
    let id = format!("user.body.{}", slug(label));
    let dir = user_dir(repo).join(&id);
    if dir.exists() { return Err(format!("{id} existiert bereits")); }
    std::fs::create_dir_all(dir.join("data")).map_err(|e| e.to_string())?;
    let fname = model.file_name().and_then(|f| f.to_str()).ok_or("ungültiger Dateiname")?;
    std::fs::copy(model, dir.join("data").join(fname)).map_err(|e| e.to_string())?;
    let m = serde_json::json!({
        "schemaVersion": "1.0", "id": id, "type": "body", "version": "0.1.0", "name": {"de": label},
        "description": "Importierter Körperbasis-Entwurf: noch nicht aufbereitet (siehe Prüfstufen).", "authors": ["Benutzer"], "tags": ["user", "draft"],
        "license": {"spdx": "LicenseRef-User", "attribution": null, "redistribution": false},
        "files": [], "dev": {"status": "draft", "source": fname, "stages": stages.iter().map(|(n, ok, d)| serde_json::json!({"name": n, "ok": ok, "detail": d})).collect::<Vec<_>>()},
        "body": {"provides": {"interface": "user.iface.unknown", "version": "0.1.0", "capabilities": []}}
    });
    std::fs::write(dir.join("hase-package.json"), serde_json::to_string_pretty(&m).unwrap()).map_err(|e| e.to_string())?;
    Ok(dir)
}
