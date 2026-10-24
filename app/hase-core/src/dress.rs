//! Anprobe von MakeHuman-Kleidung auf fremden (importierten) Körpern.
//! Die MakeHuman-Basis wird in zwei Schritten auf die Haut des Zielmodells abgebildet:
//! 1. Skelett-Vorausrichtung: je humanoidem Bone eine Ähnlichkeitstransformation (Drehung + Skalierung entlang des Knochens), per Gewichten
//!    über die Basis geblendet – gleicht Proportionen und die A-/T-Pose-Differenz aus.
//! 2. Nicht-rigide ICP-Verfeinerung auf die Hautvertices des Ziels (Normalen-Kompatibilität, Glättung des Verschiebungsfelds); Kopf, Hände und Augen
//!    bleiben bei der Skelett-Ausrichtung (dort passen Topologie und Proportionen nicht zusammen).
//! Die Kleidung (.mhclo-Proxys) folgt dann der abgebildeten Basis und wird anschließend gegen die Zielhaut kollisionskorrigiert (`fit::push_out`).
use crate::export::{humanoid_map, ExBone};
use crate::fit::BodyGrid;
use crate::pose::{q_from_to, qrot, Q, QI};
use std::collections::BTreeMap;

pub struct Target<'a> {
    pub bones: &'a [ExBone],
    pub humanoid: &'a BTreeMap<String, usize>,
    pub skin_pos: &'a [[f32; 3]],
    pub skin_nrm: &'a [[f32; 3]],
}

pub struct Registered { pub base: Vec<[f32; 3]>, pub paired: usize, pub icp_rms_before: f32, pub icp_rms_after: f32, pub ms: f64 }

/// Kandidaten für den „Kind“-Bone je humanoidem Bone (Richtung der Knochenachse).
fn child_candidates(name: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let fixed: &[(&str, &[&str])] = &[("hips", &["spine"]), ("spine", &["chest", "upperChest", "neck"]), ("chest", &["upperChest", "neck"]), ("upperChest", &["neck"]), ("neck", &["head"])];
    for (a, c) in fixed { if *a == name { v.extend(c.iter().map(|s| s.to_string())); } }
    for side in ["left", "right"] {
        let s = |x: &str| format!("{side}{x}");
        if name == s("Shoulder") { v.push(s("UpperArm")); }
        if name == s("UpperArm") { v.push(s("LowerArm")); }
        if name == s("LowerArm") { v.push(s("Hand")); }
        if name == s("Hand") { v.push(s("MiddleProximal")); }
        if name == s("UpperLeg") { v.push(s("LowerLeg")); }
        if name == s("LowerLeg") { v.push(s("Foot")); }
        if name == s("Foot") { v.push(s("Toes")); }
        for f in ["Index", "Middle", "Ring", "Little"] { if name == s(&format!("{f}Proximal")) { v.push(s(&format!("{f}Intermediate"))); } if name == s(&format!("{f}Intermediate")) { v.push(s(&format!("{f}Distal"))); } }
        if name == s("ThumbMetacarpal") { v.push(s("ThumbProximal")); } if name == s("ThumbProximal") { v.push(s("ThumbDistal")); }
    }
    v
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

/// Nicht beteiligte Bereiche der ICP-Verfeinerung (Topologie/Proportionen passen nicht zusammen).
fn skip_icp(vrm: &str) -> bool { ["head", "Eye", "Hand", "Thumb", "Index", "Middle", "Ring", "Little", "jaw"].iter().any(|k| vrm.contains(k)) && !vrm.contains("Shoulder") }

pub fn register(mh_pos: &[[f32; 3]], body_verts: &[u32], tris: &[[u32; 3]], mh_bones: &[ExBone], mh_weights: &[Vec<(u16, f32)>], t: &Target) -> Result<Registered, String> {
    let t0 = std::time::Instant::now();
    let map = humanoid_map();
    let vrm_of: Vec<Option<String>> = mh_bones.iter().map(|b| map.iter().find(|(_, ue)| ue.eq_ignore_ascii_case(&b.name)).map(|(v, _)| v.clone()).filter(|v| t.humanoid.contains_key(v))).collect();
    let mh_by_vrm: BTreeMap<&str, usize> = vrm_of.iter().enumerate().filter_map(|(i, v)| v.as_deref().map(|v| (v, i))).collect();
    let paired = mh_by_vrm.len();
    for req in ["hips", "head", "leftUpperArm", "rightUpperArm", "leftUpperLeg", "rightUpperLeg", "leftFoot", "rightFoot"] { if !mh_by_vrm.contains_key(req) { return Err(format!("Zielmodell hat keinen humanoiden Bone „{req}“ – Anprobe nicht möglich.")); } }
    let nb = mh_bones.len();
    // Ähnlichkeitstransformation je zugeordnetem Bone: (Anker alt, Anker neu, Drehung, Skalierung)
    let mut xf: Vec<Option<([f32; 3], [f32; 3], Q, f32)>> = vec![None; nb];
    for (i, v) in vrm_of.iter().enumerate() {
        let Some(name) = v else { continue };
        let (j0, j1) = (mh_bones[i].pos, t.bones[t.humanoid[name]].pos);
        let mut dirs = None;
        for c in child_candidates(name) { if let Some(&ci) = mh_by_vrm.get(c.as_str()) { dirs = Some((sub(mh_bones[ci].pos, j0), sub(t.bones[t.humanoid[&c]].pos, j1))); break; } }
        if let Some((d0, d1)) = dirs { let (l0, l1) = (len(d0), len(d1)); if l0 > 1e-5 && l1 > 1e-5 { xf[i] = Some((j0, j1, q_from_to(d0, d1), l1 / l0)); } }
    }
    // Blätter und Bones ohne Zuordnung: Drehung/Skalierung vom nächsten Vorfahren, Anker = eigener Bone (falls zugeordnet) sonst Vorfahre
    let mut res: Vec<([f32; 3], [f32; 3], Q, f32)> = Vec::with_capacity(nb);
    for i in 0..nb {
        let mut a = i; let mut found = None;
        loop { if let Some(x) = xf[a] { found = Some(x); break; } match mh_bones[a].parent { Some(p) => a = p, None => break } }
        let (j0, j1, q, s) = found.unwrap_or((mh_bones[0].pos, t.bones[t.humanoid["hips"]].pos, QI, 1.0));
        if xf[i].is_none() {
            if let Some(v) = &vrm_of[i] { res.push((mh_bones[i].pos, t.bones[t.humanoid[v]].pos, q, s)); continue; }
        }
        res.push((j0, j1, q, s));
    }
    let pelvis = mh_by_vrm["hips"];
    let tr = |b: usize, p: [f32; 3]| { let (j0, j1, q, s) = res[b]; let r = qrot(q, sub(p, j0)); [j1[0] + s * r[0], j1[1] + s * r[1], j1[2] + s * r[2]] };
    let mut cur: Vec<[f32; 3]> = Vec::with_capacity(mh_pos.len());
    for (v, &p) in mh_pos.iter().enumerate() {
        let w = &mh_weights[v];
        if w.is_empty() { cur.push(tr(pelvis, p)); continue; }
        let (mut o, mut tot) = ([0f32; 3], 0f32);
        for &(b, wt) in w { let q = tr(b as usize, p); for c in 0..3 { o[c] += wt * q[c]; } tot += wt; }
        cur.push([o[0] / tot, o[1] / tot, o[2] / tot]);
    }
    let dom = dominant_names(body_verts, mh_bones, mh_weights);
    let skip: Vec<bool> = dom.iter().map(|n| skip_icp(n)).collect();
    let (r0, r1) = refine(&mut cur, body_verts, tris, &skip, t.skin_pos, t.skin_nrm, 10);
    Ok(Registered { base: cur, paired, icp_rms_before: r0, icp_rms_after: r1, ms: t0.elapsed().as_secs_f64() * 1e3 })
}

/// Skin-Gewichte für Kleidungs-Vertices: Gewichte des nächsten Hautvertex des Zielmodells.
pub fn transfer_weights(cloth: &[[f32; 3]], skin_pos: &[[f32; 3]], skin_joints: &[[u16; 4]], skin_weights: &[[f32; 4]], grid: &BodyGrid) -> Vec<([u16; 4], [f32; 4])> {
    cloth.iter().map(|&p| match grid.nearest_where(skin_pos, p, 8, |_| true) { Some((v, _)) => (skin_joints[v as usize], skin_weights[v as usize]), None => ([0; 4], [1.0, 0.0, 0.0, 0.0]) }).collect()
}

/// Dominanter humanoider Name je Körper-Vertex (über die MH-Gewichte und Bone-Vorfahren), unabhängig vom Zielmodell.
fn dominant_names(body_verts: &[u32], mh_bones: &[ExBone], mh_weights: &[Vec<(u16, f32)>]) -> Vec<String> {
    let map = humanoid_map();
    let name_of: Vec<Option<String>> = mh_bones.iter().map(|b| map.iter().find(|(_, ue)| ue.eq_ignore_ascii_case(&b.name)).map(|(v, _)| v.clone())).collect();
    body_verts.iter().map(|&v| mh_weights[v as usize].iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).and_then(|(b, _)| { let mut a = *b as usize; loop { if let Some(n) = &name_of[a] { return Some(n.clone()); } a = mh_bones[a].parent?; } }).unwrap_or_else(|| "hips".into())).collect()
}

/// Nicht-rigide ICP-Verfeinerung der Basis auf die Zielhaut. `skip[k]` = Körper-Vertex k nimmt nicht teil (wird nur geglättet).
fn refine(cur: &mut Vec<[f32; 3]>, body_verts: &[u32], tris: &[[u32; 3]], skip: &[bool], skin_pos: &[[f32; 3]], skin_nrm: &[[f32; 3]], iters: usize) -> (f32, f32) {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); cur.len()];
    for tr in tris { for k in 0..3 { let (a, b) = (tr[k], tr[(k + 1) % 3]); if !adj[a as usize].contains(&b) { adj[a as usize].push(b); } if !adj[b as usize].contains(&a) { adj[b as usize].push(a); } } }
    let ids: Vec<u32> = (0..skin_pos.len() as u32).collect();
    let grid = BodyGrid::build(skin_pos, &ids, 0.02);
    let mut rms = (0f32, 0f32);
    let mut nrm = Vec::new();
    for it in 0..iters {
        crate::eval::normals(cur, tris, &mut nrm);
        let mut disp = vec![[0f32; 3]; cur.len()];
        let mut fixed = vec![false; cur.len()];
        let (mut e2, mut cnt) = (0f32, 0f32);
        for (k, &v) in body_verts.iter().enumerate() {
            if skip[k] { continue; }
            let (p, n) = (cur[v as usize], nrm[v as usize]);
            let hit = grid.nearest_where(skin_pos, p, 4, |q| dot(skin_nrm[q as usize], n) > 0.3);
            if let Some((q, d2)) = hit { if d2.sqrt() < 0.08 { let d = sub(skin_pos[q as usize], p); let nq = skin_nrm[q as usize]; let along = dot(d, nq) * 0.8; disp[v as usize] = [nq[0] * along, nq[1] * along, nq[2] * along]; fixed[v as usize] = true; e2 += d2; cnt += 1.0; } }
        }
        let r = (e2 / cnt.max(1.0)).sqrt(); if it == 0 { rms.0 = r; } rms.1 = r;
        // Nicht beteiligte Vertices: Mittel der Nachbarn (Dirichlet auf den beteiligten); beteiligte nur leicht glätten (Detail bleibt)
        let first = disp.clone();
        for _ in 0..40 {
            let prev = disp.clone();
            for &v in body_verts { let v = v as usize; if fixed[v] || adj[v].is_empty() { continue; } let mut m = [0f32; 3]; for &j in &adj[v] { for c in 0..3 { m[c] += prev[j as usize][c]; } } let k = adj[v].len() as f32; for c in 0..3 { disp[v][c] = m[c] / k; } }
        }
        for _ in 0..3 {
            let prev = disp.clone();
            for &v in body_verts { let v = v as usize; if !fixed[v] || adj[v].is_empty() { continue; } let mut m = [0f32; 3]; for &j in &adj[v] { for c in 0..3 { m[c] += prev[j as usize][c]; } } let k = adj[v].len() as f32; for c in 0..3 { disp[v][c] = 0.5 * prev[v][c] + 0.5 * m[c] / k; } }
        }
        let _ = &first;
        for &v in body_verts { let v = v as usize; for c in 0..3 { cur[v][c] += disp[v][c]; } }
    }
    rms
}

/// Anpassung ohne Zielskelett (rohe Mesh-Datei): Ähnlichkeitstransformation (Höhe, Mitte) + ICP mit wachsender Reichweite.
pub fn register_rigless(mh_pos: &[[f32; 3]], body_verts: &[u32], tris: &[[u32; 3]], mh_bones: &[ExBone], mh_weights: &[Vec<(u16, f32)>], skin_pos: &[[f32; 3]], skin_nrm: &[[f32; 3]]) -> Result<Registered, String> {
    let t0 = std::time::Instant::now();
    if skin_pos.is_empty() { return Err("keine Hautvertices".into()); }
    let ext = |p: &mut dyn Iterator<Item = [f32; 3]>| p.fold(([f32::MAX; 3], [f32::MIN; 3]), |a, q| ([a.0[0].min(q[0]), a.0[1].min(q[1]), a.0[2].min(q[2])], [a.1[0].max(q[0]), a.1[1].max(q[1]), a.1[2].max(q[2])]));
    let (slo, shi) = ext(&mut body_verts.iter().map(|&v| mh_pos[v as usize]));
    let (tlo, thi) = ext(&mut skin_pos.iter().copied());
    let sc = (thi[1] - tlo[1]) / (shi[1] - slo[1]).max(1e-3);
    let (sc_x, tc_x) = ((slo[0] + shi[0]) * 0.5, (tlo[0] + thi[0]) * 0.5);
    let (sc_z, tc_z) = ((slo[2] + shi[2]) * 0.5, (tlo[2] + thi[2]) * 0.5);
    let mut cur: Vec<[f32; 3]> = mh_pos.iter().map(|p| [tc_x + (p[0] - sc_x) * sc, tlo[1] + (p[1] - slo[1]) * sc, tc_z + (p[2] - sc_z) * sc]).collect();
    let dom = dominant_names(body_verts, mh_bones, mh_weights);
    let skip: Vec<bool> = dom.iter().map(|n| skip_icp(n)).collect();
    let (r0, r1) = refine(&mut cur, body_verts, tris, &skip, skin_pos, skin_nrm, 32);
    if std::env::var("HASE_DBG").is_ok() {
        let ids: Vec<u32> = (0..skin_pos.len() as u32).collect(); let grid = BodyGrid::build(skin_pos, &ids, 0.02);
        let mut st: std::collections::BTreeMap<String, (f32, f32, f32, u32)> = Default::default();
        for (k, &v) in body_verts.iter().enumerate() { if let Some((q, d2)) = grid.nearest_where(skin_pos, cur[v as usize], 6, |_| true) { let d = sub(cur[v as usize], skin_pos[q as usize]); let sg = dot(d, skin_nrm[q as usize]); let e = st.entry(dom[k].clone()).or_default(); e.0 += sg; e.1 = e.1.min(sg); e.2 = e.2.max(sg); e.3 += 1; let _ = d2; } }
        for (n, e) in st { eprintln!("DBG {n}: mean {:.1} mm min {:.1} max {:.1} n={}", e.0 / e.3 as f32 * 1000.0, e.1 * 1000.0, e.2 * 1000.0, e.3); }
    }
    Ok(Registered { base: cur, paired: 0, icp_rms_before: r0, icp_rms_after: r1, ms: t0.elapsed().as_secs_f64() * 1e3 })
}
