//! Statisches VRM-/glTF-Modell mit Skelett: Teile (Meshes) mit Gewichten, Texturen, Blendshapes und humanoider Zuordnung.
//! Zum Ansehen, Posen (VRMA), Ausdrücken und Weiterreichen – die Geometrie wird nicht verändert.
use crate::export::ExBone;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub struct VrmImage { pub w: u32, pub h: u32, pub rgba: Vec<u8> }

pub struct VrmPart {
    pub name: String, pub pos: Vec<[f32; 3]>, pub nrm: Vec<[f32; 3]>, pub uv: Vec<[f32; 2]>, pub idx: Vec<u32>,
    pub joints: Vec<[u16; 4]>, pub weights: Vec<[f32; 4]>, pub image: Option<VrmImage>, pub color: [f32; 4], pub mask: bool, pub double_sided: bool,
    pub targets: Vec<(String, Vec<[f32; 3]>)>,
}

pub struct VrmModel {
    pub name: String, pub parts: Vec<VrmPart>, pub bones: Vec<ExBone>, pub humanoid: BTreeMap<String, usize>,
    pub expressions: Vec<(String, Vec<(usize, usize, f32)>)>, // (Name, [(Teil, Target, Gewicht)])
    pub spring_bones: bool, pub identity_rest: bool, pub height: f32, pub rest_rot: Vec<[f32; 4]>, pub vrm_names: Vec<Option<String>>,
}

fn mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] { let mut r = [[0f32; 4]; 4]; for c in 0..4 { for rw in 0..4 { for k in 0..4 { r[c][rw] += a[k][rw] * b[c][k]; } } } r }

/// Rotation (Quaternion x,y,z,w) einer Spalten-Major-Matrix (Skalierung wird herausnormiert).
fn mat_to_q(m: &[[f32; 4]; 4]) -> [f32; 4] {
    let n = |c: usize| { let l = (m[c][0] * m[c][0] + m[c][1] * m[c][1] + m[c][2] * m[c][2]).sqrt().max(1e-9); [m[c][0] / l, m[c][1] / l, m[c][2] / l] };
    let (x, y, z) = (n(0), n(1), n(2));
    // r[row][col] = Spalte col, Zeile row
    let (r00, r01, r02, r10, r11, r12, r20, r21, r22) = (x[0], y[0], z[0], x[1], y[1], z[1], x[2], y[2], z[2]);
    let tr = r00 + r11 + r22;
    let q = if tr > 0.0 { let s = (tr + 1.0).sqrt() * 2.0; [(r21 - r12) / s, (r02 - r20) / s, (r10 - r01) / s, 0.25 * s] }
        else if r00 > r11 && r00 > r22 { let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0; [0.25 * s, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s] }
        else if r11 > r22 { let s = (1.0 + r11 - r00 - r22).sqrt() * 2.0; [(r01 + r10) / s, 0.25 * s, (r12 + r21) / s, (r02 - r20) / s] }
        else { let s = (1.0 + r22 - r00 - r11).sqrt() * 2.0; [(r02 + r20) / s, (r12 + r21) / s, 0.25 * s, (r10 - r01) / s] };
    let l = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt().max(1e-9); [q[0] / l, q[1] / l, q[2] / l, q[3] / l]
}

pub fn load(path: &Path) -> Result<VrmModel, String> {
    let g = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
    let blob = g.blob.clone().ok_or("Datei hat keinen Binärblock (bitte .vrm/.glb)")?;
    let doc = &g.document;
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let jl = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
    let j: Value = serde_json::from_slice(&data[20..20 + jl]).map_err(|e| e.to_string())?;
    let skin = doc.skins().next().ok_or("Das Modell hat kein Skelett (Skin)")?;
    let joints: Vec<usize> = skin.joints().map(|n| n.index()).collect();
    let jidx: BTreeMap<usize, usize> = joints.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    // Weltmatrizen aller Knoten
    let nn = doc.nodes().count();
    let mut world = vec![[[0f32; 4]; 4]; nn];
    let ident = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    fn rec(n: gltf::Node, p: [[f32; 4]; 4], w: &mut Vec<[[f32; 4]; 4]>) { let m = mul(p, n.transform().matrix()); w[n.index()] = m; for c in n.children() { rec(c, m, w); } }
    for s in doc.scenes() { for n in s.nodes() { rec(n, ident, &mut world); } }
    let mut parent_of: BTreeMap<usize, usize> = BTreeMap::new();
    for n in doc.nodes() { for c in n.children() { parent_of.insert(c.index(), n.index()); } }
    let mut identity_rest = true;
    let mut bones = Vec::new(); let mut rest_rot: Vec<[f32; 4]> = Vec::new();
    for &n in &joints {
        let node = doc.nodes().nth(n).unwrap();
        let r = node.transform().decomposed().1; if (r[3].abs() - 1.0).abs() > 1e-3 { identity_rest = false; }
        let mut p = parent_of.get(&n).copied(); let mut par = None;
        while let Some(x) = p { if let Some(&k) = jidx.get(&x) { par = Some(k); break; } p = parent_of.get(&x).copied(); }
        rest_rot.push(mat_to_q(&world[n]));
        bones.push(ExBone { name: node.name().unwrap_or("").to_string(), parent: par, pos: [world[n][3][0], world[n][3][1], world[n][3][2]] });
    }
    // Humanoid-Zuordnung (VRM 1.0 oder VRM 0.x)
    let ibm: Vec<[[f32; 4]; 4]> = skin.reader(|_| Some(&blob[..])).read_inverse_bind_matrices().map(|i| i.collect()).unwrap_or_default();
    let mut humanoid = BTreeMap::new();
    if let Some(hb) = j["extensions"]["VRMC_vrm"]["humanoid"]["humanBones"].as_object() { for (k, v) in hb { if let Some(&b) = v["node"].as_u64().and_then(|x| jidx.get(&(x as usize))) { humanoid.insert(k.clone(), b); } } }
    else if let Some(hb) = j["extensions"]["VRM"]["humanoid"]["humanBones"].as_array() { for v in hb { if let (Some(k), Some(&b)) = (v["bone"].as_str(), v["node"].as_u64().and_then(|x| jidx.get(&(x as usize)))) { humanoid.insert(k.to_string(), b); } } }
    // Teile
    let mut parts: Vec<VrmPart> = Vec::new();
    let mut node_parts: BTreeMap<usize, Vec<usize>> = BTreeMap::new(); // Knoten -> Teil-Indizes
    for node in doc.nodes() {
        let Some(mesh) = node.mesh() else { continue };
        let tnames: Vec<String> = mesh.extras().as_ref().and_then(|e| serde_json::from_str::<Value>(e.get()).ok()).and_then(|v| v["targetNames"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())).unwrap_or_default();
        for prim in mesh.primitives() {
            let r = prim.reader(|_| Some(&blob[..]));
            let pos: Vec<[f32; 3]> = r.read_positions().ok_or("keine Positionen")?.collect();
            let n = pos.len();
            let nrm: Vec<[f32; 3]> = r.read_normals().map(|x| x.collect()).unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; n]);
            let uv: Vec<[f32; 2]> = r.read_tex_coords(0).map(|t| t.into_f32().collect()).unwrap_or_else(|| vec![[0.0, 0.0]; n]);
            let idx: Vec<u32> = r.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..n as u32).collect());
            let jo: Vec<[u16; 4]> = r.read_joints(0).map(|x| x.into_u16().collect()).unwrap_or_else(|| vec![[0; 4]; n]);
            let we: Vec<[f32; 4]> = r.read_weights(0).map(|x| x.into_f32().collect()).unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; n]);
            let mut targets = Vec::new();
            for (ti, (dp, _, _)) in r.read_morph_targets().enumerate() { if let Some(it) = dp { targets.push((tnames.get(ti).cloned().unwrap_or_else(|| format!("target_{ti}")), it.collect::<Vec<[f32; 3]>>())); } }
            let mat = prim.material(); let pbr = mat.pbr_metallic_roughness();
            let image = pbr.base_color_texture().and_then(|t| {
                let img = t.texture().source();
                let bytes: Vec<u8> = match img.source() { gltf::image::Source::View { view, .. } => blob.get(view.offset()..view.offset() + view.length())?.to_vec(), _ => return None };
                let d = image::load_from_memory(&bytes).ok()?.to_rgba8();
                Some(VrmImage { w: d.width(), h: d.height(), rgba: d.into_raw() })
            });
            node_parts.entry(node.index()).or_default().push(parts.len());
            parts.push(VrmPart { name: mesh.name().unwrap_or("Teil").to_string(), pos, nrm, uv, idx, joints: jo, weights: we, image, color: pbr.base_color_factor(), mask: !matches!(mat.alpha_mode(), gltf::material::AlphaMode::Opaque), double_sided: mat.double_sided(), targets });
        }
    }
    // Ruhe-Positionen = Skinning mit Ruhepose (Netze liegen im Mesh-Raum; für Modelle mit Ruhe-Rotationen/Skalierung nötig)
    if ibm.len() == joints.len() {
        let sk: Vec<[[f32; 4]; 4]> = joints.iter().enumerate().map(|(i, &n)| mul(world[n], ibm[i])).collect();
        let tp = |m: &[[f32; 4]; 4], p: [f32; 3]| [m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0], m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1], m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2]];
        let tn = |m: &[[f32; 4]; 4], p: [f32; 3]| [m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2], m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2], m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2]];
        for part in parts.iter_mut() {
            for v in 0..part.pos.len() {
                let (mut o, mut on) = ([0f32; 3], [0f32; 3]);
                for k in 0..4 { let w = part.weights[v][k]; if w > 0.0 { let m = &sk[part.joints[v][k] as usize]; let q = tp(m, part.pos[v]); let qn = tn(m, part.nrm[v]); for c in 0..3 { o[c] += w * q[c]; on[c] += w * qn[c]; } } }
                part.pos[v] = o;
                let l = (on[0] * on[0] + on[1] * on[1] + on[2] * on[2]).sqrt().max(1e-9); part.nrm[v] = [on[0] / l, on[1] / l, on[2] / l];
            }
            for t in part.targets.iter_mut() { for (v, d) in t.1.iter_mut().enumerate() { let mut o = [0f32; 3]; for k in 0..4 { let w = part.weights[v][k]; if w > 0.0 { let q = tn(&sk[part.joints[v][k] as usize], *d); for c in 0..3 { o[c] += w * q[c]; } } } *d = o; } }
        }
    }
    // Expressions (VRM 1.0): morphTargetBinds → (Teil, Target, Gewicht)
    let mut expressions = Vec::new();
    if let Some(pr) = j["extensions"]["VRMC_vrm"]["expressions"]["preset"].as_object() {
        for (name, e) in pr {
            let mut binds = Vec::new();
            for b in e["morphTargetBinds"].as_array().into_iter().flatten() {
                let (Some(nd), Some(ix)) = (b["node"].as_u64(), b["index"].as_u64()) else { continue };
                for &pi in node_parts.get(&(nd as usize)).into_iter().flatten() { if (ix as usize) < parts[pi].targets.len() { binds.push((pi, ix as usize, b["weight"].as_f64().unwrap_or(1.0) as f32)); } }
            }
            if !binds.is_empty() { expressions.push((name.clone(), binds)); }
        }
    }
    let mut vrm_names: Vec<Option<String>> = vec![None; bones.len()];
    for (k, &b) in &humanoid { vrm_names[b] = Some(k.clone()); }
    let height = parts.iter().flat_map(|p| p.pos.iter().map(|v| v[1])).fold(0f32, f32::max);
    let name = j["extensions"]["VRMC_vrm"]["meta"]["name"].as_str().or_else(|| j["extensions"]["VRM"]["meta"]["title"].as_str()).unwrap_or("Modell").to_string();
    Ok(VrmModel { name, parts, bones, humanoid, expressions, spring_bones: j["extensions"].get("VRMC_springBone").is_some() || j["extensions"]["VRM"].get("secondaryAnimation").is_some(), identity_rest, height, rest_rot, vrm_names })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_avatar_a_and_b() {
        let repo = crate::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        for f in ["avatar_a_quaternius_female.vrm", "avatar_b_makehuman.vrm"] {
            let m = load(&repo.join("content/testcandidate").join(f)).unwrap();
            eprintln!("{f}: {} Teile, {} Bones, humanoid {}, Expressions {}, Höhe {:.2}, identity_rest {}", m.parts.len(), m.bones.len(), m.humanoid.len(), m.expressions.len(), m.height, m.identity_rest);
            assert!(m.humanoid.len() >= 15 && !m.parts.is_empty());
        }
    }
}

#[cfg(test)]
mod lylia {
    use super::*;
    #[test]
    fn load_lylia_if_present() {
        let p = std::path::Path::new("/root/.claude/uploads/c759dbf5-f338-59fd-9a78-b4266d05211b/3df6c184-Lylia_v2.vrm");
        if !p.exists() { return; }
        let t = std::time::Instant::now();
        let m = load(p).unwrap();
        eprintln!("LYLIA {} Teile {:?}, {} Bones, humanoid {}, Expressions {}, Höhe {:.2}, identity_rest {}, spring {}, Laden {:?}", m.parts.len(), m.parts.iter().map(|x| (x.name.clone(), x.pos.len(), x.targets.len(), x.image.as_ref().map(|i| (i.w, i.h)))).collect::<Vec<_>>(), m.bones.len(), m.humanoid.len(), m.expressions.len(), m.height, m.identity_rest, m.spring_bones, t.elapsed());
    }
}
