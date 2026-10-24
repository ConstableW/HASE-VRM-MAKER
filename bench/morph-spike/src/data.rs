//! Laden der realen Testcandidate-Daten: Basismesh (body-Gruppe), 609 Morph-Targets, Skin-Gewichte.
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct Mesh {
    pub pos: Vec<[f32; 3]>,
    pub tris: Vec<u32>,
    pub orig_to_new: Vec<i32>,
    pub orig_vertex_count: usize,
}

pub struct Morphs {
    pub names: Vec<String>,
    // CSR je Target (Scatter-Pfad)
    pub t_off: Vec<u32>,
    pub t_idx: Vec<u32>,
    pub t_delta: Vec<[f32; 3]>,
    // CSR je Vertex (Gather-Pfad, GPU-geeignet)
    pub v_off: Vec<u32>,
    pub v_tgt: Vec<u32>,
    pub v_delta: Vec<[f32; 3]>,
}

pub struct Skin {
    pub bones: usize,
    pub joints: Vec<[u32; 4]>,
    pub weights: Vec<[f32; 4]>,
}

pub fn load_mesh(base_obj: &Path) -> Mesh {
    let f = BufReader::new(File::open(base_obj).expect("base.obj"));
    let (mut v, mut quads, mut group) = (Vec::<[f32; 3]>::new(), Vec::<[u32; 4]>::new(), String::new());
    for line in f.lines().map_while(Result::ok) {
        if let Some(r) = line.strip_prefix("v ") {
            let p: Vec<f32> = r.split_whitespace().take(3).map(|x| x.parse().unwrap()).collect();
            v.push([p[0], p[1], p[2]]);
        } else if let Some(r) = line.strip_prefix("g ") {
            group = r.trim().to_string();
        } else if let Some(r) = line.strip_prefix("f ") {
            if group == "body" {
                let ix: Vec<u32> = r.split_whitespace().map(|t| t.split('/').next().unwrap().parse::<u32>().unwrap() - 1).collect();
                if ix.len() == 4 { quads.push([ix[0], ix[1], ix[2], ix[3]]); } else if ix.len() == 3 { quads.push([ix[0], ix[1], ix[2], ix[2]]); }
            }
        }
    }
    let mut orig_to_new = vec![-1i32; v.len()];
    let mut pos = Vec::new();
    for q in &quads {
        for &i in q {
            if orig_to_new[i as usize] < 0 { orig_to_new[i as usize] = -2; }
        }
    }
    for (i, m) in orig_to_new.iter_mut().enumerate() {
        if *m == -2 { *m = pos.len() as i32; pos.push([v[i][0] * 0.1, v[i][1] * 0.1, v[i][2] * 0.1]); }
    }
    let mut tris = Vec::new();
    for q in &quads {
        let n: Vec<u32> = q.iter().map(|&i| orig_to_new[i as usize] as u32).collect();
        tris.extend_from_slice(&[n[0], n[1], n[2]]);
        if q[2] != q[3] { tris.extend_from_slice(&[n[0], n[2], n[3]]); }
    }
    Mesh { pos, tris, orig_to_new, orig_vertex_count: v.len() }
}

fn collect_targets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for g in std::fs::read_dir(root).unwrap().flatten() {
        if g.path().is_dir() {
            for f in std::fs::read_dir(g.path()).unwrap().flatten() {
                if f.path().to_string_lossy().ends_with(".target.gz") { out.push(f.path()); }
            }
        }
    }
    out.sort();
    out
}

pub fn load_morphs(target_root: &Path, mesh: &Mesh) -> Morphs {
    let files = collect_targets(target_root);
    let nv = mesh.pos.len();
    let (mut names, mut t_off, mut t_idx, mut t_delta) = (Vec::new(), vec![0u32], Vec::new(), Vec::new());
    for p in &files {
        let rd = BufReader::new(GzDecoder::new(File::open(p).unwrap()));
        let mut entries: Vec<(u32, [f32; 3])> = Vec::new();
        for line in rd.lines().map_while(Result::ok) {
            if line.starts_with('#') || line.trim().is_empty() { continue; }
            let mut it = line.split_whitespace();
            let i: usize = it.next().unwrap().parse().unwrap();
            let d: Vec<f32> = it.take(3).map(|x| x.parse().unwrap()).collect();
            if i < mesh.orig_to_new.len() && mesh.orig_to_new[i] >= 0 {
                entries.push((mesh.orig_to_new[i] as u32, [d[0] * 0.1, d[1] * 0.1, d[2] * 0.1]));
            }
        }
        entries.sort_by_key(|e| e.0);
        for (i, d) in entries { t_idx.push(i); t_delta.push(d); }
        t_off.push(t_idx.len() as u32);
        names.push(format!("{}/{}", p.parent().unwrap().file_name().unwrap().to_string_lossy(), p.file_name().unwrap().to_string_lossy().trim_end_matches(".target.gz")));
    }
    // Transponieren: je Vertex
    let mut counts = vec![0u32; nv + 1];
    for &i in &t_idx { counts[i as usize + 1] += 1; }
    for i in 0..nv { counts[i + 1] += counts[i]; }
    let v_off = counts.clone();
    let mut cursor = counts[..nv].to_vec();
    let nnz = t_idx.len();
    let (mut v_tgt, mut v_delta) = (vec![0u32; nnz], vec![[0f32; 3]; nnz]);
    for t in 0..names.len() {
        for k in t_off[t] as usize..t_off[t + 1] as usize {
            let i = t_idx[k] as usize;
            let c = cursor[i] as usize;
            v_tgt[c] = t as u32; v_delta[c] = t_delta[k]; cursor[i] += 1;
        }
    }
    Morphs { names, t_off, t_idx, t_delta, v_off, v_tgt, v_delta }
}

pub fn load_skin(weights_json: &Path, mesh: &Mesh) -> Skin {
    let v: serde_json::Value = serde_json::from_reader(File::open(weights_json).unwrap()).unwrap();
    let w = v["weights"].as_object().unwrap();
    let bone_ids: HashMap<&str, u32> = w.keys().enumerate().map(|(i, k)| (k.as_str(), i as u32)).collect();
    let nv = mesh.pos.len();
    let mut acc: Vec<Vec<(u32, f32)>> = vec![Vec::new(); nv];
    for (b, list) in w {
        for e in list.as_array().unwrap() {
            let (vi, wt) = (e[0].as_u64().unwrap() as usize, e[1].as_f64().unwrap() as f32);
            if vi < mesh.orig_to_new.len() && mesh.orig_to_new[vi] >= 0 { acc[mesh.orig_to_new[vi] as usize].push((bone_ids[b.as_str()], wt)); }
        }
    }
    let (mut joints, mut weights) = (Vec::with_capacity(nv), Vec::with_capacity(nv));
    for mut a in acc {
        a.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
        a.truncate(4);
        let s: f32 = a.iter().map(|x| x.1).sum::<f32>().max(1e-6);
        let (mut j, mut ww) = ([0u32; 4], [0f32; 4]);
        for (k, (b, x)) in a.iter().enumerate() { j[k] = *b; ww[k] = x / s; }
        joints.push(j); weights.push(ww);
    }
    Skin { bones: w.len(), joints, weights }
}
