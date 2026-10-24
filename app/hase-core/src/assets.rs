//! Asset-Katalog (Entwicklungs-Bibliothek aus `content/testcandidate/downloads`), gefiltert nach Lizenzstatus (Provenienzregister).
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum Category { Hair, Eyebrows, Eyelashes, Clothes }
impl Category { pub fn label(&self) -> &'static str { match self { Category::Hair => "Haare", Category::Eyebrows => "Augenbrauen", Category::Eyelashes => "Wimpern", Category::Clothes => "Kleidung" } } }

#[derive(Clone, Debug)]
pub struct AssetEntry { pub id: String, pub name: String, pub category: Category, pub mhclo: PathBuf, pub license: String, pub needs_attribution: bool }

pub struct Catalog { pub assets: Vec<AssetEntry>, pub skins: Vec<(String, PathBuf)>, pub eye_colors: Vec<(String, PathBuf)>, pub skipped_locked: usize }

fn parse_csv_line(l: &str) -> Vec<String> {
    let (mut out, mut cur, mut q) = (vec![], String::new(), false);
    for ch in l.chars() { match ch { '"' => q = !q, ',' if !q => { out.push(std::mem::take(&mut cur)); } c => cur.push(c) } }
    out.push(cur);
    out
}

impl Catalog {
    pub fn scan(repo: &Path) -> Catalog {
        let dl = repo.join("content/testcandidate/downloads");
        let mut status: HashMap<String, (String, String)> = HashMap::new(); // asset -> (status, license)
        if let Ok(s) = std::fs::read_to_string(repo.join("content/PROVENANCE_downloads.csv")) {
            let mut lines = s.lines();
            let head: Vec<String> = lines.next().map(parse_csv_line).unwrap_or_default();
            let (ia, il, is) = (head.iter().position(|h| h == "asset"), head.iter().position(|h| h == "license"), head.iter().position(|h| h == "status"));
            if let (Some(ia), Some(il), Some(is)) = (ia, il, is) {
                for l in lines { let f = parse_csv_line(l); if f.len() > is.max(ia).max(il) { status.insert(f[ia].clone(), (f[is].clone(), f[il].clone())); } }
            }
        }
        let mut assets = Vec::new(); let mut skipped = 0;
        for (dir, cat) in [("hair", Category::Hair), ("eyebrows", Category::Eyebrows), ("eyelashes", Category::Eyelashes), ("clothes", Category::Clothes)] {
            let Ok(rd) = std::fs::read_dir(dl.join(dir)) else { continue };
            let mut v: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
            v.sort_by_key(|e| e.file_name());
            for e in v {
                let name = e.file_name().to_string_lossy().to_string();
                let mhclo = e.path().join(format!("{name}.mhclo"));
                if !mhclo.exists() { continue; }
                let (st, lic) = status.get(&name).cloned().unwrap_or(("CC0".into(), "CC0".into())); // System-Assets ohne CSV-Eintrag: Pack-Lizenz CC0
                if st.starts_with("GESPERRT") || st.starts_with("PRUEFEN") { skipped += 1; continue; }
                assets.push(AssetEntry { id: format!("{dir}/{name}"), name, category: cat.clone(), mhclo, license: lic, needs_attribution: st.starts_with("CC-BY") });
            }
        }
        let mut skins = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dl.join("skins")) {
            let mut v: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
            v.sort_by_key(|e| e.file_name());
            for e in v {
                if let Ok(files) = std::fs::read_dir(e.path()) {
                    let mut pngs: Vec<PathBuf> = files.flatten().map(|f| f.path()).filter(|p| p.extension().map(|x| x == "png").unwrap_or(false)).collect();
                    pngs.sort();
                    if let Some(p) = pngs.iter().find(|p| { let n = p.file_name().unwrap().to_string_lossy().to_lowercase(); n.contains("diffuse") || n.contains("skin") }).or(pngs.first()) { skins.push((e.file_name().to_string_lossy().into(), p.clone())); }
                }
            }
        }
        let mut eyes = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dl.join("eyes/materials")) {
            let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with("_eye.png")).collect();
            v.sort();
            for p in v { eyes.push((p.file_name().unwrap().to_string_lossy().trim_end_matches("_eye.png").to_string(), p)); }
        }
        Catalog { assets, skins, eye_colors: eyes, skipped_locked: skipped }
    }
}
