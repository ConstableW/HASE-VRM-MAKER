//! VRM-1.0-Export (GLB): Skelett aus dem MPFB2-Rig, Skin-Gewichte, Meshes mit Materialien (MToon), Morph-Targets für Expressions,
//! VRMC_vrm-Metadaten. Reine Datenschicht ohne GPU/UI – wird im Editor aus dem aktuellen Zustand zusammengesetzt und im Hintergrund geschrieben.
use crate::obj::ObjData;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub struct ExBone { pub name: String, pub parent: Option<usize>, pub pos: [f32; 3] }
pub struct ExMaterial { pub name: String, pub color: [f32; 4], pub image: Option<usize>, pub mask: bool }
pub struct ExImage { pub name: String, pub w: u32, pub h: u32, pub rgba: Vec<u8> }
pub struct ExMesh {
    pub name: String, pub pos: Vec<[f32; 3]>, pub nrm: Vec<[f32; 3]>, pub uv: Vec<[f32; 2]>, pub idx: Vec<u32>,
    pub joints: Vec<[u16; 4]>, pub weights: Vec<[f32; 4]>, pub material: usize, pub targets: Vec<(String, Vec<[f32; 3]>)>,
}
pub struct ExMeta { pub name: String, pub author: String, pub copyright: String, pub license_url: String, pub commercial: String, pub redistribution: bool }
pub struct ExDoc {
    pub meshes: Vec<ExMesh>, pub materials: Vec<ExMaterial>, pub images: Vec<ExImage>, pub bones: Vec<ExBone>,
    pub humanoid: BTreeMap<String, usize>,
    pub expressions: Vec<(String, Vec<(usize, usize, f32)>)>, // (Name, [(Mesh, Target, Gewicht)])
    pub meta: ExMeta,
    pub springs: Vec<(String, Vec<usize>)>, // (Name, Bone-Kette) → VRMC_springBone
}

/// UE-Mannequin-Namen → VRM-Humanoid-Bones (nur Pflicht- und gängige optionale Bones).
pub fn humanoid_map() -> Vec<(String, String)> {
    let mut m: Vec<(String, String)> = [("hips", "pelvis"), ("spine", "spine_01"), ("chest", "spine_02"), ("upperChest", "spine_03"), ("neck", "neck_01"), ("head", "head"),
        ("leftShoulder", "clavicle_l"), ("leftUpperArm", "upperarm_l"), ("leftLowerArm", "lowerarm_l"), ("leftHand", "hand_l"),
        ("rightShoulder", "clavicle_r"), ("rightUpperArm", "upperarm_r"), ("rightLowerArm", "lowerarm_r"), ("rightHand", "hand_r"),
        ("leftUpperLeg", "thigh_l"), ("leftLowerLeg", "calf_l"), ("leftFoot", "foot_l"), ("leftToes", "ball_l"),
        ("rightUpperLeg", "thigh_r"), ("rightLowerLeg", "calf_r"), ("rightFoot", "foot_r"), ("rightToes", "ball_r")].iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    for (side, s) in [("left", "l"), ("right", "r")] {
        for (fv, fu) in [("Thumb", "thumb"), ("Index", "index"), ("Middle", "middle"), ("Ring", "ring"), ("Little", "pinky")] {
            let names: [&str; 3] = if fv == "Thumb" { ["Metacarpal", "Proximal", "Distal"] } else { ["Proximal", "Intermediate", "Distal"] };
            for (i, n) in names.iter().enumerate() { m.push((format!("{side}{fv}{n}"), format!("{fu}_0{}_{s}", i + 1))); }
        }
    }
    m
}

/// Rig-Rohdaten (einmal geladen): Bone-Namen/Eltern, Gelenk-Vertexlisten, Gewichte je OBJ-Vertex (bis 4 Einflüsse).
pub struct RigData { pub names: Vec<String>, pub parents: Vec<Option<usize>>, heads: Vec<Vec<u32>>, pub weights: Vec<Vec<(u16, f32)>> }

impl RigData {
    pub fn load(data_root: &Path, obj: &ObjData) -> Result<RigData, String> {
        let rig: Value = serde_json::from_str(&std::fs::read_to_string(data_root.join("rigs/rig.game_engine.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let rig = rig.as_object().ok_or("Rig: kein Objekt")?;
        let mut groups: HashMap<&str, std::collections::BTreeSet<u32>> = HashMap::new();
        for (f, &g) in obj.faces.iter().zip(&obj.face_group) { groups.entry(obj.groups[g as usize].as_str()).or_default().extend(f.iter().map(|c| c.0)); }
        let names: Vec<String> = rig.keys().cloned().collect();
        let index: HashMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (n.as_str(), i)).collect();
        let mut heads = Vec::new(); let mut parents = Vec::new();
        for n in &names {
            let b = &rig[n.as_str()]; let spec = &b["head"];
            let ids: Vec<u32> = match spec["strategy"].as_str() {
                Some("CUBE") => spec["cube_name"].as_str().and_then(|c| groups.get(c)).map(|g| g.iter().copied().collect()).unwrap_or_default(),
                Some("MEAN") => spec["vertex_indices"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as u32)).collect()).unwrap_or_default(),
                _ => vec![],
            };
            if ids.is_empty() { return Err(format!("Bone {n}: Gelenkposition nicht bestimmbar")); }
            heads.push(ids); parents.push(b["parent"].as_str().and_then(|p| index.get(p).copied()));
        }
        let w: Value = serde_json::from_str(&std::fs::read_to_string(data_root.join("rigs/weights.game_engine.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let mut per: Vec<Vec<(u16, f32)>> = vec![Vec::new(); obj.pos.len()];
        for (bone, lst) in w["weights"].as_object().ok_or("Gewichte: kein Objekt")? {
            let Some(&bi) = index.get(bone.as_str()) else { continue };
            for e in lst.as_array().into_iter().flatten() { if let (Some(v), Some(wt)) = (e[0].as_u64(), e[1].as_f64()) { if (v as usize) < per.len() { per[v as usize].push((bi as u16, wt as f32)); } } }
        }
        for p in per.iter_mut() { p.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); p.truncate(4); let t: f32 = p.iter().map(|x| x.1).sum(); if t > 0.0 { for x in p.iter_mut() { x.1 /= t; } } }
        Ok(RigData { names, parents, heads, weights: per })
    }

    /// Bone-Positionen aus dem (gemorphten) Basismesh; `floor` hebt die Füße auf y = 0.
    pub fn bones(&self, pos: &[[f32; 3]], floor: f32) -> Vec<ExBone> {
        self.names.iter().enumerate().map(|(i, n)| {
            let (mut s, k) = ([0f32; 3], self.heads[i].len() as f32);
            for &v in &self.heads[i] { let p = pos[v as usize]; for c in 0..3 { s[c] += p[c]; } }
            ExBone { name: n.clone(), parent: self.parents[i], pos: [s[0] / k, s[1] / k + floor, s[2] / k] }
        }).collect()
    }
}

/// Rig laden (Kompatibilitätsfunktion für den Export).
pub fn load_rig(data_root: &Path, obj: &ObjData, pos: &[[f32; 3]], floor: f32) -> Result<(Vec<ExBone>, Vec<Vec<(u16, f32)>>), String> {
    let r = RigData::load(data_root, obj)?;
    Ok((r.bones(pos, floor), r.weights))
}

pub fn pack_weights(w: &[(u16, f32)], fallback: u16) -> ([u16; 4], [f32; 4]) {
    let (mut j, mut x) = ([0u16; 4], [0f32; 4]);
    if w.is_empty() { j[0] = fallback; x[0] = 1.0; return (j, x); }
    // Gewichte aus Barycentric-Mischungen können leicht negativ sein → auf ≥ 0 begrenzen und exakt normieren
    let mut sum = 0.0;
    for (k, (b, wt)) in w.iter().take(4).enumerate() { j[k] = *b; x[k] = wt.max(0.0); sum += x[k]; }
    if sum <= 1e-8 { return ([fallback, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]); }
    for v in x.iter_mut() { *v /= sum; }
    for k in 0..4 { if x[k] == 0.0 { j[k] = 0; } }
    (j, x)
}

fn pad4(v: &mut Vec<u8>) { while v.len() % 4 != 0 { v.push(0); } }

struct Bin { data: Vec<u8>, views: Vec<Value>, accs: Vec<Value> }
impl Bin {
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        pad4(&mut self.data);
        let off = self.data.len(); self.data.extend_from_slice(bytes);
        let mut v = json!({"buffer": 0, "byteOffset": off, "byteLength": bytes.len()});
        if let Some(t) = target { v["target"] = json!(t); }
        self.views.push(v); self.views.len() - 1
    }
    fn acc(&mut self, bytes: &[u8], target: Option<u32>, comp: u32, count: usize, ty: &str, minmax: Option<([f32; 3], [f32; 3])>) -> usize {
        let bv = self.view(bytes, target);
        let mut a = json!({"bufferView": bv, "componentType": comp, "count": count, "type": ty});
        if let Some((lo, hi)) = minmax { a["min"] = json!(lo); a["max"] = json!(hi); }
        self.accs.push(a); self.accs.len() - 1
    }
}

fn f32b(v: &[f32]) -> Vec<u8> { v.iter().flat_map(|x| x.to_le_bytes()).collect() }
fn bounds(p: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) { let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]); for q in p { for i in 0..3 { lo[i] = lo[i].min(q[i]); hi[i] = hi[i].max(q[i]); } } (lo, hi) }

pub fn write_glb(doc: &ExDoc) -> Result<Vec<u8>, String> {
    let mut bin = Bin { data: Vec::new(), views: vec![], accs: vec![] };
    let nb = doc.bones.len();
    // Knoten: Bones, dann Meshes
    let mut nodes: Vec<Value> = doc.bones.iter().map(|b| {
        let p = b.parent.map(|p| doc.bones[p].pos).unwrap_or([0.0; 3]);
        json!({"name": b.name, "translation": [b.pos[0] - p[0], b.pos[1] - p[1], b.pos[2] - p[2]]})
    }).collect();
    for (i, b) in doc.bones.iter().enumerate() { if let Some(p) = b.parent { let c = nodes[p].get_mut("children").and_then(|c| c.as_array_mut()).map(|c| { c.push(json!(i)); }); if c.is_none() { nodes[p]["children"] = json!([i]); } } }
    // inverse Bind-Matrizen (reine Translation)
    let mut ibm = Vec::with_capacity(nb * 16);
    for b in &doc.bones { ibm.extend_from_slice(&[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -b.pos[0], -b.pos[1], -b.pos[2], 1.0]); }
    let ibm_acc = bin.acc(&f32b(&ibm), None, 5126, nb, "MAT4", None);
    let root = doc.bones.iter().position(|b| b.parent.is_none()).unwrap_or(0);
    let skins = json!([{"inverseBindMatrices": ibm_acc, "joints": (0..nb).collect::<Vec<_>>(), "skeleton": root, "name": "Armature"}]);
    // Bilder/Texturen
    let mut images = Vec::new(); let mut textures = Vec::new();
    for im in &doc.images { let png = png_bytes(im.w, im.h, &im.rgba)?; let bv = bin.view(&png, None); images.push(json!({"bufferView": bv, "mimeType": "image/png", "name": im.name})); textures.push(json!({"sampler": 0, "source": images.len() - 1})); }
    let materials: Vec<Value> = doc.materials.iter().map(|m| {
        let mut pbr = json!({"baseColorFactor": m.color, "metallicFactor": 0.0, "roughnessFactor": 0.9});
        let mut mt = json!({"specVersion": "1.0", "shadeColorFactor": [m.color[0] * 0.72, m.color[1] * 0.72, m.color[2] * 0.72], "shadingToonyFactor": 0.9, "shadingShiftFactor": -0.1, "outlineWidthMode": "none", "giEqualizationFactor": 0.9});
        if let Some(t) = m.image { pbr["baseColorTexture"] = json!({"index": t}); mt["shadeMultiplyTexture"] = json!({"index": t}); }
        let mut mat = json!({"name": m.name, "pbrMetallicRoughness": pbr, "doubleSided": true, "extensions": {"KHR_materials_unlit": {}, "VRMC_materials_mtoon": mt}});
        mat["alphaMode"] = json!(if m.mask { "MASK" } else { "OPAQUE" });
        if m.mask { mat["alphaCutoff"] = json!(0.5); }
        mat
    }).collect();
    // Meshes
    let mut meshes = Vec::new();
    for (mi, m) in doc.meshes.iter().enumerate() {
        let n = m.pos.len();
        if m.nrm.len() != n || m.uv.len() != n || m.joints.len() != n || m.weights.len() != n { return Err(format!("{}: inkonsistente Attributanzahl", m.name)); }
        let nfix: Vec<[f32; 3]> = m.nrm.iter().map(|n| { let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt(); if l < 1e-6 { [0.0, 1.0, 0.0] } else { [n[0] / l, n[1] / l, n[2] / l] } }).collect();
        let pa = bin.acc(&f32b(&m.pos.iter().flatten().copied().collect::<Vec<_>>()), Some(34962), 5126, n, "VEC3", Some(bounds(&m.pos)));
        let na = bin.acc(&f32b(&nfix.iter().flatten().copied().collect::<Vec<_>>()), Some(34962), 5126, n, "VEC3", None);
        let ua = bin.acc(&f32b(&m.uv.iter().flatten().copied().collect::<Vec<_>>()), Some(34962), 5126, n, "VEC2", None);
        let ja = bin.acc(&m.joints.iter().flatten().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>(), Some(34962), 5123, n, "VEC4", None);
        let wa = bin.acc(&f32b(&m.weights.iter().flatten().copied().collect::<Vec<_>>()), Some(34962), 5126, n, "VEC4", None);
        let ia = bin.acc(&m.idx.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>(), Some(34963), 5125, m.idx.len(), "SCALAR", None);
        let mut targets = Vec::new();
        for (_, d) in &m.targets { if d.len() != n { return Err(format!("{}: Morph-Target-Länge", m.name)); } let ta = bin.acc(&f32b(&d.iter().flatten().copied().collect::<Vec<_>>()), Some(34962), 5126, n, "VEC3", Some(bounds(d))); targets.push(json!({"POSITION": ta})); }
        let mut prim = json!({"attributes": {"POSITION": pa, "NORMAL": na, "TEXCOORD_0": ua, "JOINTS_0": ja, "WEIGHTS_0": wa}, "indices": ia, "material": m.material, "mode": 4});
        if !targets.is_empty() { prim["targets"] = json!(targets); }
        let mut mesh = json!({"name": m.name, "primitives": [prim]});
        if !m.targets.is_empty() { mesh["extras"] = json!({"targetNames": m.targets.iter().map(|t| t.0.clone()).collect::<Vec<_>>()}); }
        meshes.push(mesh);
        nodes.push(json!({"name": m.name, "mesh": mi, "skin": 0}));
    }
    let mut scene_nodes: Vec<usize> = doc.bones.iter().enumerate().filter(|(_, b)| b.parent.is_none()).map(|(i, _)| i).collect();
    scene_nodes.extend(nb..nb + doc.meshes.len());
    // VRM
    let mut hb = serde_json::Map::new();
    for (k, v) in &doc.humanoid { hb.insert(k.clone(), json!({"node": v})); }
    let mut expr = serde_json::Map::new();
    for (name, binds) in &doc.expressions { expr.insert(name.clone(), json!({"morphTargetBinds": binds.iter().map(|(m, t, w)| json!({"node": nb + m, "index": t, "weight": w})).collect::<Vec<_>>()})); }
    let vrm = json!({"specVersion": "1.0",
        "meta": {"name": doc.meta.name, "version": "1.0", "authors": [doc.meta.author], "copyrightInformation": doc.meta.copyright, "licenseUrl": doc.meta.license_url,
                 "avatarPermission": "everyone", "commercialUsage": doc.meta.commercial, "creditNotation": "unnecessary", "allowRedistribution": doc.meta.redistribution, "modification": "allowModificationRedistribution",
                 "allowExcessivelyViolentUsage": false, "allowExcessivelySexualUsage": false, "allowPoliticalOrReligiousUsage": false, "allowAntisocialOrHateUsage": false},
        "humanoid": {"humanBones": hb}, "expressions": {"preset": expr}});
    let mut gltf = json!({
        "asset": {"version": "2.0", "generator": "HASE VRM MAKER"},
        "extensionsUsed": ["KHR_materials_unlit", "VRMC_materials_mtoon", "VRMC_vrm"],
        "scene": 0, "scenes": [{"name": "Scene", "nodes": scene_nodes}], "nodes": nodes, "skins": skins, "meshes": meshes, "materials": materials,
        "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}],
        "extensions": {"VRMC_vrm": vrm}
    });
    if !doc.springs.is_empty() {
        let springs: Vec<Value> = doc.springs.iter().map(|(n, ch)| json!({"name": n, "joints": ch.iter().map(|&b| json!({"node": b, "hitRadius": 0.02, "stiffness": 0.8, "gravityPower": 0.15, "gravityDir": [0.0, -1.0, 0.0], "dragForce": 0.35})).collect::<Vec<_>>()})).collect();
        gltf["extensions"]["VRMC_springBone"] = json!({"specVersion": "1.0", "springs": springs});
        gltf["extensionsUsed"].as_array_mut().unwrap().push(json!("VRMC_springBone"));
    }
    if !images.is_empty() { gltf["images"] = json!(images); gltf["textures"] = json!(textures); }
    gltf["bufferViews"] = json!(bin.views); gltf["accessors"] = json!(bin.accs);
    pad4(&mut bin.data);
    gltf["buffers"] = json!([{"byteLength": bin.data.len()}]);
    let mut js = serde_json::to_vec(&gltf).map_err(|e| e.to_string())?;
    while js.len() % 4 != 0 { js.push(b' '); }
    let total = 12 + 8 + js.len() + 8 + bin.data.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF"); out.extend_from_slice(&2u32.to_le_bytes()); out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(js.len() as u32).to_le_bytes()); out.extend_from_slice(b"JSON"); out.extend_from_slice(&js);
    out.extend_from_slice(&(bin.data.len() as u32).to_le_bytes()); out.extend_from_slice(b"BIN\0"); out.extend_from_slice(&bin.data);
    Ok(out)
}

pub fn png_bytes(w: u32, h: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).ok_or("PNG: Bildgröße passt nicht")?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

/// Augentextur backen: Iris-Farben/Pupillengröße je Auge wie im Shader (Pupillen-Dilatation, Irisfarbverlauf, Limbusring).
pub fn bake_eye(rgba: &[u8], w: u32, h: u32, st: &crate::state::EyeStyle, left_center_is_first: bool) -> Vec<u8> {
    const C0: [f32; 2] = [0.705, 0.296]; const C1: [f32; 2] = [0.290, 0.710];
    let (ri, rp) = (0.118f32, 0.040f32);
    let mut out = rgba.to_vec();
    let sample = |u: f32, v: f32| -> [f32; 4] { let x = ((u * w as f32) as i32).clamp(0, w as i32 - 1) as usize; let y = ((v * h as f32) as i32).clamp(0, h as i32 - 1) as usize; let i = (y * w as usize + x) * 4; [rgba[i] as f32, rgba[i + 1] as f32, rgba[i + 2] as f32, rgba[i + 3] as f32] };
    let sm = |a: f32, b: f32, x: f32| { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
    for y in 0..h { for x in 0..w {
        let (u, v) = ((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
        let d0 = ((u - C0[0]).powi(2) + (v - C0[1]).powi(2)).sqrt(); let d1 = ((u - C1[0]).powi(2) + (v - C1[1]).powi(2)).sqrt();
        let first = d0 < d1; let c = if first { C0 } else { C1 };
        let is_left = first == left_center_is_first;
        let side = if st.linked || is_left { &st.left } else { &st.right };
        let (dx, dy) = (u - c[0], v - c[1]); let r = (dx * dx + dy * dy).sqrt();
        let psc = side.pupil; let rps = rp * psc;
        let mut r2 = r;
        if r < ri { r2 = if r < rps { r / psc.max(0.05) } else { rp + (r - rps) * (ri - rp) / (ri - rps).max(1e-4) }; }
        let (u2, v2) = if r > 1e-6 { (c[0] + dx / r * r2, c[1] + dy / r * r2) } else { (u, v) };
        let t = sample(u2, v2);
        let mut col = [t[0] / 255.0, t[1] / 255.0, t[2] / 255.0];
        if side.custom {
            let l = ((0.30 * col[0] + 0.59 * col[1] + 0.11 * col[2]) / 0.26).clamp(0.25, 1.9);
            let k = sm(rp * 1.1, ri, r2);
            let m = 1.0 - sm(ri * 0.93, ri * 1.02, r2); let pupil = 1.0 - sm(rp * 0.9, rp * 1.12, r2);
            for i in 0..3 { let ic = (side.inner[i] * (1.0 - k) + side.outer[i] * k) * l; let ic = ic * (1.0 - pupil) + 0.01 * pupil; col[i] = col[i] * (1.0 - m) + ic * m; }
        }
        let limb = sm(ri * 0.80, ri, r2) * (1.0 - sm(ri, ri * 1.1, r2)) * st.limbal;
        let o = ((y * w + x) * 4) as usize;
        for i in 0..3 { out[o + i] = (col[i] * (1.0 - limb * 0.6) * 255.0).clamp(0.0, 255.0) as u8; }
        out[o + 3] = t[3] as u8;
    } }
    out
}

/// Rücklesen-Prüfung: (Meshes, Materialien, Bilder, Morph-Targets gesamt).
pub fn verify_glb(glb: &[u8]) -> Result<(usize, usize, usize, usize), String> {
    let g = gltf::Gltf::from_slice(glb).map_err(|e| format!("Rücklesen: {e}"))?;
    let d = &g.document;
    let morphs: usize = d.meshes().map(|m| m.primitives().map(|p| p.morph_targets().len()).sum::<usize>()).sum();
    Ok((d.meshes().count(), d.materials().count(), d.images().count(), morphs))
}
