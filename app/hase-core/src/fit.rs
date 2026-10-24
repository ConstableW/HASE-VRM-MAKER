//! Proxy-Fitting (.mhclo): Barycentric-Bindung an Basis-Vertices + Offset, skaliert mit aktueller Körpergröße.
use crate::body::SCALE;
use crate::obj::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Proxy {
    pub name: String,
    pub obj_pos_len: usize,
    pub ia: Vec<u32>, pub ib: Vec<u32>, pub ic: Vec<u32>,
    pub w: Vec<[f32; 3]>,
    pub off: Vec<[f32; 3]>,
    pub scale: [(u32, u32, f32); 3],
    pub topo: RenderTopo,
    pub texture: Option<PathBuf>,
    pub alpha_mask: bool,
    pub diffuse: [f32; 3],
    pub adj: Vec<Vec<u32>>,
}

fn parse_mhmat(p: &Path) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(s) = std::fs::read_to_string(p) {
        for l in s.lines() { let t: Vec<&str> = l.split_whitespace().collect(); if t.len() >= 2 && !t[0].starts_with('#') && !t[0].starts_with("//") { m.insert(t[0].to_string(), t[1..].join(" ")); } }
    }
    m
}

impl Proxy {
    pub fn load(mhclo: &Path) -> Result<Proxy, String> {
        let dir = mhclo.parent().ok_or("kein Verzeichnis")?;
        let txt = std::fs::read_to_string(mhclo).map_err(|e| format!("{}: {e}", mhclo.display()))?;
        let (mut scale, mut obj_file, mut material, mut in_verts) = ([(0u32, 0u32, 1.0f32); 3], String::new(), String::new(), false);
        let (mut ia, mut ib, mut ic, mut w, mut off) = (vec![], vec![], vec![], vec![], vec![]);
        for l in txt.lines() {
            let t: Vec<&str> = l.split_whitespace().collect();
            if t.is_empty() || t[0].starts_with('#') { continue; }
            match t[0] {
                "verts" => in_verts = true,
                "delete_verts" | "weights" => in_verts = false,
                "x_scale" | "y_scale" | "z_scale" if t.len() >= 4 => { let i = match t[0] { "x_scale" => 0, "y_scale" => 1, _ => 2 }; scale[i] = (t[1].parse().unwrap_or(0), t[2].parse().unwrap_or(0), t[3].parse().unwrap_or(1.0)); in_verts = false; }
                "obj_file" => obj_file = t[1..].join(" "),
                "material" => material = t[1..].join(" "),
                _ if in_verts && t.len() == 9 => {
                    let f: Vec<f32> = t.iter().map(|x| x.parse().unwrap_or(0.0)).collect();
                    ia.push(f[0] as u32); ib.push(f[1] as u32); ic.push(f[2] as u32); w.push([f[3], f[4], f[5]]); off.push([f[6], f[7], f[8]]);
                }
                _ => {}
            }
        }
        let obj = parse_obj(&dir.join(&obj_file)).map_err(|e| format!("{obj_file}: {e}"))?;
        if obj.pos.len() != ia.len() { return Err(format!("{}: Vertexanzahl {} ≠ Bindungen {}", mhclo.display(), obj.pos.len(), ia.len())); }
        let topo = build_render_topo(&obj, |_| true);
        let mat = if material.is_empty() { Default::default() } else { parse_mhmat(&dir.join(&material)) };
        let texture = mat.get("diffuseTexture").map(|t| dir.join(t)).filter(|p| p.exists());
        let alpha_mask = mat.get("transparent").map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false);
        let diffuse = mat.get("diffuseColor").map(|v| { let f: Vec<f32> = v.split_whitespace().filter_map(|x| x.parse().ok()).collect(); if f.len() >= 3 { [f[0], f[1], f[2]] } else { [1.0; 3] } }).unwrap_or([1.0; 3]);
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); obj.pos.len()];
        for t in &topo.base_tris { for k in 0..3 { let (a, b) = (t[k] as usize, t[(k + 1) % 3]); if !adj[a].contains(&b) { adj[a].push(b); } let (a2, b2) = (b as usize, t[k]); if !adj[a2].contains(&b2) { adj[a2].push(b2); } } }
        Ok(Proxy { adj, name: mhclo.file_stem().unwrap().to_string_lossy().into(), obj_pos_len: obj.pos.len(), ia, ib, ic, w, off, scale, topo, texture, alpha_mask, diffuse })
    }

    /// Passt das Proxy an die aktuelle (gemorphte) Basis an. `out` erhält die Positionen aller Proxy-OBJ-Vertices.
    pub fn fit(&self, base: &[[f32; 3]], out: &mut Vec<[f32; 3]>) {
        let mut sc = [1.0f32; 3];
        for i in 0..3 { let (a, b, den) = self.scale[i]; if den > 0.0 { sc[i] = (base[a as usize][i] - base[b as usize][i]).abs() / (den * SCALE); } }
        out.clear();
        out.reserve(self.ia.len());
        for k in 0..self.ia.len() {
            let (a, b, c, w, o) = (base[self.ia[k] as usize], base[self.ib[k] as usize], base[self.ic[k] as usize], self.w[k], self.off[k]);
            out.push([
                w[0] * a[0] + w[1] * b[0] + w[2] * c[0] + o[0] * sc[0] * SCALE,
                w[0] * a[1] + w[1] * b[1] + w[2] * c[1] + o[1] * sc[1] * SCALE,
                w[0] * a[2] + w[1] * b[2] + w[2] * c[2] + o[2] * sc[2] * SCALE,
            ]);
        }
    }
}

/// Räumliches Gitter über Körper-Vertices (für Nächster-Punkt-Suche bei der Kollisionskorrektur).
pub struct BodyGrid { cell: f32, map: HashMap<(i32, i32, i32), Vec<u32>> }

impl BodyGrid {
    pub fn build(pos: &[[f32; 3]], verts: &[u32], cell: f32) -> BodyGrid {
        let mut map: HashMap<(i32, i32, i32), Vec<u32>> = HashMap::new();
        for &v in verts { let p = pos[v as usize]; map.entry(((p[0] / cell).floor() as i32, (p[1] / cell).floor() as i32, (p[2] / cell).floor() as i32)).or_default().push(v); }
        BodyGrid { cell, map }
    }
    pub fn nearest(&self, pos: &[[f32; 3]], p: [f32; 3]) -> Option<(u32, f32)> {
        let c = ((p[0] / self.cell).floor() as i32, (p[1] / self.cell).floor() as i32, (p[2] / self.cell).floor() as i32);
        let mut best: Option<(u32, f32)> = None;
        for dx in -1..=1 { for dy in -1..=1 { for dz in -1..=1 {
            if let Some(l) = self.map.get(&(c.0 + dx, c.1 + dy, c.2 + dz)) { for &v in l { let q = pos[v as usize]; let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2); if best.map(|b| d < b.1).unwrap_or(true) { best = Some((v, d)); } } }
        } } }
        best
    }
    /// Nächster Vertex innerhalb von `ring` Zellen, der `ok` erfüllt (z. B. passende Normalenrichtung).
    pub fn nearest_where(&self, pos: &[[f32; 3]], p: [f32; 3], ring: i32, ok: impl Fn(u32) -> bool) -> Option<(u32, f32)> {
        let c = ((p[0] / self.cell).floor() as i32, (p[1] / self.cell).floor() as i32, (p[2] / self.cell).floor() as i32);
        let mut best: Option<(u32, f32)> = None;
        for dx in -ring..=ring { for dy in -ring..=ring { for dz in -ring..=ring {
            if let Some(l) = self.map.get(&(c.0 + dx, c.1 + dy, c.2 + dz)) { for &v in l { if !ok(v) { continue; } let q = pos[v as usize]; let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2); if best.map(|b| d < b.1).unwrap_or(true) { best = Some((v, d)); } } }
        } } }
        best
    }
}

/// Kollisionskorrektur nach dem Fitting: Kleidungs-Vertices, die im Körper stecken (oder weniger als `gap` davor liegen), werden entlang der
/// Körpernormalen nach außen geschoben; die Verschiebung wird über die Kleidungs-Nachbarschaft geglättet (keine Spitzen).
/// Rückgabe: Anzahl korrigierter Vertices.
pub fn push_out(cloth: &mut [[f32; 3]], adj: &[Vec<u32>], tris: &[[u32; 3]], grid: &BodyGrid, body_pos: &[[f32; 3]], body_nrm: &[[f32; 3]], body_verts: &[u32], gap: f32) -> usize {
    let n = cloth.len();
    let mut push = vec![0f32; n]; let mut dir = vec![[0f32; 3]; n];
    let mut cn = Vec::new(); crate::eval::normals(cloth, tris, &mut cn); cn.resize(n, [0.0, 1.0, 0.0]);
    let reach2 = 0.045f32 * 0.045;
    for i in 0..n {
        let Some((b, d2)) = grid.nearest(body_pos, cloth[i]) else { continue };
        if d2 > reach2 { continue; }
        let (bp, bn) = (body_pos[b as usize], body_nrm[b as usize]);
        let d = (cloth[i][0] - bp[0]) * bn[0] + (cloth[i][1] - bp[1]) * bn[1] + (cloth[i][2] - bp[2]) * bn[2];
        if d < gap { push[i] = gap - d; dir[i] = bn; }
    }
    // Zweiter Durchgang: Körper-Vertices vor der Kleidung (z. B. Brustwarzen-Spitzen) schieben den nächsten Kleidungs-Vertex heraus
    let cg = {
        let ids: Vec<u32> = (0..n as u32).collect();
        BodyGrid::build(cloth, &ids, 0.02)
    };
    for &bv in body_verts {
        let b = body_pos[bv as usize];
        let Some((c, d2)) = cg.nearest(cloth, b) else { continue };
        if d2 > 0.035 * 0.035 { continue; }
        let (cp, nn) = (cloth[c as usize], cn[c as usize]);
        let d = (b[0] - cp[0]) * nn[0] + (b[1] - cp[1]) * nn[1] + (b[2] - cp[2]) * nn[2];
        if d > -gap && d < 0.025 { let need = d + gap; if need > push[c as usize] { push[c as usize] = need; dir[c as usize] = nn; } }
    }
    let raw = push.clone();
    // Glättung: Maximum aus eigenem Bedarf und gemittelter Nachbarschaft (verhindert Löcher und Spitzen)
    for _ in 0..4 {
        let prev = push.clone();
        for i in 0..n { if adj[i].is_empty() { continue; } let m: f32 = adj[i].iter().map(|&j| prev[j as usize]).sum::<f32>() / adj[i].len() as f32; push[i] = raw[i].max(0.5 * prev[i] + 0.5 * m); if dir[i] == [0.0; 3] { /* Richtung vom Nachbarn übernehmen */ if let Some(&j) = adj[i].iter().find(|&&j| dir[j as usize] != [0.0; 3]) { dir[i] = dir[j as usize]; } } }
    }
    let mut changed = 0;
    for i in 0..n { if push[i] > 1e-6 && dir[i] != [0.0; 3] { for k in 0..3 { cloth[i][k] += dir[i][k] * push[i]; } changed += 1; } }
    changed
}
