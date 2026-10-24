//! Import fremder Körpermodelle (glTF/GLB/VRM) als Körperbasis: Geometrie, Morph-Targets, Skelett, Aufbereitungs-Konfiguration
//! (Landmarken, Regionszuordnung, Bone-Mapping, Symmetrie). Die Konfiguration wird im Package gespeichert.
use crate::body::{Body, Morphs};
use crate::obj::{build_render_topo, ObjData};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const LANDMARKS: [(&str, &str); 5] = [("top", "Scheitel"), ("chin", "Kinn"), ("shoulder", "Schulter (links des Charakters)"), ("crotch", "Schritt"), ("sole", "Fußsohle")];

pub const VRM_SLOTS: [(&str, &[&str]); 15] = [
    ("hips", &["hips", "pelvis", "mixamorig:hips"]), ("spine", &["spine", "spine_01", "mixamorig:spine"]), ("head", &["head", "mixamorig:head"]),
    ("leftUpperLeg", &["upperleg_l", "thigh_l", "leftupleg", "mixamorig:leftupleg"]), ("rightUpperLeg", &["upperleg_r", "thigh_r", "rightupleg", "mixamorig:rightupleg"]),
    ("leftLowerLeg", &["lowerleg_l", "calf_l", "leftleg", "mixamorig:leftleg"]), ("rightLowerLeg", &["lowerleg_r", "calf_r", "rightleg", "mixamorig:rightleg"]),
    ("leftFoot", &["foot_l", "leftfoot", "mixamorig:leftfoot"]), ("rightFoot", &["foot_r", "rightfoot", "mixamorig:rightfoot"]),
    ("leftUpperArm", &["upperarm_l", "leftarm", "mixamorig:leftarm"]), ("rightUpperArm", &["upperarm_r", "rightarm", "mixamorig:rightarm"]),
    ("leftLowerArm", &["lowerarm_l", "leftforearm", "mixamorig:leftforearm"]), ("rightLowerArm", &["lowerarm_r", "rightforearm", "mixamorig:rightforearm"]),
    ("leftHand", &["hand_l", "lefthand", "mixamorig:lefthand"]), ("rightHand", &["hand_r", "righthand", "mixamorig:righthand"]),
];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImportConfig {
    pub source: String,
    pub primitive: usize,
    pub scale: f32,
    pub landmarks: BTreeMap<String, [f32; 3]>,
    pub regions: BTreeMap<String, String>, // Morph-Name -> Regionsname
    pub bones: BTreeMap<String, String>,   // VRM-Slot -> Joint-Name
    pub mirror: Vec<[String; 2]>,          // Morph-Namen links/rechts
}

#[derive(Clone, Debug)]
pub struct PrimInfo { pub label: String, pub verts: usize, pub tris: usize, pub morphs: usize, pub skinned: bool }
#[derive(Clone, Debug)]
pub struct Joint { pub name: String, pub parent: Option<usize>, pub pos: [f32; 3] }

pub struct Model { pub path: PathBuf, pub prims: Vec<PrimInfo>, pub joints: Vec<Joint> }

struct Loaded { doc: gltf::Document, bufs: Vec<Vec<u8>> }

fn load(path: &Path) -> Result<Loaded, String> {
    let g = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
    let mut bufs = Vec::new();
    for b in g.document.buffers() {
        match b.source() {
            gltf::buffer::Source::Bin => bufs.push(g.blob.clone().ok_or("GLB ohne Binärblock")?),
            gltf::buffer::Source::Uri(u) => {
                if u.starts_with("data:") { return Err("eingebettete data:-URIs werden nicht unterstützt (bitte GLB exportieren)".into()); }
                bufs.push(std::fs::read(path.parent().unwrap_or(Path::new(".")).join(u)).map_err(|e| format!("{u}: {e}"))?);
            }
        }
    }
    Ok(Loaded { doc: g.document, bufs })
}

fn mat_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] { let mut r = [[0f32; 4]; 4]; for c in 0..4 { for rw in 0..4 { for k in 0..4 { r[c][rw] += a[k][rw] * b[c][k]; } } } r }

/// Weltmatrizen aller Knoten (Spalten-major wie glTF).
fn world_matrices(doc: &gltf::Document) -> Vec<[[f32; 4]; 4]> {
    let n = doc.nodes().count();
    let mut w = vec![[[0f32; 4]; 4]; n];
    fn rec(node: gltf::Node, parent: [[f32; 4]; 4], w: &mut Vec<[[f32; 4]; 4]>) {
        let m = mat_mul(parent, node.transform().matrix());
        w[node.index()] = m;
        for c in node.children() { rec(c, m, w); }
    }
    let ident = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for s in doc.scenes() { for n in s.nodes() { rec(n, ident, &mut w); } }
    w
}

impl Model {
    pub fn open(path: &Path) -> Result<Model, String> {
        let l = load(path)?;
        let mut prims = Vec::new();
        for m in l.doc.meshes() { for p in m.primitives() {
            let verts = p.get(&gltf::Semantic::Positions).map(|a| a.count()).unwrap_or(0);
            let tris = p.indices().map(|i| i.count() / 3).unwrap_or(0);
            prims.push(PrimInfo { label: format!("{} #{}", m.name().unwrap_or("Mesh"), p.index()), verts, tris, morphs: p.morph_targets().len(), skinned: false });
        } }
        let skinned_nodes: Vec<usize> = l.doc.nodes().filter(|n| n.skin().is_some()).filter_map(|n| n.mesh().map(|m| m.index())).collect();
        let mut k = 0; let mut idx = 0;
        for m in l.doc.meshes() { for _ in m.primitives() { if skinned_nodes.contains(&m.index()) { prims[idx].skinned = true; } idx += 1; k += 1; } }
        let _ = k;
        let w = world_matrices(&l.doc);
        let mut joints: Vec<Joint> = Vec::new();
        if let Some(skin) = l.doc.skins().next() {
            let js: Vec<usize> = skin.joints().map(|j| j.index()).collect();
            let mut parent_of: BTreeMap<usize, usize> = BTreeMap::new();
            for n in l.doc.nodes() { for c in n.children() { parent_of.insert(c.index(), n.index()); } }
            for &j in &js {
                let node = l.doc.nodes().nth(j).unwrap();
                joints.push(Joint { name: node.name().unwrap_or("").to_string(), parent: parent_of.get(&j).and_then(|p| js.iter().position(|x| x == p)), pos: [w[j][3][0], w[j][3][1], w[j][3][2]] });
            }
        }
        Ok(Model { path: path.to_path_buf(), prims, joints })
    }
}

/// Baut aus einem Primitive eine Körperbasis (Geometrie + sparse Morph-Targets) laut Konfiguration.
pub fn build_body(path: &Path, cfg: &ImportConfig) -> Result<(Body, Option<ImageData>), String> {
    let t0 = std::time::Instant::now();
    let l = load(path)?;
    let (mut mesh_extras_names, mut found) = (Vec::<String>::new(), None);
    let mut idx = 0;
    'o: for m in l.doc.meshes() { for p in m.primitives() { if idx == cfg.primitive { found = Some((m.clone(), p)); break 'o; } idx += 1; } }
    let (mesh, prim) = found.ok_or("Primitive nicht gefunden")?;
    if let Some(ex) = mesh.extras() { if let Ok(v) = serde_json::from_str::<serde_json::Value>(ex.get()) { if let Some(a) = v["targetNames"].as_array() { mesh_extras_names = a.iter().filter_map(|x| x.as_str().map(String::from)).collect(); } } }
    let reader = prim.reader(|b| l.bufs.get(b.index()).map(|v| v.as_slice()));
    let scale = if cfg.scale > 0.0 { cfg.scale } else { 1.0 };
    let raw: Vec<[f32; 3]> = reader.read_positions().ok_or("keine Positionen")?.map(|p| [p[0] * scale, p[1] * scale, p[2] * scale]).collect();
    let uv: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|t| t.into_f32().map(|u| [u[0], 1.0 - u[1]]).collect()).unwrap_or_else(|| vec![[0.0, 0.0]; raw.len()]);
    let idxs: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..raw.len() as u32).collect());
    // Positionen verschweißen (Exporte duplizieren Vertices an UV-/Normalen-Nähten): glatte Normalen, kompakte Morph-Deltas
    let mut weld: std::collections::HashMap<[i64; 3], u32> = std::collections::HashMap::new();
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let map: Vec<u32> = raw.iter().map(|p| { let k = [(p[0] * 1e5).round() as i64, (p[1] * 1e5).round() as i64, (p[2] * 1e5).round() as i64]; *weld.entry(k).or_insert_with(|| { pos.push(*p); pos.len() as u32 - 1 }) }).collect();
    let faces: Vec<Vec<(u32, u32)>> = idxs.chunks_exact(3).map(|t| t.iter().map(|&i| (map[i as usize], i)).collect()).collect();
    let obj = ObjData { pos: pos.clone(), uv, face_group: vec![0; faces.len()], faces, groups: vec!["body".into()] };
    let topo = build_render_topo(&obj, |g| g == "body");
    let mut names = Vec::new(); let (mut t_off, mut t_idx, mut t_delta) = (vec![0u32], Vec::<u32>::new(), Vec::<[f32; 3]>::new());
    for (ti, (dp, _, _)) in reader.read_morph_targets().enumerate() {
        let name = mesh_extras_names.get(ti).cloned().unwrap_or_else(|| format!("morph_{ti}"));
        if let Some(it) = dp {
            let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
            let mut entries: Vec<(u32, [f32; 3])> = Vec::new();
            for (v, d) in it.enumerate() { let d = [d[0] * scale, d[1] * scale, d[2] * scale]; if d[0].abs() + d[1].abs() + d[2].abs() > 1e-7 && seen.insert(map[v]) { entries.push((map[v], d)); } }
            entries.sort_by_key(|e| e.0);
            for (v, d) in entries { t_idx.push(v); t_delta.push(d); }
        }
        t_off.push(t_idx.len() as u32); names.push(name);
    }
    let image = prim.material().pbr_metallic_roughness().base_color_texture().and_then(|t| image_of(&l, t.texture().source()));
    Ok((Body { base_pos: pos, obj, topo, morphs: Morphs { names, t_off, t_idx, t_delta }, load_ms: t0.elapsed().as_secs_f64() * 1e3, from_cache: false }, image))
}

pub struct ImageData { pub w: u32, pub h: u32, pub rgba: Vec<u8> }

fn image_of(l: &Loaded, img: gltf::Image) -> Option<ImageData> {
    let bytes: Vec<u8> = match img.source() {
        gltf::image::Source::View { view, .. } => l.bufs.get(view.buffer().index())?.get(view.offset()..view.offset() + view.length())?.to_vec(),
        gltf::image::Source::Uri { .. } => return None,
    };
    let d = image::load_from_memory(&bytes).ok()?.to_rgba8();
    Some(ImageData { w: d.width(), h: d.height(), rgba: d.into_raw() })
}

fn bbox(pos: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in pos { for i in 0..3 { lo[i] = lo[i].min(p[i]); hi[i] = hi[i].max(p[i]); } }
    (lo, hi)
}

/// Landmarken-Vorschlag aus der Bounding-Box (Proportionen eines aufrechten Humanoiden; vom Benutzer am Modell zu korrigieren).
pub fn default_landmarks(pos: &[[f32; 3]]) -> BTreeMap<String, [f32; 3]> {
    let (lo, hi) = bbox(pos);
    let h = hi[1] - lo[1];
    let cx = (lo[0] + hi[0]) * 0.5;
    let mut m = BTreeMap::new();
    m.insert("top".into(), [cx, hi[1], 0.0]); m.insert("chin".into(), [cx, lo[1] + h * 0.86, 0.0]);
    m.insert("shoulder".into(), [cx + h * 0.12, lo[1] + h * 0.82, 0.0]); m.insert("crotch".into(), [cx, lo[1] + h * 0.47, 0.0]); m.insert("sole".into(), [cx, lo[1], 0.0]);
    m
}

pub const REGION_NAMES: [&str; 20] = ["Gesamtkörper", "Gesamtkopf", "Stirn & Schläfen", "Augen", "Brauen", "Nase", "Wangen", "Mund & Lippen", "Kiefer & Kinn", "Ohren", "Brust", "Hals", "Schultern & Torso", "Bauch", "Becken & Hüfte", "Gesäß", "Arme", "Hände", "Beine", "Füße"];

/// Region für einen Punkt anhand der Landmarken (Heuristik, korrigierbar).
pub fn region_for_point(p: [f32; 3], lm: &BTreeMap<String, [f32; 3]>) -> &'static str {
    let g = |k: &str| lm.get(k).copied().unwrap_or([0.0; 3]);
    let (top, chin, sh, cr, sole) = (g("top"), g("chin"), g("shoulder"), g("crotch"), g("sole"));
    let cx = (top[0] + sole[0]) * 0.5;
    let x = (p[0] - cx).abs(); let y = p[1];
    let half_sh = (sh[0] - cx).abs().max(1e-3);
    let head_h = (top[1] - chin[1]).max(1e-3);
    if y >= chin[1] {
        let r = (y - chin[1]) / head_h;
        let hw = head_h * 0.34;
        if x > hw * 0.95 && r > 0.2 && r < 0.65 { return "Ohren"; }
        return if r < 0.14 { "Kiefer & Kinn" } else if r < 0.3 { if x > hw * 0.6 { "Wangen" } else { "Mund & Lippen" } } else if r < 0.47 { if x > hw * 0.6 { "Wangen" } else { "Nase" } } else if r < 0.6 { "Augen" } else if r < 0.68 { "Brauen" } else if r < 0.85 { "Stirn & Schläfen" } else { "Gesamtkopf" };
    }
    if y >= sh[1] { return "Hals"; }
    if y >= cr[1] {
        let hand_x = half_sh * 3.1;
        if x > half_sh * 0.95 { return if x > hand_x { "Hände" } else { "Arme" }; }
        let t = (y - cr[1]) / (sh[1] - cr[1]).max(1e-3);
        return if t > 0.85 { "Schultern & Torso" } else if t > 0.6 { "Brust" } else if t > 0.3 { "Bauch" } else if p[2] < (top[2] - 0.02) && t < 0.2 { "Gesäß" } else { "Becken & Hüfte" };
    }
    let foot_h = (cr[1] - sole[1]) * 0.07;
    if y < sole[1] + foot_h { "Füße" } else { "Beine" }
}

/// Ordnet jedes Morph-Target anhand des Schwerpunkts seiner Verschiebungen einer Region zu.
pub fn suggest_regions(body: &Body, lm: &BTreeMap<String, [f32; 3]>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (t, name) in body.morphs.names.iter().enumerate() {
        let (a, b) = (body.morphs.t_off[t] as usize, body.morphs.t_off[t + 1] as usize);
        let (mut c, mut wsum) = ([0f32; 3], 0f32);
        for k in a..b { let d = body.morphs.t_delta[k]; let w = d[0].abs() + d[1].abs() + d[2].abs(); let p = body.base_pos[body.morphs.t_idx[k] as usize]; for i in 0..3 { c[i] += p[i] * w; } wsum += w; }
        if wsum <= 0.0 { continue; }
        if (b - a) * 10 > body.base_pos.len() * 6 { out.insert(name.clone(), "Gesamtkörper".to_string()); continue; }
        out.insert(name.clone(), region_for_point([c[0] / wsum, c[1] / wsum, c[2] / wsum], lm).to_string());
    }
    out
}

/// Links/Rechts-Paare aus Morph-Namen (_l/_r, .l/.r, Left/Right, -l-/-r-).
pub fn detect_mirror(names: &[String]) -> Vec<(usize, usize)> {
    let pairs = [("_l", "_r"), (".l", ".r"), ("_L", "_R"), (".L", ".R"), ("Left", "Right"), ("left", "right"), ("-l-", "-r-"), ("_left", "_right")];
    let mut out = Vec::new();
    for (i, n) in names.iter().enumerate() {
        'p: for (l, r) in pairs {
            for (pos, _) in n.match_indices(l) {
                // Treffer muss an einer Wortgrenze enden (nicht „_l“ in „_longer“)
                let end = pos + l.len();
                if n[end..].chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) && !l.starts_with(char::is_alphabetic) { continue; }
                let m = format!("{}{}{}", &n[..pos], r, &n[end..]);
                if let Some(j) = names.iter().position(|x| *x == m) { if i < j { out.push((i, j)); } break 'p; }
            }
        }
    }
    out
}

pub fn auto_bones(joints: &[Joint]) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for (slot, aliases) in VRM_SLOTS { if let Some(j) = joints.iter().find(|j| aliases.contains(&j.name.to_lowercase().as_str())) { m.insert(slot.to_string(), j.name.clone()); } }
    m
}

/// Strahl-Dreieck-Test (Möller–Trumbore) gegen alle Dreiecke; liefert (t, Dreiecksindex).
pub fn raycast(pos: &[[f32; 3]], y_off: f32, tris: &[[u32; 3]], o: [f32; 3], d: [f32; 3]) -> Option<(f32, usize)> {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let mut best: Option<(f32, usize)> = None;
    for (ti, t) in tris.iter().enumerate() {
        let g = |i: u32| { let p = pos[i as usize]; [p[0], p[1] + y_off, p[2]] };
        let (v0, v1, v2) = (g(t[0]), g(t[1]), g(t[2]));
        let (e1, e2) = (sub(v1, v0), sub(v2, v0));
        let h = cross(d, e2); let a = dot(e1, h);
        if a.abs() < 1e-9 { continue; }
        let f = 1.0 / a; let s = sub(o, v0); let u = f * dot(s, h);
        if !(0.0..=1.0).contains(&u) { continue; }
        let q = cross(s, e1); let v = f * dot(d, q);
        if v < 0.0 || u + v > 1.0 { continue; }
        let tt = f * dot(e2, q);
        if tt > 1e-4 && best.map(|b| tt < b.0).unwrap_or(true) { best = Some((tt, ti)); }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_testcandidate_body() {
        let repo = crate::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let p = repo.join("content/testcandidate/body_mpfb2_testcandidate.glb");
        if !p.exists() { eprintln!("GLB fehlt (nicht versioniert) – Test übersprungen"); return; }
        let m = Model::open(&p).unwrap();
        for (i, pr) in m.prims.iter().enumerate() { eprintln!("prim {i}: {} v{} t{} morphs{} skinned{}", pr.label, pr.verts, pr.tris, pr.morphs, pr.skinned); }
        let mut cfg = ImportConfig { source: "x".into(), scale: 1.0, ..Default::default() };
        let (body, img) = build_body(&p, &cfg).unwrap();
        eprintln!("verts {} morphs {} nnz {} img {} {:?}", body.base_pos.len(), body.morphs.names.len(), body.morphs.nnz(), img.is_some(), body.morphs.names.iter().take(5).collect::<Vec<_>>());
        cfg.landmarks = default_landmarks(&body.base_pos);
        let reg = suggest_regions(&body, &cfg.landmarks);
        let mut hist: BTreeMap<&str, usize> = BTreeMap::new();
        for r in reg.values() { *hist.entry(r.as_str()).or_default() += 1; }
        eprintln!("{hist:?}");
        let mir = detect_mirror(&body.morphs.names);
        eprintln!("mirror pairs {}", mir.len());
        assert!(body.morphs.names.len() > 100 && reg.len() > 100);
        let truth = [("eyes", "Augen"), ("nose", "Nase"), ("mouth", "Mund & Lippen"), ("ears", "Ohren"), ("eyebrows", "Brauen"), ("cheek", "Wangen"), ("chin", "Kiefer & Kinn"), ("forehead", "Stirn & Schläfen"), ("head", "Gesamtkopf"), ("neck", "Hals"), ("macrodetails", "Gesamtkörper"), ("breast", "Brust"), ("stomach", "Bauch"), ("hip", "Becken & Hüfte"), ("buttocks", "Gesäß"), ("arms", "Arme"), ("hands", "Hände"), ("legs", "Beine"), ("feet", "Füße"), ("torso", "Schultern & Torso")];
        let (mut ok, mut tot) = (0, 0);
        let mut conf: BTreeMap<String, usize> = BTreeMap::new();
        for (n, r) in &reg { if let Some((_, t)) = truth.iter().find(|(g, _)| n.starts_with(&format!("{g}__"))) { tot += 1; if r == t { ok += 1; } else { *conf.entry(format!("{t} -> {r}")).or_default() += 1; } } }
        eprintln!("Zuordnung korrekt: {ok}/{tot}"); let mut c: Vec<_> = conf.into_iter().collect(); c.sort_by_key(|x| std::cmp::Reverse(x.1)); eprintln!("{:?}", &c[..c.len().min(14)]);
        // Plausibilität: Augen-Targets landen im Kopfbereich
        let head = ["Gesamtkopf", "Stirn & Schläfen", "Augen", "Brauen", "Nase", "Wangen", "Mund & Lippen", "Kiefer & Kinn", "Ohren"];
        let eye_ok = reg.iter().filter(|(k, _)| k.contains("eye")).filter(|(_, v)| head.contains(&v.as_str())).count();
        let eye_all = reg.keys().filter(|k| k.contains("eye")).count();
        eprintln!("eye morphs in head zone: {eye_ok}/{eye_all}");
        assert!(eye_all == 0 || eye_ok * 10 >= eye_all * 8);
    }
}
