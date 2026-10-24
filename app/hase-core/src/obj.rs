//! Wavefront-OBJ (MakeHuman-Dialekt, Quads) mit UVs und Gruppen.
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct ObjData {
    pub pos: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub faces: Vec<Vec<(u32, u32)>>, // (Vertex, UV) je Ecke; UV = u32::MAX wenn fehlend
    pub face_group: Vec<u32>,
    pub groups: Vec<String>,
}

pub fn parse_obj(path: &Path) -> std::io::Result<ObjData> {
    let rd = BufReader::new(File::open(path)?);
    let mut d = ObjData { pos: vec![], uv: vec![], faces: vec![], face_group: vec![], groups: vec!["default".into()] };
    let mut cur = 0u32;
    for line in rd.lines().map_while(Result::ok) {
        if let Some(r) = line.strip_prefix("v ") {
            let mut it = r.split_whitespace().map(|x| x.parse::<f32>().unwrap_or(0.0));
            d.pos.push([it.next().unwrap_or(0.0), it.next().unwrap_or(0.0), it.next().unwrap_or(0.0)]);
        } else if let Some(r) = line.strip_prefix("vt ") {
            let mut it = r.split_whitespace().map(|x| x.parse::<f32>().unwrap_or(0.0));
            d.uv.push([it.next().unwrap_or(0.0), it.next().unwrap_or(0.0)]);
        } else if let Some(r) = line.strip_prefix("g ").or_else(|| line.strip_prefix("o ")) {
            let name = r.trim().to_string();
            cur = match d.groups.iter().position(|g| *g == name) { Some(i) => i as u32, None => { d.groups.push(name); d.groups.len() as u32 - 1 } };
        } else if let Some(r) = line.strip_prefix("f ") {
            let f: Vec<(u32, u32)> = r.split_whitespace().map(|t| {
                let mut p = t.split('/');
                let v = p.next().and_then(|x| x.parse::<i64>().ok()).unwrap_or(1) - 1;
                let vt = p.next().and_then(|x| x.parse::<i64>().ok()).map(|x| (x - 1) as u32).unwrap_or(u32::MAX);
                (v as u32, vt)
            }).collect();
            d.faces.push(f);
            d.face_group.push(cur);
        }
    }
    Ok(d)
}

/// Renderbare Topologie: Vertex-Duplikate je (Vertex, UV)-Paar, Dreiecksindizes.
pub struct RenderTopo {
    pub vmap: Vec<u32>,       // Renderer-Vertex -> OBJ-Vertex
    pub uv: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub base_tris: Vec<[u32; 3]>, // Dreiecke auf OBJ-Vertices (für Normalen)
}

pub fn build_render_topo(d: &ObjData, filter: impl Fn(&str) -> bool) -> RenderTopo {
    use std::collections::HashMap;
    let mut key: HashMap<(u32, u32), u32> = HashMap::new();
    let mut t = RenderTopo { vmap: vec![], uv: vec![], indices: vec![], base_tris: vec![] };
    for (f, &g) in d.faces.iter().zip(&d.face_group) {
        if !filter(&d.groups[g as usize]) || f.len() < 3 { continue; }
        let ids: Vec<u32> = f.iter().map(|&(v, vt)| *key.entry((v, vt)).or_insert_with(|| {
            t.vmap.push(v);
            t.uv.push(if vt == u32::MAX { [0.0, 0.0] } else { let u = d.uv[vt as usize]; [u[0], 1.0 - u[1]] });
            t.vmap.len() as u32 - 1
        })).collect();
        for k in 1..f.len() - 1 {
            t.indices.extend_from_slice(&[ids[0], ids[k], ids[k + 1]]);
            t.base_tris.push([f[0].0, f[k].0, f[k + 1].0]);
        }
    }
    t
}
