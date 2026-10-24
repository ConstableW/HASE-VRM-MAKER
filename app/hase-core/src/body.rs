//! Body-Daten: Basismesh (hm08), Sparse-Morph-Targets (CSR je Target) mit Binär-Cache.
use crate::obj::*;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

pub const SCALE: f32 = 0.1; // MakeHuman dm -> m

pub struct Morphs {
    pub names: Vec<String>, // "gruppe/name"
    pub t_off: Vec<u32>,
    pub t_idx: Vec<u32>,
    pub t_delta: Vec<[f32; 3]>,
}

pub struct Body {
    pub base_pos: Vec<[f32; 3]>, // alle 19.158 OBJ-Vertices (inkl. Helper), Meter
    pub obj: ObjData,
    pub topo: RenderTopo, // nur Gruppe "body"
    pub morphs: Morphs,
    pub load_ms: f64,
    pub from_cache: bool,
}

impl Morphs {
    pub fn index_of(&self, name: &str) -> Option<usize> { self.names.iter().position(|n| n == name) }
    pub fn nnz(&self) -> usize { self.t_idx.len() }
}

fn collect_targets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root) {
        for g in rd.flatten() {
            if g.path().is_dir() {
                if let Ok(rd2) = std::fs::read_dir(g.path()) {
                    for f in rd2.flatten() { if f.path().to_string_lossy().ends_with(".target.gz") { out.push(f.path()); } }
                }
            }
        }
    }
    out.sort();
    out
}

fn parse_targets(root: &Path, nv: usize) -> Morphs {
    use rayon::prelude::*;
    let files = collect_targets(root);
    let parsed: Vec<(String, Vec<(u32, [f32; 3])>)> = files.par_iter().map(|p| {
        let rd = BufReader::new(GzDecoder::new(File::open(p).unwrap()));
        let mut e = Vec::new();
        for line in rd.lines().map_while(Result::ok) {
            if line.starts_with('#') || line.trim().is_empty() { continue; }
            let mut it = line.split_whitespace();
            let i: usize = it.next().and_then(|x| x.parse().ok()).unwrap_or(usize::MAX);
            let d: Vec<f32> = it.take(3).map(|x| x.parse().unwrap_or(0.0)).collect();
            if i < nv && d.len() == 3 { e.push((i as u32, [d[0] * SCALE, d[1] * SCALE, d[2] * SCALE])); }
        }
        e.sort_by_key(|x| x.0);
        let name = format!("{}/{}", p.parent().unwrap().file_name().unwrap().to_string_lossy(), p.file_name().unwrap().to_string_lossy().trim_end_matches(".target.gz"));
        (name, e)
    }).collect();
    let (mut names, mut t_off, mut t_idx, mut t_delta) = (vec![], vec![0u32], vec![], vec![]);
    for (n, e) in parsed { names.push(n); for (i, d) in e { t_idx.push(i); t_delta.push(d); } t_off.push(t_idx.len() as u32); }
    Morphs { names, t_off, t_idx, t_delta }
}

const MAGIC: &[u8; 8] = b"HASEMRF1";

fn save_cache(path: &Path, m: &Morphs, stamp: u64) -> std::io::Result<()> {
    if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
    let mut f = std::io::BufWriter::new(File::create(path)?);
    f.write_all(MAGIC)?;
    f.write_all(&stamp.to_le_bytes())?;
    let names = m.names.join("\n");
    for n in [m.names.len(), names.len(), m.t_off.len(), m.t_idx.len()] { f.write_all(&(n as u64).to_le_bytes())?; }
    f.write_all(names.as_bytes())?;
    f.write_all(bytemuck::cast_slice(&m.t_off))?;
    f.write_all(bytemuck::cast_slice(&m.t_idx))?;
    f.write_all(bytemuck::cast_slice(&m.t_delta))?;
    Ok(())
}

fn load_cache(path: &Path, stamp: u64) -> Option<Morphs> {
    let mut buf = Vec::new();
    File::open(path).ok()?.read_to_end(&mut buf).ok()?;
    if buf.len() < 56 || &buf[..8] != MAGIC || u64::from_le_bytes(buf[8..16].try_into().ok()?) != stamp { return None; }
    let rd = |o: usize| u64::from_le_bytes(buf[o..o + 8].try_into().unwrap()) as usize;
    let (nn, nlen, noff, nnz) = (rd(16), rd(24), rd(32), rd(40));
    let mut o = 48;
    let names: Vec<String> = std::str::from_utf8(buf.get(o..o + nlen)?).ok()?.split('\n').map(String::from).collect();
    if names.len() != nn { return None; }
    o += nlen;
    let cast = |o: usize, bytes: usize| buf.get(o..o + bytes);
    let t_off: Vec<u32> = bytemuck::pod_collect_to_vec(cast(o, noff * 4)?); o += noff * 4;
    let t_idx: Vec<u32> = bytemuck::pod_collect_to_vec(cast(o, nnz * 4)?); o += nnz * 4;
    let t_delta: Vec<[f32; 3]> = bytemuck::pod_collect_to_vec(cast(o, nnz * 12)?);
    Some(Morphs { names, t_off, t_idx, t_delta })
}

fn stamp_of(dir: &Path) -> u64 {
    // Summe aus Dateianzahl und Gesamtgröße der Targets + Versionskonstante (billiger Änderungsdetektor)
    let mut s = 1u64;
    for p in collect_targets(dir) { s = s.wrapping_mul(1099511628211).wrapping_add(std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)); }
    s
}

impl Body {
    pub fn load(data_root: &Path, cache_dir: &Path) -> Result<Body, String> {
        let t0 = std::time::Instant::now();
        let obj = parse_obj(&data_root.join("base.obj")).map_err(|e| format!("base.obj: {e}"))?;
        let base_pos: Vec<[f32; 3]> = obj.pos.iter().map(|p| [p[0] * SCALE, p[1] * SCALE, p[2] * SCALE]).collect();
        let topo = build_render_topo(&obj, |g| g == "body");
        let troot = data_root.join("targets");
        let stamp = stamp_of(&troot);
        let cache = cache_dir.join("morphs.hasebin");
        let (morphs, from_cache) = match load_cache(&cache, stamp) {
            Some(m) => (m, true),
            None => { let m = parse_targets(&troot, base_pos.len()); let _ = save_cache(&cache, &m, stamp); (m, false) }
        };
        let mut morphs = morphs;
        let mut nrm = Vec::new();
        crate::eval::normals(&base_pos, &topo.base_tris, &mut nrm);
        nrm.resize(base_pos.len(), [0.0, 1.0, 0.0]);
        crate::anatomy::append_breast(&mut morphs, &base_pos, &nrm);
        crate::anatomy::append_glutes(&mut morphs, &base_pos, &nrm);
        Ok(Body { base_pos, obj, topo, morphs, load_ms: t0.elapsed().as_secs_f64() * 1e3, from_cache })
    }
}
