//! Prüfstufen für Körperbasen/Spezies/Importe: ehrliche Unterscheidung zwischen importiert, geometrisch verwendbar,
//! Rig-kompatibel, Morph-kompatibel, konfiguriert und validiert.
use crate::body::Body;
use crate::packages::{BodyPackage, Species};
use crate::params::Catalog;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Stage { pub name: &'static str, pub ok: Option<bool>, pub detail: String }

fn st(name: &'static str, ok: Option<bool>, detail: impl Into<String>) -> Stage { Stage { name, ok, detail: detail.into() } }

fn rig_info(root: &Path) -> (Option<usize>, bool) {
    let rig = root.join("rigs/rig.game_engine.json");
    let bones = std::fs::read_to_string(&rig).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).and_then(|v| v.as_object().map(|o| o.len()));
    (bones, root.join("rigs/weights.game_engine.json").exists())
}

/// Stufen der aktiven Körperbasis.
pub fn body_stages(body: &Body, params: &Catalog, pkg: &BodyPackage, unassigned_params: usize) -> Vec<Stage> {
    let (nv, nt) = (body.topo.vmap.len(), body.topo.base_tris.len());
    let degenerate = body.topo.base_tris.iter().filter(|t| t[0] == t[1] || t[1] == t[2] || t[0] == t[2]).count();
    let (bones, weights) = rig_info(&pkg.data_root);
    let pairs = params.defs.iter().filter(|d| d.mirror.is_some()).count() / 2;
    let humanoid = pkg.capabilities.iter().any(|c| c.starts_with("humanoid:"));
    vec![
        st("Importiert", Some(true), format!("{} Vertices (inkl. Helfer), {} Gruppen", body.obj.pos.len(), body.obj.groups.len())),
        st("Geometrisch verwendbar", Some(degenerate == 0 && nv > 0), format!("{nv} sichtbare Vertices, {nt} Dreiecke, {degenerate} degenerierte")),
        st("Rig-kompatibel", Some(bones.is_some() && weights), match bones { Some(b) => format!("{b} Bones, Gewichte {}", if weights { "vorhanden" } else { "fehlen" }), None => "kein Rig gefunden".into() }),
        st("Morph-kompatibel", Some(!body.morphs.names.is_empty()), format!("{} Targets → {} Parameter", body.morphs.names.len(), params.defs.len())),
        st("Als Körperbasis vollständig konfiguriert", Some(pairs > 0 && unassigned_params == 0 && humanoid), format!("{pairs} Links/Rechts-Paare, {unassigned_params} Parameter ohne Region, VRM-Humanoid-Fähigkeit {}", if humanoid { "deklariert" } else { "fehlt" })),
    ]
}

/// Probleme einer Spezies gegenüber dem Parameterkatalog (leer = validiert).
pub fn species_issues(sp: &Species, params: &Catalog) -> Vec<String> {
    let mut v = Vec::new();
    for k in sp.defaults.keys() { if !params.index.contains_key(k) { v.push(format!("Standardwert für unbekannten Parameter {k}")); } }
    for (k, l) in &sp.limits { match params.index.get(k) { None => v.push(format!("Grenze für unbekannten Parameter {k}")), Some(&i) => { if l[0] > l[1] { v.push(format!("Grenze {k}: min > max")); } if let Some(d) = sp.defaults.get(k) { if *d < l[0] || *d > l[1] { v.push(format!("Standardwert {k} außerhalb der Grenzen")); } } let _ = i; } } }
    for p in &sp.presets { for k in p.params.keys() { if !params.index.contains_key(k) { v.push(format!("Preset „{}“: unbekannter Parameter {k}", p.label)); } } }
    for t in &sp.traits { for k in t.set.keys() { if !params.index.contains_key(k) { v.push(format!("Merkmal „{}“: unbekannter Parameter {k}", t.label)); } } }
    for c in &sp.components { if c.stub { v.push(format!("Komponente {} ist nur ein Stub (keine Geometrie)", c.asset)); } }
    if sp.body_id.is_empty() { v.push("Keine Körperbasis referenziert".into()); }
    v
}

/// Analyse eines externen OBJ-Modells gegenüber der aktiven Körperbasis – was ist vorhanden, was fehlt.
pub fn analyze_obj(path: &Path, body: &Body) -> Result<Vec<Stage>, String> {
    let o = crate::obj::parse_obj(path).map_err(|e| e.to_string())?;
    let tris: usize = o.faces.iter().map(|f| f.len().saturating_sub(2)).sum();
    let valid = o.faces.iter().all(|f| f.iter().all(|c| (c.0 as usize) < o.pos.len()));
    let same_topo = o.pos.len() == body.obj.pos.len() && o.faces.len() == body.obj.faces.len();
    let has_uv = !o.uv.is_empty();
    let dir = path.parent().unwrap_or(Path::new("."));
    let has_mat = std::fs::read_dir(dir).map(|rd| rd.flatten().any(|e| matches!(e.path().extension().and_then(|x| x.to_str()), Some("mtl") | Some("png") | Some("jpg")))).unwrap_or(false);
    Ok(vec![
        st("Importiert", Some(true), format!("{} Vertices, {} Flächen, {} Gruppen", o.pos.len(), o.faces.len(), o.groups.len())),
        st("Geometrisch verwendbar", Some(valid && !o.pos.is_empty() && tris > 0), format!("{tris} Dreiecke, UVs {}, Indizes {}", if has_uv { "vorhanden" } else { "fehlen" }, if valid { "gültig" } else { "fehlerhaft" })),
        st("Materialien/Texturen", Some(has_mat), if has_mat { "Material-/Bilddateien neben dem Modell gefunden".to_string() } else { "keine .mtl/.png/.jpg neben dem Modell".to_string() }),
        st("Rig-kompatibel", Some(false), "OBJ enthält kein Skelett – Rig und Gewichte müssen ergänzt werden (z. B. in Blender, Export als glTF)"),
        st("Morph-kompatibel", Some(same_topo), if same_topo { "Topologie identisch zur aktiven Körperbasis: vorhandene Morph-Targets sind anwendbar".to_string() } else { format!("Topologie weicht von der Körperbasis ab ({} vs. {} Vertices): eigene Morph-Targets nötig, Kleidungs-/Haar-Proxys nicht direkt wiederverwendbar", o.pos.len(), body.obj.pos.len()) }),
        st("Als Körperbasis vollständig konfiguriert", Some(false), "Regionen, Links/Rechts-Zuordnung, Bone-Mapping noch nicht definiert (Aufbereitung nötig)"),
    ])
}
