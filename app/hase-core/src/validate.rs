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
        st("Morph-kompatibel", Some(!body.morphs.names.is_empty()), format!("{} Targets -> {} Parameter", body.morphs.names.len(), params.defs.len())),
        st("Als Körperbasis vollständig konfiguriert", Some(pairs > 0 && unassigned_params == 0 && humanoid), format!("{pairs} Links/Rechts-Paare, {unassigned_params} Parameter ohne Region, VRM-Humanoid-Fähigkeit {}", if humanoid { "deklariert" } else { "fehlt" })),
    ]
}

/// Probleme einer Spezies gegenüber dem Parameterkatalog (leer = validiert).
pub fn species_issues(sp: &Species, params: &Catalog) -> Vec<String> {
    let mut v = Vec::new();
    for k in sp.defaults.keys() { if params.canonical(k).is_none() { v.push(format!("Standardwert für unbekannten Parameter {k}")); } }
    for (k, l) in &sp.limits { match params.canonical(k) { None => v.push(format!("Grenze für unbekannten Parameter {k}")), Some(i) => { if l[0] > l[1] { v.push(format!("Grenze {k}: min > max")); } if let Some(d) = sp.defaults.get(k) { if *d < l[0] || *d > l[1] { v.push(format!("Standardwert {k} außerhalb der Grenzen")); } } let _ = i; } } }
    for p in &sp.presets { for k in p.params.keys() { if params.canonical(k).is_none() { v.push(format!("Preset „{}“: unbekannter Parameter {k}", p.label)); } } }
    for t in &sp.traits { for k in t.set.keys() { if params.canonical(k).is_none() { v.push(format!("Merkmal „{}“: unbekannter Parameter {k}", t.label)); } } }
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

const VRM_REQUIRED: &[(&str, &[&str])] = &[
    ("hips", &["hips", "pelvis", "mixamorig:hips"]), ("spine", &["spine", "spine_01", "mixamorig:spine"]), ("head", &["head", "mixamorig:head"]),
    ("upperLeg L", &["upperleg_l", "thigh_l", "leftupleg", "mixamorig:leftupleg"]), ("upperLeg R", &["upperleg_r", "thigh_r", "rightupleg", "mixamorig:rightupleg"]),
    ("lowerLeg L", &["lowerleg_l", "calf_l", "leftleg", "mixamorig:leftleg"]), ("lowerLeg R", &["lowerleg_r", "calf_r", "rightleg", "mixamorig:rightleg"]),
    ("foot L", &["foot_l", "leftfoot", "mixamorig:leftfoot"]), ("foot R", &["foot_r", "rightfoot", "mixamorig:rightfoot"]),
    ("upperArm L", &["upperarm_l", "leftarm", "mixamorig:leftarm"]), ("upperArm R", &["upperarm_r", "rightarm", "mixamorig:rightarm"]),
    ("lowerArm L", &["lowerarm_l", "leftforearm", "mixamorig:leftforearm"]), ("lowerArm R", &["lowerarm_r", "rightforearm", "mixamorig:rightforearm"]),
    ("hand L", &["hand_l", "lefthand", "mixamorig:lefthand"]), ("hand R", &["hand_r", "righthand", "mixamorig:righthand"]),
];

/// Analyse eines glTF/GLB/VRM-Modells: Geometrie, Skelett (VRM-Humanoid-Zuordnung bzw. Namensabgleich), Morph-Targets (Ausdrücke vs. Körper), Materialien.
pub fn analyze_gltf(path: &Path, body: &Body) -> Result<Vec<Stage>, String> {
    let g = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
    let doc = &g.document;
    let (mut verts, mut tris, mut morphs, mut prims) = (0usize, 0usize, 0usize, 0usize);
    for m in doc.meshes() { for p in m.primitives() {
        prims += 1;
        if let Some(a) = p.get(&gltf::Semantic::Positions) { verts += a.count(); }
        if let Some(i) = p.indices() { tris += i.count() / 3; }
        morphs += p.morph_targets().len();
    } }
    let skins = doc.skins().count();
    let joints: Vec<String> = doc.skins().flat_map(|s| s.joints().map(|j| j.name().unwrap_or("").to_lowercase()).collect::<Vec<_>>()).collect();
    // humanoide Zuordnung aus VRM 1.0 / 0.x
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let jv: Option<serde_json::Value> = if data.len() > 20 && &data[0..4] == b"glTF" { let jl = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize; serde_json::from_slice(&data[20..20 + jl.min(data.len() - 20)]).ok() } else { None };
    let mut mapped: Vec<String> = Vec::new();
    let mut expr_binds = 0usize;
    if let Some(j) = &jv {
        if let Some(hb) = j["extensions"]["VRMC_vrm"]["humanoid"]["humanBones"].as_object() { mapped = hb.keys().cloned().collect(); }
        else if let Some(hb) = j["extensions"]["VRM"]["humanoid"]["humanBones"].as_array() { mapped = hb.iter().filter_map(|v| v["bone"].as_str().map(String::from)).collect(); }
        if let Some(pr) = j["extensions"]["VRMC_vrm"]["expressions"]["preset"].as_object() { expr_binds = pr.values().map(|e| e["morphTargetBinds"].as_array().map(|a| a.len()).unwrap_or(0)).sum(); }
    }
    let required = ["hips", "spine", "head", "leftUpperLeg", "rightUpperLeg", "leftLowerLeg", "rightLowerLeg", "leftFoot", "rightFoot", "leftUpperArm", "rightUpperArm", "leftLowerArm", "rightLowerArm", "leftHand", "rightHand"];
    let (rig_ok, rig_detail) = if !mapped.is_empty() {
        let missing: Vec<&str> = required.iter().filter(|r| !mapped.iter().any(|m| m == *r)).copied().collect();
        (skins > 0 && missing.is_empty(), if missing.is_empty() { format!("{} Joints, VRM-Humanoid-Zuordnung vollständig ({} Bones)", joints.len(), mapped.len()) } else { format!("{} Joints; in der VRM-Zuordnung fehlen: {}", joints.len(), missing.join(", ")) })
    } else {
        let missing: Vec<&str> = VRM_REQUIRED.iter().filter(|(_, names)| !joints.iter().any(|j| names.contains(&j.as_str()))).map(|(n, _)| *n).collect();
        (skins > 0 && missing.is_empty(), if skins == 0 { "kein Skelett (Skin) im Modell".to_string() } else if missing.is_empty() { format!("{} Joints, alle 15 VRM-Pflicht-Bones über Namen zuordenbar", joints.len()) } else { format!("{} Joints, keine VRM-Zuordnung im Modell; über Namen nicht zuordenbar: {}", joints.len(), missing.join(", ")) })
    };
    let has_vrm = !mapped.is_empty() || doc.extensions_used().any(|e| e.contains("VRM"));
    let mats = doc.materials().count(); let imgs = doc.images().count();
    let same_topo = verts == body.topo.vmap.len();
    let (morph_ok, morph_detail) = if morphs == 0 { (false, "keine Morph-Targets – Körperparameter müssten ergänzt werden (z. B. Shape Keys in Blender)".to_string()) }
        else if expr_binds > 0 { (false, format!("{morphs} Blendshapes, die als Gesichtsausdrücke genutzt werden (keine Körper-Morphs) – als Modell zum Ansehen/Posen/Ausdrücken öffenbar, aber nicht als parametrische Körperbasis")) }
        else { (true, format!("{morphs} Shape Keys/Morph-Targets; Topologie {} zur aktiven Körperbasis", if same_topo { "identisch" } else { "abweichend" })) };
    Ok(vec![
        st("Importiert", Some(true), format!("{prims} Primitive, {verts} Vertices, {} Knoten{}", doc.nodes().count(), if has_vrm { ", VRM-Datei" } else { "" })),
        st("Geometrisch verwendbar", Some(verts > 0 && tris > 0), format!("{tris} Dreiecke")),
        st("Materialien/Texturen", Some(mats > 0), format!("{mats} Materialien, {imgs} Bilder")),
        st("Rig-kompatibel", Some(rig_ok), rig_detail),
        st("Morph-kompatibel", Some(morph_ok), morph_detail),
        st("Als Körperbasis vollständig konfiguriert", Some(false), "Regionen, Links/Rechts-Zuordnung und Morph-Kategorien sind noch nicht festgelegt"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gltf_analysis_on_testcandidate_avatars() {
        let repo = crate::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let body = Body::load(&repo.join("content/testcandidate/mpfb2-cc0"), &repo.join(".hase-cache")).unwrap();
        for f in ["avatar_a_quaternius_female.vrm", "avatar_b_makehuman.vrm"] {
            let r = analyze_gltf(&repo.join("content/testcandidate").join(f), &body).unwrap();
            for s in &r { eprintln!("{f}: {} {:?} {}", s.name, s.ok, s.detail); }
            assert_eq!(r[0].ok, Some(true));
            assert_eq!(r[1].ok, Some(true));
        }
    }
}
