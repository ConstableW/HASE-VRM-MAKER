//! Posen/Animation: VRMA-Clips (VRM-Animation, humanoid) laden, auf das MPFB2-Rig retargeten und Skin-Matrizen berechnen.
//! Retargeting: Die VRMA-Dateien enthalten die Rotationen der Quellknoten samt deren Ruhe-Rotationen. Je humanoidem Bone wird die
//! Welt-Rotation relativ zur Quell-Ruhepose bestimmt, D(n) = A(n)·R(n)⁻¹ (A = animierte, R = Ruhe-Weltrotation), und auf unser Rig
//! (Ruherotationen = Identität) angewendet: lokal L(b) = G(parent)⁻¹·G(b) mit G = D der zugeordneten bzw. des nächsten humanoiden Vorfahren.
use crate::export::{humanoid_map, ExBone};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub type Q = [f32; 4]; // x, y, z, w
pub type M = [[f32; 4]; 3]; // Zeilen einer affinen 3×4-Matrix

pub const QI: Q = [0.0, 0.0, 0.0, 1.0];
pub fn qmul(a: Q, b: Q) -> Q { [a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1], a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0], a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3], a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2]] }
pub fn qinv(a: Q) -> Q { [-a[0], -a[1], -a[2], a[3]] }
fn qnorm(a: Q) -> Q { let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3]).sqrt().max(1e-9); [a[0] / l, a[1] / l, a[2] / l, a[3] / l] }
pub fn qslerp(a: Q, mut b: Q, t: f32) -> Q {
    let mut d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    if d < 0.0 { d = -d; b = [-b[0], -b[1], -b[2], -b[3]]; }
    if d > 0.9995 { return qnorm([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]); }
    let th = d.acos(); let (s0, s1) = (((1.0 - t) * th).sin() / th.sin(), (t * th).sin() / th.sin());
    [a[0] * s0 + b[0] * s1, a[1] * s0 + b[1] * s1, a[2] * s0 + b[2] * s1, a[3] * s0 + b[3] * s1]
}
pub fn qrot(q: Q, v: [f32; 3]) -> [f32; 3] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])]
}
/// Minimale Rotation, die Richtung a auf b dreht.
pub fn q_from_to(a: [f32; 3], b: [f32; 3]) -> Q {
    let n = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9); [v[0] / l, v[1] / l, v[2] / l] };
    let (a, b) = (n(a), n(b));
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    if d < -0.99999 { return [0.0, 0.0, 1.0, 0.0]; }
    qnorm([a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], 1.0 + d])
}
fn mat(q: Q, t: [f32; 3]) -> M {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    [[1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w), t[0]], [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w), t[1]], [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y), t[2]]]
}
fn mmul(a: &M, b: &M) -> M {
    let mut r = [[0f32; 4]; 3];
    for i in 0..3 { for j in 0..4 { r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + if j == 3 { a[i][3] } else { 0.0 }; } }
    r
}
pub fn mpoint(m: &M, p: [f32; 3]) -> [f32; 3] { [m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3], m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3], m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3]] }

pub struct Clip { pub name: String, pub duration: f32, parents: Vec<Option<usize>>, rest: Vec<Q>, bone_node: BTreeMap<String, usize>, rot: BTreeMap<usize, Vec<(f32, Q)>>, hips_pos: Vec<(f32, [f32; 3])>, hips_rest: [f32; 3] }

fn sample_q(k: &[(f32, Q)], t: f32) -> Q {
    if k.is_empty() { return QI; }
    if t <= k[0].0 { return k[0].1; }
    for w in k.windows(2) { if t <= w[1].0 { let u = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-9); return qslerp(w[0].1, w[1].1, u); } }
    k[k.len() - 1].1
}

impl Clip {
    pub fn load(path: &Path) -> Result<Clip, String> {
        let g = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
        let blob = g.blob.clone().ok_or("VRMA ohne Binärblock")?;
        let doc = &g.document;
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let jl = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let j: Value = serde_json::from_slice(&data[20..20 + jl]).map_err(|e| e.to_string())?;
        let hb = j["extensions"]["VRMC_vrm_animation"]["humanoid"]["humanBones"].as_object().ok_or("keine VRMC_vrm_animation-Humanoid-Zuordnung")?;
        let bone_node: BTreeMap<String, usize> = hb.iter().filter_map(|(k, v)| Some((k.clone(), v["node"].as_u64()? as usize))).collect();
        let n = doc.nodes().count();
        let mut parents = vec![None; n]; let mut rest = vec![QI; n];
        for nd in doc.nodes() { for c in nd.children() { parents[c.index()] = Some(nd.index()); } let r = nd.transform().decomposed().1; rest[nd.index()] = r; }
        let mut clip = Clip { name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("clip").to_string(), duration: 0.0, parents, rest, bone_node, rot: BTreeMap::new(), hips_pos: vec![], hips_rest: [0.0, 1.0, 0.0] };
        if let Some(&h) = clip.bone_node.get("hips") { if let Some(nd) = doc.nodes().nth(h) { let t = nd.transform().decomposed().0; clip.hips_rest = [t[0], t[1].max(1e-3), t[2]]; } }
        let hips_node = clip.bone_node.get("hips").copied();
        let anim = doc.animations().next().ok_or("keine Animation")?;
        for ch in anim.channels() {
            let ni = ch.target().node().index();
            let r = ch.reader(|_| Some(&blob[..]));
            let times: Vec<f32> = r.read_inputs().ok_or("Zeiten fehlen")?.collect();
            match r.read_outputs() {
                Some(gltf::animation::util::ReadOutputs::Rotations(rr)) => { clip.duration = clip.duration.max(times.last().copied().unwrap_or(0.0)); let q: Vec<Q> = rr.into_f32().collect(); clip.rot.insert(ni, times.iter().copied().zip(q).collect()); }
                Some(gltf::animation::util::ReadOutputs::Translations(tt)) if Some(ni) == hips_node => { clip.hips_pos = times.iter().copied().zip(tt).collect(); }
                _ => {}
            }
        }
        if clip.rot.is_empty() { return Err("Clip enthält keine Rotationskanäle".into()); }
        Ok(clip)
    }
    pub fn hips_rest_height(&self) -> f32 { self.hips_rest[1] }
    pub fn hips_rest(&self) -> [f32; 3] { self.hips_rest }

    /// Welt-Rotation relativ zur Quell-Ruhepose je humanoidem Bone zur Zeit t.
    pub fn deltas(&self, t: f32) -> BTreeMap<String, Q> {
        let n = self.rest.len();
        let (mut a, mut r): (Vec<Option<Q>>, Vec<Option<Q>>) = (vec![None; n], vec![None; n]);
        fn world(i: usize, c: &Clip, t: f32, a: &mut Vec<Option<Q>>, r: &mut Vec<Option<Q>>) -> (Q, Q) {
            if let (Some(x), Some(y)) = (a[i], r[i]) { return (x, y); }
            let (pa, pr) = match c.parents[i] { Some(p) => world(p, c, t, a, r), None => (QI, QI) };
            let local = c.rot.get(&i).map(|k| sample_q(k, t)).unwrap_or(c.rest[i]);
            let (wa, wr) = (qmul(pa, local), qmul(pr, c.rest[i]));
            a[i] = Some(wa); r[i] = Some(wr); (wa, wr)
        }
        let mut out = BTreeMap::new();
        for (name, &node) in &self.bone_node { let (wa, wr) = world(node, self, t, &mut a, &mut r); out.insert(name.clone(), qnorm(qmul(wa, qinv(wr)))); }
        out
    }
    pub fn hips(&self, t: f32) -> Option<[f32; 3]> {
        let k = &self.hips_pos; if k.is_empty() { return None; }
        if t <= k[0].0 { return Some(k[0].1); }
        for w in k.windows(2) { if t <= w[1].0 { let u = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-9); return Some([w[0].1[0] + (w[1].1[0] - w[0].1[0]) * u, w[0].1[1] + (w[1].1[1] - w[0].1[1]) * u, w[0].1[2] + (w[1].1[2] - w[0].1[2]) * u]); } }
        Some(k[k.len() - 1].1)
    }
}

pub struct PoseRig { bones: Vec<ExBone>, order: Vec<usize>, vrm: Vec<Option<String>>, c: Vec<Q>, rest: Vec<Q>, hips: usize }

impl PoseRig {
    /// MPFB2-Rig (UE-Mannequin-Namen, Ruherotationen = Identität).
    pub fn new(bones: Vec<ExBone>) -> PoseRig {
        let map = humanoid_map();
        let vrm: Vec<Option<String>> = bones.iter().map(|b| map.iter().find(|(_, ue)| ue.eq_ignore_ascii_case(&b.name)).map(|(v, _)| v.clone())).collect();
        let n = bones.len();
        PoseRig::generic(bones, vec![QI; n], vrm)
    }

    /// Beliebiges Rig: Welt-Ruherotationen und humanoide Namen je Bone (None = kein Humanoid-Bone, folgt dem Vorfahren).
    pub fn generic(bones: Vec<ExBone>, rest: Vec<Q>, vrm: Vec<Option<String>>) -> PoseRig {
        let mut order = Vec::new(); let mut done = vec![false; bones.len()];
        while order.len() < bones.len() { let before = order.len(); for i in 0..bones.len() { if !done[i] && bones[i].parent.map(|p| done[p]).unwrap_or(true) { done[i] = true; order.push(i); } } if order.len() == before { break; } }
        let slot = |name: &str| vrm.iter().position(|v| v.as_deref() == Some(name));
        // A-Pose-Ausgleich: Die Quell-Ruhepose der Clips ist eine T-Pose; C(b) dreht die eigene Ruherichtung der Arm-Ketten in die T-Richtung (±x).
        let mut c = vec![QI; bones.len()];
        for (sx, side) in [(1.0f32, "left"), (-1.0, "right")] {
            let chain = [format!("{side}Shoulder"), format!("{side}UpperArm"), format!("{side}LowerArm"), format!("{side}Hand")];
            let mut hand_q = QI;
            for (k, name) in chain.iter().enumerate() {
                let child = match k { 0 => slot(&chain[1]), 1 => slot(&chain[2]), 2 => slot(&chain[3]), _ => slot(&format!("{side}MiddleProximal")) };
                let (Some(b), Some(ch)) = (slot(name), child) else { continue };
                let d = [bones[ch].pos[0] - bones[b].pos[0], bones[ch].pos[1] - bones[b].pos[1], bones[ch].pos[2] - bones[b].pos[2]];
                c[b] = q_from_to(d, [sx, 0.0, 0.0]); if k == 3 { hand_q = c[b]; }
            }
            for (i, v) in vrm.iter().enumerate() { if let Some(v) = v { if v.starts_with(side) && ["Thumb", "Index", "Middle", "Ring", "Little"].iter().any(|f| v[side.len()..].starts_with(f)) { c[i] = hand_q; } } }
        }
        let hips = slot("hips").unwrap_or(0);
        PoseRig { bones, order, vrm, c, rest, hips }
    }
    pub fn bone_count(&self) -> usize { self.bones.len() }

    /// Skin-Matrizen (Pose-Welt × inverse Ruhe-Welt) je Bone. `anim`: (Clip, Zeit); None = Ruhepose.
    pub fn skin_matrices(&self, anim: Option<(&Clip, f32)>, hip_scale: f32) -> Vec<M> {
        let n = self.bones.len();
        let ident: M = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        let deltas = anim.map(|(c, t)| c.deltas(t));
        let mut dc: Vec<Q> = vec![QI; n]; // D·C des nächsten humanoiden Vorfahren (bzw. des Bones selbst)
        let mut g: Vec<Q> = vec![QI; n];   // Pose-Weltrotation
        let mut world: Vec<M> = vec![ident; n];
        for &i in &self.order {
            let b = &self.bones[i];
            let (pp, parent_world, gp, dcp, rp) = match b.parent { Some(p) => (self.bones[p].pos, world[p], g[p], dc[p], self.rest[p]), None => ([0.0; 3], ident, QI, QI, QI) };
            dc[i] = match (&deltas, &self.vrm[i]) { (Some(d), Some(name)) => d.get(name).map(|q| qmul(*q, self.c[i])).unwrap_or(dcp), _ => dcp };
            g[i] = qmul(dc[i], self.rest[i]);
            // lokale Ruhe-Translation im Eltern-Ruhe-Frame
            let mut t = qrot(qinv(rp), [b.pos[0] - pp[0], b.pos[1] - pp[1], b.pos[2] - pp[2]]);
            if i == self.hips { if let Some((c, tm)) = anim { if let Some(h) = c.hips(tm) { let r = c.hips_rest(); let d = [(h[0] - r[0]) * hip_scale, (h[1] - r[1]) * hip_scale, (h[2] - r[2]) * hip_scale]; let dl = qrot(qinv(gp), d); t = [t[0] + dl[0], t[1] + dl[1], t[2] + dl[2]]; } } }
            let l = qmul(qinv(gp), g[i]);
            world[i] = mmul(&parent_world, &mat(l, t));
        }
        (0..n).map(|i| {
            // inverse Ruhe-Welt: R⁻¹ · T(−pos)
            let ri = qinv(self.rest[i]); let p = self.bones[i].pos; let tp = qrot(ri, [-p[0], -p[1], -p[2]]);
            mmul(&world[i], &mat(ri, tp))
        }).collect()
    }
    pub fn hips_rest_y(&self) -> f32 { self.bones[self.hips].pos[1] }
}

/// Lineares Blend-Skinning eines Vertex.
pub fn skin_point(m: &[M], j: [u16; 4], w: [f32; 4], p: [f32; 3]) -> [f32; 3] {
    let mut o = [0f32; 3];
    for k in 0..4 { if w[k] > 0.0 { let q = mpoint(&m[j[k] as usize], p); o[0] += w[k] * q[0]; o[1] += w[k] * q[1]; o[2] += w[k] * q[2]; } }
    o
}

#[cfg(test)]
mod debug_tests {
    use super::*;
    #[test]
    fn print_arm_deltas() {
        let repo = crate::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let c = Clip::load(&repo.join("content/testcandidate/animations/UAL1/Idle_Loop.vrma")).unwrap();
        let d = c.deltas(0.0);
        let ang = |q: Q| 2.0 * q[3].abs().min(1.0).acos().to_degrees();
        for b in ["hips", "spine", "chest", "leftShoulder", "leftUpperArm", "leftLowerArm", "leftHand", "rightUpperArm", "leftUpperLeg", "head"] {
            let q = d[b]; eprintln!("DELTA {b}: winkel {:.1}° achse [{:.2},{:.2},{:.2}]", ang(q), q[0], q[1], q[2]);
        }
        let pos = c.rot.get(&c.bone_node["leftUpperArm"]).map(|k| k[0].1);
        eprintln!("rest upperarm {:?} anim0 {:?}", c.rest[c.bone_node["leftUpperArm"]], pos);
    }
}
