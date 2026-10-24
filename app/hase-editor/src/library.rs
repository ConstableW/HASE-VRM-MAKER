//! Charakterbibliothek: persistente Charaktere (editierbare Projekte) mit Metadaten, Vorlagen, Favoriten, Varianten und Snapshots.
//! Ablage: `user-content/characters/<id>/{character.hase.json, meta.json, thumb.png, snapshots/*.json}`.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LibEntry { pub id: String, pub name: String, pub template: bool, pub favorite: bool, pub tags: Vec<String>, pub created: String, pub modified: String, pub parent: Option<String>, pub species: String, pub body: String, pub dir: PathBuf }

pub fn root(repo: &Path) -> PathBuf { repo.join("user-content/characters") }

/// UTC-Zeitstempel „JJJJ-MM-TT hh:mm“ ohne externe Abhängigkeit.
pub fn now_string() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02} {:02}:{:02}", if m <= 2 { y + 1 } else { y }, m, d, rem / 3600, (rem % 3600) / 60)
}

fn new_id(name: &str) -> String {
    let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos() ^ (d.as_secs() as u32)).unwrap_or(1);
    let s = hase_core::packages::slug(name);
    format!("{}-{:06x}", if s.is_empty() { "charakter" } else { &s }, n & 0xffffff)
}

fn read_meta(dir: &Path) -> Option<LibEntry> {
    let m: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json")).ok()?).ok()?;
    let c: Value = std::fs::read_to_string(dir.join("character.hase.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    Some(LibEntry {
        id: m["id"].as_str()?.into(), name: m["name"].as_str().unwrap_or("?").into(), template: m["template"].as_bool().unwrap_or(false), favorite: m["favorite"].as_bool().unwrap_or(false),
        tags: m["tags"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
        created: m["created"].as_str().unwrap_or("").into(), modified: m["modified"].as_str().unwrap_or("").into(), parent: m["parent"].as_str().map(String::from),
        species: c["species"]["id"].as_str().unwrap_or("").into(), body: c["body"]["id"].as_str().unwrap_or("").into(), dir: dir.to_path_buf(),
    })
}

pub fn list(repo: &Path) -> Vec<LibEntry> {
    let mut v: Vec<LibEntry> = std::fs::read_dir(root(repo)).map(|rd| rd.flatten().filter_map(|e| read_meta(&e.path())).collect()).unwrap_or_default();
    v.sort_by(|a, b| b.modified.cmp(&a.modified));
    v
}

fn write_meta(e: &LibEntry) -> Result<(), String> {
    let m = json!({"id": e.id, "name": e.name, "template": e.template, "favorite": e.favorite, "tags": e.tags, "created": e.created, "modified": e.modified, "parent": e.parent});
    std::fs::write(e.dir.join("meta.json"), serde_json::to_string_pretty(&m).unwrap()).map_err(|x| x.to_string())
}

/// Legt einen neuen Bibliothekseintrag an.
pub fn create(repo: &Path, name: &str, character: &Value, template: bool, parent: Option<&str>) -> Result<LibEntry, String> {
    let id = new_id(name);
    let dir = root(repo).join(&id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("character.hase.json"), serde_json::to_string_pretty(character).unwrap()).map_err(|e| e.to_string())?;
    let now = now_string();
    let e = LibEntry { id, name: name.into(), template, favorite: false, tags: vec![], created: now.clone(), modified: now, parent: parent.map(String::from), species: character["species"]["id"].as_str().unwrap_or("").into(), body: character["body"]["id"].as_str().unwrap_or("").into(), dir };
    write_meta(&e)?;
    Ok(e)
}

/// Überschreibt den gespeicherten Charakter (und merkt sich vorher einen Snapshot, wenn gewünscht).
pub fn update(e: &mut LibEntry, character: &Value) -> Result<(), String> {
    std::fs::write(e.dir.join("character.hase.json"), serde_json::to_string_pretty(character).unwrap()).map_err(|x| x.to_string())?;
    e.modified = now_string();
    write_meta(e)
}

pub fn load(e: &LibEntry) -> Result<Value, String> { serde_json::from_str(&std::fs::read_to_string(e.dir.join("character.hase.json")).map_err(|x| x.to_string())?).map_err(|x| x.to_string()) }

pub fn set_flags(e: &mut LibEntry) -> Result<(), String> { write_meta(e) }

pub fn rename(e: &mut LibEntry, name: &str) -> Result<(), String> { e.name = name.into(); e.modified = now_string(); write_meta(e) }

pub fn delete(e: &LibEntry) -> Result<(), String> { std::fs::remove_dir_all(&e.dir).map_err(|x| x.to_string()) }

/// Dupliziert als Variante (Verweis auf das Original, Thumbnail wird mitkopiert).
pub fn duplicate(repo: &Path, e: &LibEntry) -> Result<LibEntry, String> {
    let c = load(e)?;
    let n = create(repo, &format!("{} (Variante)", e.name), &c, false, Some(&e.id))?;
    let _ = std::fs::copy(e.dir.join("thumb.png"), n.dir.join("thumb.png"));
    Ok(n)
}

pub fn snapshots(e: &LibEntry) -> Vec<(String, PathBuf)> {
    let mut v: Vec<(String, PathBuf)> = std::fs::read_dir(e.dir.join("snapshots")).map(|rd| rd.flatten().map(|x| (x.file_name().to_string_lossy().trim_end_matches(".json").to_string(), x.path())).collect()).unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v
}

pub fn snapshot(e: &LibEntry, label: &str) -> Result<(), String> {
    let d = e.dir.join("snapshots");
    std::fs::create_dir_all(&d).map_err(|x| x.to_string())?;
    let name = format!("{}_{}", now_string().replace([' ', ':'], "-"), hase_core::packages::slug(label));
    std::fs::copy(e.dir.join("character.hase.json"), d.join(format!("{name}.json"))).map(|_| ()).map_err(|x| x.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("hase-lib-{}", std::process::id()));
        let repo = tmp.clone();
        let c = json!({"species": {"id": "x"}, "body": {"id": "b"}, "parameters": {"a": 0.5}});
        let mut e = create(&repo, "Mein Elf", &c, false, None).unwrap();
        assert_eq!(list(&repo).len(), 1);
        rename(&mut e, "Elfe Lyria").unwrap();
        e.favorite = true; e.template = true; set_flags(&mut e).unwrap();
        snapshot(&e, "vor Umbau").unwrap();
        assert_eq!(snapshots(&e).len(), 1);
        let d = duplicate(&repo, &e).unwrap();
        assert_eq!(d.parent.as_deref(), Some(e.id.as_str()));
        let l = list(&repo);
        assert_eq!(l.len(), 2);
        let back = l.iter().find(|x| x.id == e.id).unwrap();
        assert!(back.favorite && back.template && back.name == "Elfe Lyria" && back.species == "x");
        update(&mut e, &json!({"species": {"id": "y"}, "body": {"id": "b"}, "parameters": {}})).unwrap();
        assert_eq!(load(&e).unwrap()["species"]["id"], "y");
        delete(&d).unwrap();
        assert_eq!(list(&repo).len(), 1);
        let _ = std::fs::remove_dir_all(tmp);
        assert!(now_string().len() == 16);
    }
}
