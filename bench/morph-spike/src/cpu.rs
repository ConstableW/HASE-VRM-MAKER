//! CPU-Referenz: Kosten der Morph-/Skinning-Algorithmen auf den realen Daten (kein GPU-Bezug).
use crate::data::*;
use crate::stats::*;
use rayon::prelude::*;
use serde_json::{json, Value};
use std::time::Instant;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn f(&mut self) -> f32 { (self.next() >> 40) as f32 / 16777216.0 }
}

fn bench<F: FnMut()>(iters: usize, mut f: F) -> Stats {
    for _ in 0..(iters / 10).max(3) { f(); }
    let mut v = Vec::with_capacity(iters);
    for _ in 0..iters { let t = Instant::now(); f(); v.push(t.elapsed().as_secs_f64() * 1e3); }
    stats(v)
}

pub fn run(mesh: &Mesh, morphs: &Morphs, skin: &Skin) -> Value {
    let nv = mesh.pos.len();
    let nt = morphs.names.len();
    let nnz = morphs.t_idx.len();
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut out = vec![[0f32; 3]; nv];
    let mut res = serde_json::Map::new();

    let sparse_mb = (nnz * (4 + 12)) as f64 / 1048576.0;
    res.insert("daten".into(), json!({
        "vertices": nv, "dreiecke": mesh.tris.len() / 3, "targets": nt, "nnz_gesamt": nnz,
        "nnz_pro_target_mittel": nnz as f64 / nt as f64,
        "sparse_csr_je_target_mb": sparse_mb,
        "sparse_csr_je_vertex_mb": (nnz * (4 + 12) + (nv + 1) * 4) as f64 / 1048576.0,
        "dicht_f32x3_mb": (nt * nv * 12) as f64 / 1048576.0,
        "dicht_f16x4_mb": (nt * nv * 8) as f64 / 1048576.0,
        "gather_je_vertex_max_eintraege": (0..nv).map(|i| morphs.v_off[i + 1] - morphs.v_off[i]).max().unwrap(),
    }));

    // aktive Mengen
    let mut sets = Vec::new();
    for &k in &[1usize, 8, 64, 128, nt] {
        let mut ids: Vec<usize> = (0..nt).collect();
        for i in 0..nt { let j = i + (rng.next() as usize) % (nt - i); ids.swap(i, j); }
        ids.truncate(k.min(nt));
        let mut w = vec![0f32; nt];
        for &t in &ids { w[t] = rng.f() * 2.0 - 1.0; }
        let active_nnz: usize = ids.iter().map(|&t| (morphs.t_off[t + 1] - morphs.t_off[t]) as usize).sum();
        sets.push((k, ids, w, active_nnz));
    }

    let mut scatter = serde_json::Map::new();
    let mut gather = serde_json::Map::new();
    let mut gather_par = serde_json::Map::new();
    for (k, ids, w, active_nnz) in &sets {
        let s = bench(300, || {
            out.copy_from_slice(&mesh.pos);
            for &t in ids {
                let wt = w[t];
                for e in morphs.t_off[t] as usize..morphs.t_off[t + 1] as usize {
                    let o = &mut out[morphs.t_idx[e] as usize]; let d = morphs.t_delta[e];
                    o[0] += wt * d[0]; o[1] += wt * d[1]; o[2] += wt * d[2];
                }
            }
            std::hint::black_box(&out);
        });
        scatter.insert(k.to_string(), json!({"aktive_nnz": active_nnz, "stats": s}));
        let g = bench(300, || {
            for i in 0..nv {
                let mut p = mesh.pos[i];
                for e in morphs.v_off[i] as usize..morphs.v_off[i + 1] as usize {
                    let wt = w[morphs.v_tgt[e] as usize];
                    if wt != 0.0 { let d = morphs.v_delta[e]; p[0] += wt * d[0]; p[1] += wt * d[1]; p[2] += wt * d[2]; }
                }
                out[i] = p;
            }
            std::hint::black_box(&out);
        });
        gather.insert(k.to_string(), json!({"stats": g}));
        let gp = bench(300, || {
            out.par_iter_mut().enumerate().for_each(|(i, o)| {
                let mut p = mesh.pos[i];
                for e in morphs.v_off[i] as usize..morphs.v_off[i + 1] as usize {
                    let wt = w[morphs.v_tgt[e] as usize];
                    if wt != 0.0 { let d = morphs.v_delta[e]; p[0] += wt * d[0]; p[1] += wt * d[1]; p[2] += wt * d[2]; }
                }
                *o = p;
            });
            std::hint::black_box(&out);
        });
        gather_par.insert(k.to_string(), json!({"stats": gp}));
    }
    res.insert("morph_scatter_je_target_1thread".into(), Value::Object(scatter));
    res.insert("morph_gather_je_vertex_1thread".into(), Value::Object(gather));
    res.insert("morph_gather_je_vertex_rayon".into(), Value::Object(gather_par));

    // inkrementell: nur ein Target ändert sich
    let mut per = Vec::new();
    let mut tmp = mesh.pos.clone();
    for t in 0..nt {
        let t0 = Instant::now();
        let dw = 0.05f32;
        for e in morphs.t_off[t] as usize..morphs.t_off[t + 1] as usize {
            let o = &mut tmp[morphs.t_idx[e] as usize]; let d = morphs.t_delta[e];
            o[0] += dw * d[0]; o[1] += dw * d[1]; o[2] += dw * d[2];
        }
        per.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    std::hint::black_box(&tmp);
    res.insert("morph_inkrementell_ein_target_alle_609".into(), json!(stats(per)));

    // Normalen CPU
    let mut nrm = vec![[0f32; 3]; nv];
    let n_stat = bench(300, || {
        nrm.iter_mut().for_each(|n| *n = [0.0; 3]);
        for t in mesh.tris.chunks_exact(3) {
            let (a, b, c) = (mesh.pos[t[0] as usize], mesh.pos[t[1] as usize], mesh.pos[t[2] as usize]);
            let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            for &i in t { let x = &mut nrm[i as usize]; x[0] += n[0]; x[1] += n[1]; x[2] += n[2]; }
        }
        std::hint::black_box(&nrm);
    });
    res.insert("normalen_neuberechnung_1thread".into(), json!(n_stat));

    // Skinning CPU
    let mats: Vec<[f32; 12]> = (0..skin.bones).map(|b| { let a = 0.1 * b as f32; let (s, c) = a.sin_cos(); [c, -s, 0.0, 0.0, s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.01] }).collect();
    let sk = bench(300, || {
        for i in 0..nv {
            let p = mesh.pos[i]; let mut r = [0f32; 3];
            for k in 0..4 {
                let m = &mats[skin.joints[i][k] as usize]; let w = skin.weights[i][k];
                r[0] += w * (m[0] * p[0] + m[1] * p[1] + m[2] * p[2] + m[3]);
                r[1] += w * (m[4] * p[0] + m[5] * p[1] + m[6] * p[2] + m[7]);
                r[2] += w * (m[8] * p[0] + m[9] * p[1] + m[10] * p[2] + m[11]);
            }
            out[i] = r;
        }
        std::hint::black_box(&out);
    });
    res.insert("skinning_cpu_4_gewichte_1thread".into(), json!(sk));
    Value::Object(res)
}
