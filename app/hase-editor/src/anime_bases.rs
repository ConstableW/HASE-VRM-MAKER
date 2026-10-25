//! Registrierte Avatar-Basen (Körperbasis-Art „parametric“): VRM-Dateien des Owners, die als Ausgangsgeometrie + Rig für das Erschaffen dienen.
//! Ablage lokal in `user-content/anime-bases/<id>/{base.vrm, meta.json}` (nicht im Repository: Lizenz der Basisgeometrie liegt beim Owner, OD-024).
use hase_core::packages::sha256_hex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AnimeBase { pub id: String, pub name: String, pub sha: String, pub size: u64, pub dir: PathBuf }

pub fn root(repo: &Path) -> PathBuf { repo.join("user-content/anime-bases") }
impl AnimeBase { pub fn vrm(&self) -> PathBuf { self.dir.join("base.vrm") } }

pub fn list(repo: &Path) -> Vec<AnimeBase> {
    let mut v: Vec<AnimeBase> = std::fs::read_dir(root(repo)).map(|rd| rd.flatten().filter_map(|e| {
        let m: Value = serde_json::from_str(&std::fs::read_to_string(e.path().join("meta.json")).ok()?).ok()?;
        Some(AnimeBase { id: m["id"].as_str()?.into(), name: m["name"].as_str().unwrap_or("Basis").into(), sha: m["sha"].as_str().unwrap_or("").into(), size: m["size"].as_u64().unwrap_or(0), dir: e.path() })
    }).collect()).unwrap_or_default();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

pub fn find(repo: &Path, id: &str) -> Option<AnimeBase> { list(repo).into_iter().find(|b| b.id == id) }

/// Kopiert eine VRM-Datei als neue Basis (gleicher Inhalt → vorhandene Basis wiederverwenden).
pub fn register(repo: &Path, src: &Path, name: Option<&str>) -> Result<AnimeBase, String> {
    let data = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let sha = sha256_hex(&data);
    if let Some(b) = list(repo).into_iter().find(|b| b.sha == sha) { return Ok(b); }
    let nm = name.map(String::from).unwrap_or_else(|| src.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Basis".into()));
    let id = format!("{}-{}", hase_core::packages::slug(&nm), &sha[..8]);
    let dir = root(repo).join(&id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("base.vrm"), &data).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&json!({"id": id, "name": nm, "sha": sha, "size": data.len(), "source": src.display().to_string()})).unwrap()).map_err(|e| e.to_string())?;
    Ok(AnimeBase { id, name: nm, sha, size: data.len() as u64, dir })
}
